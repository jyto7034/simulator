//! The opt-in native owner of new run relic operations. The outer World
//! rollback capture remains outside this added native allocation scope.
use super::GameCore;
use crate::game::{battle::runtime_control as native, behavior::GameError};
impl GameCore {
    pub(super) fn ensure_relic_operation(&self) -> Result<(), GameError> {
        if native::is_active() {
            return native::poll();
        }
        let settings = self.game_data.runtime_safety();
        native::run_with_profile(settings.native_runtime(), settings, || {
            native::seed_retained_extent(self.state.relic_owned_extent()?)
        })
    }
}

use crate::game::{
    behavior::BehaviorResult,
    relic::{RelicOperationKind, RelicOperationPreviewDto},
    reward::RewardEffect,
};
use uuid::Uuid;
impl GameCore {
    pub(super) fn ensure_relic_grant(&self, effects: &[RewardEffect]) -> Result<(), GameError> {
        if effects.iter().any(RewardEffect::uses_relic_owner) {
            self.ensure_relic_operation()?;
        }
        Ok(())
    }
    pub(super) fn handle_request_relic_state(&self) -> Result<BehaviorResult, GameError> {
        if self.state.run.is_none() {
            return Err(GameError::InvalidAction);
        }
        self.ensure_relic_operation()?;
        Ok(BehaviorResult::RelicState {
            result: self.state.relics.snapshot(self.state.crafting_resources)?,
        })
    }
    pub(super) fn handle_request_relic_effective_plan(
        &self,
        employee: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        if self.state.run.is_none() {
            return Err(GameError::InvalidAction);
        }
        self.ensure_relic_operation()?;
        if self.state.roster.get(&employee).is_none() {
            return Err(GameError::UnitNotFound);
        }
        if !crate::game::relic::structural::employee_has_structural_rules(
            &self.state.relics,
            employee,
            &self.game_data,
        )
        .map_err(|error| GameError::InvalidStaticData(error.to_string()))?
        {
            return Ok(BehaviorResult::RelicEffectivePlan { result: None });
        }
        let providers = self.structural_providers(employee)?;
        let result = crate::game::relic::structural::explain_effective_plan(
            &self.game_data,
            employee,
            &self.state.relics,
            &providers,
        )
        .map_err(|error| GameError::InvalidStaticData(error.to_string()))?;
        Ok(BehaviorResult::RelicEffectivePlan { result })
    }
    pub(super) fn structural_providers(
        &self,
        employee: Uuid,
    ) -> Result<Vec<crate::game::relic::structural::ProspectiveProvider>, GameError> {
        use crate::game::relic::structural::ProspectiveProvider;
        let employee_value = self
            .state
            .roster
            .get(&employee)
            .ok_or(GameError::UnitNotFound)?;
        let mut count = usize::from(employee_value.skill_fragments.active_copy_uuid().is_some());
        for item in employee_value.loadout.item_slot.iter() {
            crate::authoring::control::charge(
                self.game_data.equipment_data.items.len() as u64,
                "structural_provider_definition",
            )?;
            let definition = self
                .game_data
                .equipment_data
                .get_by_uuid(&item.base_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;
            count = count
                .checked_add(definition.ability_activations.len())
                .ok_or_else(|| native::invariant("structural_provider_count"))?;
        }
        native::admit_retained_growth(
            0,
            count.checked_mul(std::mem::size_of::<ProspectiveProvider>()),
            "structural_provider_table",
        )?;
        let mut providers = Vec::with_capacity(count);
        if let Some(copy_uuid) = employee_value.skill_fragments.active_copy_uuid() {
            let (copy, _, form) = self
                .state
                .skill_fragments
                .resolve(&copy_uuid, &self.game_data.skill_fragment_data)?;
            if copy.equipped_to != Some(employee) {
                return Err(GameError::InvalidAction);
            }
            crate::authoring::control::charge_serialized(copy, "structural_fragment_provider")?;
            crate::authoring::control::charge_serialized(
                &form.skill_id,
                "structural_fragment_skill",
            )?;
            providers.push(ProspectiveProvider::Fragment(
                crate::game::skill_fragment::FragmentSourceSnapshot {
                    copy_uuid,
                    definition_id: copy.definition_id.clone(),
                    form_id: copy.selected_form.clone(),
                    skill_id: form.skill_id.clone(),
                },
            ));
        }
        for item in employee_value.loadout.item_slot.iter() {
            let definition = self
                .game_data
                .equipment_data
                .get_by_uuid(&item.base_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;
            for activation in &definition.ability_activations {
                let scan = providers
                    .len()
                    .checked_mul(definition.id.len() + activation.ability_id.as_str().len() + 1)
                    .ok_or_else(|| native::invariant("structural_provider_dedup_work"))?;
                crate::authoring::control::charge(scan as u64, "structural_provider_dedup")?;
                if providers.iter().any(|provider| matches!(provider, ProspectiveProvider::Equipment { definition_id, copy_uuid, skill_id }
                    if definition_id == &definition.id && *copy_uuid == item.instance_uuid && skill_id == &activation.ability_id)) { continue; }
                crate::authoring::control::charge_serialized(
                    &definition.id,
                    "structural_equipment_provider",
                )?;
                crate::authoring::control::charge_serialized(
                    &activation.ability_id,
                    "structural_equipment_skill",
                )?;
                providers.push(ProspectiveProvider::Equipment {
                    definition_id: definition.id.clone(),
                    copy_uuid: item.instance_uuid,
                    skill_id: activation.ability_id.clone(),
                });
            }
        }
        Ok(providers)
    }
    pub(super) fn capture_structural_battle_owner(
        &self,
    ) -> Result<
        Option<std::sync::Arc<crate::game::battle::core::effective_skill::EffectiveBattleOwner>>,
        GameError,
    > {
        let mut plans = std::collections::BTreeMap::new();
        if !self.state.relics.has_equipped() {
            return Ok(None);
        }
        crate::authoring::control::charge(
            self.state.roster.len() as u64,
            "effective_battle_employees",
        )?;
        for employee in self.state.roster.iter() {
            if !crate::game::relic::structural::employee_has_structural_rules(
                &self.state.relics,
                employee.uuid,
                &self.game_data,
            )
            .map_err(|error| GameError::InvalidStaticData(error.to_string()))?
            {
                continue;
            }
            let providers = self.structural_providers(employee.uuid)?;
            if let Some(plan) = crate::game::relic::structural::compile_effective_plan(
                crate::game::relic::structural::EffectivePlanInput {
                    data: &self.game_data,
                    employee_uuid: employee.uuid,
                    inventory: &self.state.relics,
                    providers: &providers,
                },
            )
            .map_err(|error| GameError::InvalidStaticData(error.to_string()))?
            {
                native::admit_retained_growth(
                    0,
                    Some(
                        64 + std::mem::size_of::<(
                            Uuid,
                            std::sync::Arc<crate::game::relic::structural::EmployeeEffectivePlan>,
                        )>(),
                    ),
                    "effective_battle_employee_entry",
                )?;
                plans.insert(employee.uuid, plan);
            }
        }
        crate::game::battle::core::effective_skill::EffectiveBattleOwner::new(plans)
    }
    pub(super) fn validate_structural_loadouts(&self) -> Result<(), GameError> {
        if !self.state.relics.has_equipped() {
            return Ok(());
        }
        crate::authoring::control::charge(
            self.state.roster.len() as u64,
            "structural_loadout_scan",
        )?;
        for employee in self.state.roster.iter() {
            if !crate::game::relic::structural::employee_has_structural_rules(
                &self.state.relics,
                employee.uuid,
                &self.game_data,
            )
            .map_err(|error| GameError::InvalidStaticData(error.to_string()))?
            {
                continue;
            }
            let providers = self.structural_providers(employee.uuid)?;
            crate::game::relic::structural::validate_effective_configuration(
                crate::game::relic::structural::EffectivePlanInput {
                    data: &self.game_data,
                    employee_uuid: employee.uuid,
                    inventory: &self.state.relics,
                    providers: &providers,
                },
            )
            .map_err(|error| GameError::InvalidStaticData(error.to_string()))?;
        }
        Ok(())
    }
    pub(super) fn handle_request_relic_numeric_plan(
        &self,
        employee: Uuid,
    ) -> Result<BehaviorResult, GameError> {
        if self.state.run.is_none() {
            return Err(GameError::InvalidAction);
        }
        let employee_value = self
            .state
            .roster
            .get(&employee)
            .ok_or(GameError::UnitNotFound)?;
        self.ensure_relic_operation()?;
        // The reused profile path owns three copies: employee->draft,
        // draft->display profile, and draft->final stats. It also assembles
        // equipment vectors, two final-stat binding maps and a growth clone.
        // Admission precedes all of them; outside World capture stays outside.
        for _ in 0..3 {
            crate::game::relic::allocation::admit_profile_projection(
                &employee_value.combat_profile.battle_profile,
            )?;
            crate::game::resource::store::preflight_resource_clone(
                &employee_value.combat_profile.battle_profile,
            )
            .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
        }
        crate::game::relic::allocation::admit_profile_projection(
            &employee_value.combat_profile.growth_stacks,
        )?;
        let growth_buckets =
            crate::owned_collections::MapCloneHistory::from_tombstone_free_capacity(
                employee_value
                    .combat_profile
                    .growth_stacks
                    .stacks
                    .capacity(),
            )
            .bucket_bound();
        native::admit_retained_growth(
            0,
            growth_buckets
                .and_then(|n| {
                    n.checked_mul(std::mem::size_of::<(crate::game::growth::GrowthId, i32)>() + 1)
                })
                .and_then(|n| n.checked_add(16)),
            "relic_plan_growth_table",
        )?;
        crate::game::resource::store::preflight_resource_clone(
            &employee_value.combat_profile.growth_stacks,
        )
        .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
        let mut items = 0usize;
        for equipped in employee_value.loadout.item_slot.iter() {
            items = items
                .checked_add(1)
                .ok_or_else(|| native::invariant("relic_plan_loadout_count"))?;
            crate::authoring::control::charge(
                self.game_data.equipment_data.items.len() as u64,
                "relic_plan_profile_equipment",
            )?;
            let equipment = self
                .game_data
                .equipment_data
                .get_by_uuid(&equipped.base_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;
            if let Some(weapon) = &equipment.weapon_profile {
                for _ in 0..2 {
                    crate::game::relic::allocation::admit_profile_projection(weapon)?;
                    crate::game::resource::store::preflight_resource_clone(weapon)
                        .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
                }
            }
            let effect_count = equipment
                .triggered_effects
                .values()
                .try_fold(0usize, |n, effects| n.checked_add(effects.len()))
                .ok_or_else(|| native::invariant("relic_plan_profile_effect_count"))?;
            crate::authoring::control::charge(effect_count as u64, "relic_plan_profile_effects")?;
            let owned = self
                .state
                .inventory
                .equipments
                .get_item(&equipped.instance_uuid)
                .ok_or(GameError::InventoryItemNotFound)?;
            if let Some(recipe) = self
                .game_data
                .equipment_data
                .get_enhancement_recipe_by_equipment_id(&equipment.id)
            {
                let enhancement_work = recipe
                    .modifiers_per_level
                    .len()
                    .checked_mul(usize::from(owned.enhancement_level))
                    .ok_or_else(|| native::invariant("relic_plan_profile_enhancement_work"))?;
                crate::authoring::control::charge(
                    enhancement_work as u64,
                    "relic_plan_profile_enhancement",
                )?;
            }
        }
        let item_bytes = items.checked_mul(
            std::mem::size_of::<crate::game::battle::types::BattleEquipmentBinding>()
                + std::mem::size_of::<crate::game::battle::types::BattleEquipmentEnhancement>()
                + (std::mem::size_of::<(Uuid, Uuid)>() * 11 + 224)
                + (std::mem::size_of::<(Uuid, u8)>() * 11 + 224),
        );
        native::admit_retained_growth(0, item_bytes, "relic_plan_profile_loadout")?;
        if let Some(copy) = employee_value.skill_fragments.active_copy_uuid() {
            let (copy, _, form) = self
                .state
                .skill_fragments
                .resolve(&copy, &self.game_data.skill_fragment_data)?;
            // Source strings enter the assembled profile and both of its
            // clones, while compatibility diagnostics may own selected form data.
            for _ in 0..3 {
                crate::game::relic::allocation::admit_profile_projection(copy)?;
                crate::game::relic::allocation::admit_profile_projection(form)?;
                crate::game::resource::store::preflight_resource_clone(copy)
                    .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
                crate::game::resource::store::preflight_resource_clone(&form.skill_id)
                    .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
            }
        }
        let draft = crate::game::combat_setup::player_spawns::battle_unit_draft_for_employee_with_availability(
            &self.state.roster, &self.state.inventory, &self.state.skill_fragments,
            &self.game_data, employee, false,
        )?;
        let profile = match draft.combat_profile(&self.game_data) {
            Ok(profile) => profile,
            Err(GameError::WeaponRequired) => {
                return Ok(BehaviorResult::RelicNumericPlan {
                    result: crate::game::relic::plan::unavailable_plan(
                        employee,
                        &self.state.relics,
                        &self.game_data,
                        self.game_data.identity(),
                    )?,
                })
            }
            Err(error) => return Err(error),
        };
        let base_stats = draft.effective_stats(&self.game_data)?;
        Ok(BehaviorResult::RelicNumericPlan {
            result: crate::game::relic::plan::build_plan(
                employee,
                &self.state.relics,
                &employee_value.loadout.item_slot,
                &self.game_data,
                &profile,
                base_stats,
                self.game_data.identity(),
            )?,
        })
    }
    pub(super) fn handle_equip_relic(
        &mut self,
        copy: Uuid,
        employee: Uuid,
        slot: u8,
    ) -> Result<BehaviorResult, GameError> {
        if !self.is_loadout_action_context()
            || !self
                .state
                .roster
                .get(&employee)
                .is_some_and(|e| e.is_alive())
        {
            return Err(GameError::InvalidAction);
        }
        self.ensure_relic_operation()?;
        self.state
            .relics
            .equip(copy, employee, slot as usize, &self.game_data.relic_data)?;
        self.handle_request_relic_state()
    }
    pub(super) fn handle_unequip_relic(&mut self, copy: Uuid) -> Result<BehaviorResult, GameError> {
        if !self.is_loadout_action_context() {
            return Err(GameError::InvalidAction);
        }
        self.ensure_relic_operation()?;
        self.state.relics.unequip(copy)?;
        self.handle_request_relic_state()
    }
    pub(super) fn handle_preview_relic(
        &self,
        copy: Uuid,
        operation: RelicOperationKind,
    ) -> Result<BehaviorResult, GameError> {
        if !self.is_loadout_action_context()
            || matches!(operation, RelicOperationKind::Dismantle) && !self.is_in_maintenance_node()
        {
            return Err(GameError::InvalidAction);
        }
        self.ensure_relic_operation()?;
        let item = self
            .state
            .relics
            .get(copy)
            .ok_or(GameError::InvalidAction)?;
        crate::authoring::control::charge(
            self.game_data.relic_data.relics.len() as u64,
            "relic_preview_definition",
        )?;
        let definition = self
            .game_data
            .relic_data
            .get_by_id(&item.definition_id)
            .ok_or(GameError::InvalidAction)?;
        let empty_penalties = Vec::new();
        let (effects, penalties, catalyst_cost, dust) = match operation {
            RelicOperationKind::Corrupt => {
                self.state
                    .relics
                    .validate_corruption_target(copy, &self.game_data.relic_data)?;
                let variant = definition
                    .corruption
                    .as_ref()
                    .ok_or(GameError::InvalidAction)?;
                (&variant.effects, &variant.penalties, 1, 0)
            }
            RelicOperationKind::Dismantle => {
                let (effects, penalties) = match item.variant {
                    crate::game::relic::RelicVariant::Normal => {
                        (&definition.effects, &empty_penalties)
                    }
                    crate::game::relic::RelicVariant::Corrupt => {
                        let variant = definition
                            .corruption
                            .as_ref()
                            .ok_or(GameError::InvalidAction)?;
                        (&variant.effects, &variant.penalties)
                    }
                };
                (effects, penalties, 0, definition.dismantle_equipment_dust)
            }
        };
        let extent = effects
            .iter()
            .try_fold(item.definition_id.len(), |bytes, effect| {
                effect.retained_extent().and_then(|n| bytes.checked_add(n))
            })
            .and_then(|n| {
                n.checked_add(penalties.len().checked_mul(std::mem::size_of::<
                    crate::game::data::relic_data::RelicPenalty,
                >())?)
            });
        native::admit_retained_growth(0, extent, "relic_operation_preview")?;
        if !crate::authoring::control::is_active() {
            crate::game::resource::store::preflight_resource_clone(item)
                .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
            crate::game::resource::store::preflight_resource_clone(effects)
                .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
            crate::game::resource::store::preflight_resource_clone(penalties)
                .map_err(crate::game::resource::store::ResourceError::into_game_error)?;
        }
        Ok(BehaviorResult::RelicPreview {
            result: RelicOperationPreviewDto {
                copy: crate::authoring::clone_shape::controlled_clone(item)?,
                operation,
                effects: crate::authoring::clone_shape::controlled_clone(effects)?,
                penalties: crate::authoring::clone_shape::controlled_clone(penalties)?,
                catalyst_cost,
                equipment_dust_gained: dust,
                irreversible: true,
                equipped_to: self.state.relics.equipped_to(copy)?,
            },
        })
    }
    pub(super) fn handle_corrupt_relic(&mut self, copy: Uuid) -> Result<BehaviorResult, GameError> {
        if !self.is_loadout_action_context() {
            return Err(GameError::InvalidAction);
        }
        self.ensure_relic_operation()?;
        self.state
            .relics
            .validate_corruption_target(copy, &self.game_data.relic_data)?;
        self.state.crafting_resources.consume_relic_catalyst()?;
        self.state
            .relics
            .corrupt(copy, &self.game_data.relic_data)?;
        self.handle_request_relic_state()
    }
    pub(super) fn handle_dismantle_relic(
        &mut self,
        copy: Uuid,
        confirmed: bool,
    ) -> Result<BehaviorResult, GameError> {
        if !self.is_in_maintenance_node() {
            return Err(GameError::InvalidAction);
        }
        self.ensure_relic_operation()?;
        let item = self
            .state
            .relics
            .get(copy)
            .ok_or(GameError::InvalidAction)?;
        if !confirmed {
            return self.handle_request_relic_state();
        }
        crate::authoring::control::charge(
            self.game_data.relic_data.relics.len() as u64,
            "relic_dismantle_definition",
        )?;
        let definition = self
            .game_data
            .relic_data
            .get_by_id(&item.definition_id)
            .ok_or(GameError::InvalidAction)?;
        let dust = definition.dismantle_equipment_dust;
        self.state.crafting_resources.grant_equipment_dust(dust)?;
        self.state.relics.remove(copy)?;
        self.handle_request_relic_state()
    }
}

impl GameCore {
    pub(super) fn reconcile_relic_loadouts(&mut self) -> Result<(), GameError> {
        if self.state.relics.is_empty() {
            return Ok(());
        }
        crate::authoring::control::charge(
            self.state.roster.len() as u64,
            "relic_death_loadout_scan",
        )?;
        for employee in self.state.roster.iter() {
            if employee.condition.is_dead() {
                self.state.relics.recover_employee(employee.uuid);
            }
        }
        self.state
            .relics
            .validate(&self.game_data.relic_data, |id| {
                self.state
                    .roster
                    .get(&id)
                    .is_some_and(|employee| employee.is_alive())
            })
    }
}
