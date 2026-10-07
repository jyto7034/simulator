//! Strict Run projection. Only content keys, persistent owners and values cross
//! a save boundary. Runtime scopes, reservation tokens and pointers never do.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{
    store::{ResourceError, ResourceFaultCode, ResourceOwner, ResourceScope},
    ResourceCatalog, ResourceId, ResourceLifetime,
};

pub const MAX_RUN_RESOURCE_ENTRIES: usize = 65536;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunResourceEntry {
    pub resource: ResourceId,
    pub owner: ResourceOwner,
    pub balance: u32,
    pub revision: u64,
}

/// Default constructs a new Run only. Serde still requires all three fields;
/// missing old-save fields cannot turn into an empty/zero resource timeline.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunResourceState {
    pub revision: u64,
    pub closed: bool,
    pub entries: Vec<RunResourceEntry>,
}

impl RunResourceState {
    pub(crate) fn retained_extent(&self) -> Result<usize, ResourceError> {
        crate::authoring::control::charge(self.entries.len() as u64, "resource_durable_extent")?;
        let mut bytes = std::mem::size_of::<Self>()
            .checked_add(
                self.entries
                    .capacity()
                    .checked_mul(std::mem::size_of::<RunResourceEntry>())
                    .ok_or(ResourceFaultCode::Limit)?,
            )
            .ok_or(ResourceFaultCode::Limit)?;
        for entry in &self.entries {
            bytes = bytes
                .checked_add(entry.resource.0.capacity())
                .ok_or(ResourceFaultCode::Limit)?;
        }
        Ok(bytes)
    }

    pub fn validate(
        &self,
        catalog: &ResourceCatalog,
        owners: &BTreeSet<ResourceOwner>,
    ) -> Result<(), ResourceError> {
        if self.entries.len() > MAX_RUN_RESOURCE_ENTRIES {
            return Err(ResourceFaultCode::Limit.into());
        }
        catalog.validate().map_err(ResourceError::Definition)?;
        crate::authoring::control::charge(self.entries.len() as u64, "resource_run_validation")?;
        let mut previous = None;
        for entry in &self.entries {
            let key = (&entry.resource, entry.owner);
            if previous.is_some_and(|old| old >= key) {
                return Err(ResourceFaultCode::Invariant.into());
            }
            previous = Some(key);
            if !owners.contains(&entry.owner) {
                return Err(ResourceFaultCode::InvalidOwner.into());
            }
            crate::authoring::control::charge(
                catalog.resources.len() as u64,
                "resource_run_definition",
            )?;
            let definition = catalog
                .resources
                .iter()
                .find(|def| def.id == entry.resource)
                .ok_or(ResourceFaultCode::UnknownDefinition)?;
            if definition.owner != entry.owner.kind() {
                return Err(ResourceFaultCode::OwnerKindMismatch.into());
            }
            if definition.lifetime != ResourceLifetime::Run {
                return Err(ResourceFaultCode::WrongLifetime.into());
            }
            if entry.balance > definition.cap {
                return Err(ResourceFaultCode::Invariant.into());
            }
        }
        Ok(())
    }

    /// Final application owner-graph reconciliation. Equip, transfer and
    /// enhancement retain a UUID; sale/dismantle removes only that UUID's state.
    pub fn reconcile_owners(
        &mut self,
        owners: &BTreeSet<ResourceOwner>,
    ) -> Result<(), ResourceError> {
        if self.entries.len() > MAX_RUN_RESOURCE_ENTRIES {
            return Err(ResourceFaultCode::Limit.into());
        }
        crate::authoring::control::charge(
            self.entries.len() as u64,
            "resource_run_owner_reconcile",
        )?;
        if self
            .entries
            .iter()
            .all(|entry| owners.contains(&entry.owner))
        {
            return Ok(());
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(ResourceFaultCode::CounterOverflow)?;
        self.entries.retain(|entry| owners.contains(&entry.owner));
        self.revision = revision;
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), ResourceError> {
        if self.closed {
            return Err(ResourceFaultCode::StaleScope.into());
        }
        let revision = self
            .revision
            .checked_add(1)
            .ok_or(ResourceFaultCode::CounterOverflow)?;
        self.closed = true;
        self.revision = revision;
        Ok(())
    }
}

/// Battle-local binding; never part of the durable writer's type graph.
#[derive(Debug, Clone)]
pub struct RunResourceOverlay {
    pub(super) basis: RunResourceState,
    pub scope: ResourceScope,
}

/// Pending terminal payload. Full basis equality rejects different values with
/// the same numeric revision; every accepted result advances that revision.
/// The world's phase/outcome gate additionally binds the actual battle identity.
#[derive(Debug, Clone)]
pub struct RunResourceResult {
    pub(super) before: RunResourceState,
    pub(super) after: RunResourceState,
    pub(crate) limits: crate::game::battle::runtime_control::RuntimeFactLimits,
}

impl RunResourceResult {
    pub(crate) fn retained_extent(&self) -> Result<usize, ResourceError> {
        let before = self.before.retained_extent()?;
        let after = self.after.retained_extent()?;
        std::mem::size_of::<Self>()
            .checked_add(before)
            .and_then(|n| n.checked_add(after))
            .ok_or_else(|| ResourceFaultCode::Limit.into())
    }

    pub(crate) fn preflight_native_clone(&self) -> Result<(), crate::game::behavior::GameError> {
        super::store::preflight_resource_clone(self).map_err(ResourceError::into_game_error)
    }

    pub fn apply(&self, state: &mut RunResourceState) -> Result<(), ResourceError> {
        crate::authoring::control::charge(
            (state.entries.len() as u64).saturating_add(self.before.entries.len() as u64),
            "resource_result_basis_compare",
        )?;
        if state.closed || *state != self.before {
            return Err(ResourceFaultCode::StaleRevision.into());
        }
        let replacement = super::store::clone_resource_value(&self.after)?;
        *state = replacement;
        Ok(())
    }
}

crate::authoring::clone_shape::struct_shape!(RunResourceEntry {
    resource,
    owner,
    balance,
    revision
});
crate::authoring::clone_shape::struct_shape!(RunResourceState {
    revision,
    closed,
    entries
});
crate::authoring::clone_shape::struct_shape!(RunResourceOverlay { basis, scope });
crate::authoring::clone_shape::struct_shape!(RunResourceResult {
    before,
    after,
    limits
});
