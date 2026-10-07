//! The authored resource consumer resolves actual ownership and occurrence
//! capabilities before entering the atomic store batch.

use std::sync::Arc;

use super::BattleCore;
use crate::game::{
    battle::{
        ids::UnitInstanceId,
        runtime_facts::{CostReason, FactResource, ResourceOwner as FactOwner},
    },
    behavior::GameError,
    resource::{
        store::{ResourceError, ResourceFaultCode, ResourceLease, ResourceLimits, ResourceOwner},
        ResourceId, ResourceLifetime, ResourceMutation, ResourceOperation, ResourceOwnerSelector,
    },
};

fn fault(code: ResourceFaultCode) -> GameError {
    ResourceError::from(code).into_game_error()
}

impl BattleCore {
    fn typed_resource_definition(
        &self,
        resource: &ResourceId,
    ) -> Result<&crate::game::resource::ResourceDefinition, ResourceError> {
        crate::authoring::control::charge(
            self.game_data.resource_data.resources.len() as u64,
            "resource_consumer_definition",
        )?;
        self.game_data
            .resource_data
            .resources
            .iter()
            .find(|definition| definition.id == *resource)
            .ok_or_else(|| ResourceFaultCode::UnknownDefinition.into())
    }

    /// Resolve the same definition/owner/lifetime capability for reads and
    /// mutations. This function never prepares resources, opens a scope,
    /// retains a delivery, reserves a cost, or mutates the store.
    pub(super) fn resolve_typed_resource_lease(
        &self,
        resource: &ResourceId,
        selector: ResourceOwnerSelector,
        source: &crate::game::battle::runtime_facts::RuntimeFactSource,
        target: Option<UnitInstanceId>,
    ) -> Result<ResourceLease, ResourceError> {
        let definition = self.typed_resource_definition(resource)?;
        self.resolve_typed_resource_lease_with_definition(definition, selector, source, target)
    }

    fn resolve_typed_resource_lease_with_definition(
        &self,
        definition: &crate::game::resource::ResourceDefinition,
        selector: ResourceOwnerSelector,
        source: &crate::game::battle::runtime_facts::RuntimeFactSource,
        target: Option<UnitInstanceId>,
    ) -> Result<ResourceLease, ResourceError> {
        let actor = match selector {
            ResourceOwnerSelector::TargetEmployee => target,
            ResourceOwnerSelector::SourceEmployee
            | ResourceOwnerSelector::SourceEquipment
            | ResourceOwnerSelector::SourceFragment => source.actor,
        }
        .ok_or(ResourceFaultCode::InvalidOwner)?;
        let unit = self
            .units
            .get(&actor)
            .ok_or(ResourceFaultCode::InvalidOwner)?;
        let owner = match selector {
            ResourceOwnerSelector::SourceEmployee | ResourceOwnerSelector::TargetEmployee => {
                if unit.source_owned_uuid.is_nil() {
                    return Err(ResourceFaultCode::InvalidOwner.into());
                }
                ResourceOwner::Employee(unit.source_owned_uuid)
            }
            ResourceOwnerSelector::SourceFragment => {
                let source_fragment = source
                    .fragment
                    .as_ref()
                    .ok_or(ResourceFaultCode::InvalidOwner)?;
                if source_fragment.copy_uuid.is_nil()
                    || unit
                        .fragment_source
                        .as_ref()
                        .is_none_or(|bound| !std::sync::Arc::ptr_eq(bound, source_fragment))
                {
                    return Err(ResourceFaultCode::InvalidOwner.into());
                }
                ResourceOwner::FragmentCopy(source_fragment.copy_uuid)
            }
            ResourceOwnerSelector::SourceEquipment => {
                let id = source
                    .equipment
                    .and_then(|item| item.persistent_copy_uuid)
                    .filter(|id| !id.is_nil())
                    .ok_or(ResourceFaultCode::InvalidOwner)?;
                ResourceOwner::EquipmentCopy(id)
            }
        };
        let runtime = self
            .runtime_facts
            .resources
            .as_ref()
            .ok_or(ResourceFaultCode::UnknownDefinition)?;
        let context = self.resource_context_for_source(source);
        let scope = match definition.lifetime {
            ResourceLifetime::Run => runtime.run.as_ref().map(|run| &run.scope),
            ResourceLifetime::Attempt => Some(&runtime.attempt),
            ResourceLifetime::Deployment => runtime.deployments.get(&actor),
            ResourceLifetime::Activation => context
                .as_ref()
                .and_then(|context| context.activation.as_ref()),
            ResourceLifetime::Chain => context.as_ref().map(|context| &context.chain),
            ResourceLifetime::Delivery => source
                .resource_delivery()
                .or_else(|| {
                    source
                        .delivery
                        .as_ref()
                        .and_then(|ticket| ticket.resource.as_ref())
                })
                .map(|delivery| &delivery.scope),
        }
        .ok_or(ResourceFaultCode::WrongLifetime)?;
        runtime.store.bind(&definition.id, owner, scope)
    }

    /// An immediate step has a real synchronous delivery occurrence. Released
    /// carriers own their separate delivery until projectile/area completion.
    pub(super) fn with_instant_resource_delivery<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, GameError>,
    ) -> Result<T, GameError> {
        let context = self.resource_context_for_source(&self.runtime_fact_source);
        let completion = match self.runtime_facts.resources.as_mut() {
            Some(resources) => Some(resources.finite(context.as_deref())?),
            None => None,
        };
        let prior = self.runtime_fact_source.clone();
        if let Some((context, delivery)) = &completion {
            self.runtime_fact_source.resource_capabilities =
                crate::game::battle::runtime_facts::RuntimeFactSource::resource_capabilities(
                    Some(context.clone()),
                    Some(delivery.clone()),
                )?;
        }
        let operation_result = operation(self);
        self.runtime_fact_source = prior;
        let poll_result = crate::authoring::control::poll("resource_instant_completion");
        let close_result = completion
            .as_ref()
            .map(|(_, delivery)| self.close_source_resource_delivery(Some(delivery)))
            .transpose();
        let value = operation_result?;
        poll_result?;
        close_result?;
        Ok(value)
    }

    pub fn typed_resources(&self) -> Option<crate::game::resource::ResourceView<'_>> {
        self.runtime_facts
            .resources
            .as_ref()
            .map(|runtime| runtime.store.view())
    }

    pub(super) fn apply_typed_resource_operations(
        &mut self,
        operations: &[ResourceOperation],
        targets: &[UnitInstanceId],
        time_ms: u64,
    ) -> Result<(), GameError> {
        let limit = ResourceLimits::default().batch;
        if operations.is_empty() || operations.len() > limit {
            return Err(fault(ResourceFaultCode::Limit));
        }
        crate::authoring::control::charge(operations.len() as u64, "resource_expand_count")?;
        let count = operations.iter().try_fold(0usize, |count, operation| {
            let width = if operation.owner == ResourceOwnerSelector::TargetEmployee {
                if targets.is_empty() {
                    return Err(fault(ResourceFaultCode::InvalidOwner));
                }
                targets.len()
            } else {
                1
            };
            count
                .checked_add(width)
                .filter(|n| *n <= limit)
                .ok_or_else(|| fault(ResourceFaultCode::Limit))
        })?;
        crate::authoring::control::charge(count as u64, "resource_expand_batch")?;
        self.prepare_runtime_resources()?;
        let source = self.runtime_fact_source.clone();
        let mut expanded = Vec::with_capacity(count);
        for operation in operations {
            let definition = self
                .typed_resource_definition(&operation.resource)
                .map_err(ResourceError::into_game_error)?;
            let source_actor = [source.actor];
            let target_actors;
            let actors = if operation.owner == ResourceOwnerSelector::TargetEmployee {
                target_actors = targets.iter().copied().map(Some).collect::<Vec<_>>();
                target_actors.as_slice()
            } else {
                &source_actor
            };
            for actor in actors {
                let lease = self
                    .resolve_typed_resource_lease_with_definition(
                        definition,
                        operation.owner,
                        &source,
                        *actor,
                    )
                    .map_err(ResourceError::into_game_error)?;
                crate::authoring::control::charge(
                    operation.resource.as_str().len() as u64,
                    "resource_batch_key",
                )?;
                expanded.push((lease, operation.operation));
            }
        }
        let receipts = self
            .runtime_facts
            .resources
            .as_mut()
            .expect("prepared resources")
            .store
            .apply_leased_operations(&expanded)
            .map_err(ResourceError::into_game_error)?;
        for (receipt, (lease, _)) in receipts.into_iter().zip(&expanded) {
            let owner = match receipt.key.owner {
                ResourceOwner::Employee(id) => FactOwner::Employee(id),
                ResourceOwner::EquipmentCopy(id) => FactOwner::EquipmentCopy(id),
                ResourceOwner::FragmentCopy(id) => FactOwner::FragmentCopy(id),
            };
            let (delta, payment) = match receipt.operation {
                ResourceMutation::Grant(amount) => (i64::from(amount), None),
                ResourceMutation::Decay(amount) => (-i64::from(amount), None),
                ResourceMutation::Consume(amount) => {
                    (-i64::from(amount), Some(CostReason::AuthoredResource))
                }
            };
            self.record_resource_fact(
                owner,
                FactResource::Typed {
                    id: Arc::new(receipt.key.resource),
                    lifetime: lease.scope().lifetime(),
                    scope: lease.scope().ordinal(),
                },
                payment,
                source.clone(),
                delta,
                receipt.before.balance,
                receipt.after.balance,
                time_ms,
            )?;
        }
        Ok(())
    }
}
