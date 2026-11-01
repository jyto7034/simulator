use std::collections::BTreeSet;

use super::GameCore;
use crate::game::{
    behavior::GameError,
    employee::EmployeeRoster,
    resource::{
        store::{ResourceError, ResourceFaultCode, ResourceLimits, ResourceOwner},
        ResourceCatalog, ResourceOwnerKind,
    },
    resources::{GameState, Inventory},
};

pub(super) fn resource_owners(
    catalog: &ResourceCatalog,
    roster: &EmployeeRoster,
    inventory: &Inventory,
    fragments: &crate::game::skill_fragment::SkillFragmentInventory,
) -> Result<BTreeSet<ResourceOwner>, GameError> {
    crate::authoring::control::charge(catalog.resources.len() as u64, "resource_owner_kinds")?;
    let employees = catalog
        .resources
        .iter()
        .any(|def| def.owner == ResourceOwnerKind::Employee);
    let copies = catalog
        .resources
        .iter()
        .any(|def| def.owner == ResourceOwnerKind::EquipmentCopy);
    let fragment_owners = catalog
        .resources
        .iter()
        .any(|def| def.owner == ResourceOwnerKind::FragmentCopy);
    let count = (if employees { roster.len() } else { 0usize })
        .checked_add(if copies {
            inventory.equipments.len()
        } else {
            0
        })
        .and_then(|count| {
            count.checked_add(if fragment_owners {
                fragments.copies().count()
            } else {
                0
            })
        });
    if count.is_none_or(|n| n > ResourceLimits::default().owners) {
        return Err(ResourceError::from(ResourceFaultCode::Limit).into_game_error());
    }
    crate::authoring::control::charge(count.unwrap() as u64, "resource_actual_owner_graph")?;
    let mut owners = BTreeSet::new();
    if employees {
        for employee in roster.iter() {
            if employee.uuid.is_nil() {
                return Err(ResourceError::from(ResourceFaultCode::InvalidOwner).into_game_error());
            }
            owners.insert(ResourceOwner::Employee(employee.uuid));
        }
    }
    if copies {
        for equipment in inventory.equipments.iter() {
            if equipment.instance_uuid.is_nil() {
                return Err(ResourceError::from(ResourceFaultCode::InvalidOwner).into_game_error());
            }
            owners.insert(ResourceOwner::EquipmentCopy(equipment.instance_uuid));
        }
    }
    if fragment_owners {
        for copy in fragments.copies() {
            if copy.copy_uuid.is_nil() {
                return Err(ResourceError::from(ResourceFaultCode::InvalidOwner).into_game_error());
            }
            owners.insert(ResourceOwner::FragmentCopy(copy.copy_uuid));
        }
    }
    Ok(owners)
}

impl GameCore {
    pub(super) fn pending_resource_limits(
        &self,
    ) -> Option<crate::game::battle::runtime_control::RuntimeFactLimits> {
        if self
            .state
            .confirmed_combat_result
            .as_ref()
            .is_some_and(|receipt| receipt.runtime_facts.is_some())
        {
            return Some(self.game_data.runtime_safety().native_runtime());
        }
        self.state
            .active_node_content
            .as_ref()?
            .as_combat_battle()
            .ok()?
            .typed_resource_result
            .as_ref()
            .map(|result| result.limits)
    }

    pub(super) fn pending_resource_extent(&self) -> Result<Option<usize>, GameError> {
        if let Some(facts) = self
            .state
            .confirmed_combat_result
            .as_ref()
            .and_then(|receipt| receipt.runtime_facts.as_ref())
        {
            let run_extent = self
                .state
                .run
                .as_ref()
                .map(|run| run.typed_resources.retained_extent())
                .transpose()
                .map_err(ResourceError::into_game_error)?
                .unwrap_or(0);
            let extent = facts.retained_extent.checked_add(run_extent);
            crate::game::battle::runtime_control::limit(
                crate::game::battle::runtime_control::RuntimeFaultCode::RetainedByteLimit,
                "resource_confirmed_result_extent",
                self.game_data
                    .runtime_safety()
                    .native_runtime()
                    .max_retained_bytes,
                extent,
            )?;
            #[cfg(feature = "runtime-observation")]
            if let Some(bytes) = extent {
                crate::runtime_observation::pending_extent(bytes);
            }
            return Ok(extent);
        }
        let Some(crate::game::resources::ActiveNodeContent::CombatBattle(battle)) =
            &self.state.active_node_content
        else {
            return Ok(None);
        };
        let Some(result) = &battle.typed_resource_result else {
            return Ok(None);
        };
        let extent = result
            .retained_extent()
            .map_err(ResourceError::into_game_error)?
            .checked_add(
                battle
                    .runtime_facts
                    .as_ref()
                    .map_or(0, |facts| facts.retained_extent),
            );
        crate::game::battle::runtime_control::limit(
            crate::game::battle::runtime_control::RuntimeFaultCode::RetainedByteLimit,
            "resource_pending_result_extent",
            result.limits.max_retained_bytes,
            extent,
        )?;
        #[cfg(feature = "runtime-observation")]
        if let Some(bytes) = extent {
            crate::runtime_observation::pending_extent(bytes);
        }
        Ok(extent)
    }

    pub fn run_resources(&self) -> Option<&crate::game::resource::RunResourceState> {
        self.state.run.as_ref().map(|run| &run.typed_resources)
    }

    pub fn battle_typed_resources(&self) -> Option<crate::game::resource::ResourceView<'_>> {
        self.state
            .active_battle
            .as_ref()
            .and_then(|active| active.battle.typed_resources())
    }

    pub(super) fn validate_run_resources(&self) -> Result<(), GameError> {
        if let Some(run) = &self.state.run {
            run.credit_ledgers.validate(&self.state.skill_fragments)?;
            let owners = resource_owners(
                &self.game_data.resource_data,
                &self.state.roster,
                &self.state.inventory,
                &self.state.skill_fragments,
            )?;
            run.typed_resources
                .validate(&self.game_data.resource_data, &owners)
                .map_err(ResourceError::into_game_error)?;
        }
        Ok(())
    }

    pub(super) fn reconcile_run_resources(&mut self) -> Result<(), GameError> {
        let terminal = matches!(
            self.state.lifecycle.phase(),
            GameState::RunComplete | GameState::RunFailed { .. } | GameState::GameOver
        );
        if let Some(run) = self.state.run.as_mut() {
            run.credit_ledgers.reconcile(&self.state.skill_fragments,terminal)?;
            run.credit_ledgers.validate(&self.state.skill_fragments)?;
            let owners = resource_owners(
                &self.game_data.resource_data,
                &self.state.roster,
                &self.state.inventory,
                &self.state.skill_fragments,
            )?;
            run.typed_resources
                .reconcile_owners(&owners)
                .map_err(ResourceError::into_game_error)?;
            run.typed_resources
                .validate(&self.game_data.resource_data, &owners)
                .map_err(ResourceError::into_game_error)?;
            if terminal && !run.typed_resources.closed {
                run.typed_resources
                    .close()
                    .map_err(ResourceError::into_game_error)?;
            }
        }
        Ok(())
    }
}
