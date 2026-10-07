//! Candidate-owned resource authority. Handles are clone-relative capabilities;
//! diagnostic ordinals and saved values cannot manufacture them.

use std::{collections::BTreeMap, collections::BTreeSet, sync::Arc};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    ResourceCatalog, ResourceDefinitionError, ResourceId, ResourceLifetime, ResourceOwnerKind,
};
use crate::{
    authoring::clone_shape::controlled_clone,
    game::{battle::runtime_control, behavior::GameError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ResourceOwner {
    Employee(Uuid),
    EquipmentCopy(Uuid),
    FragmentCopy(Uuid),
}

impl ResourceOwner {
    pub fn kind(self) -> ResourceOwnerKind {
        match self {
            Self::Employee(_) => ResourceOwnerKind::Employee,
            Self::EquipmentCopy(_) => ResourceOwnerKind::EquipmentCopy,
            Self::FragmentCopy(_) => ResourceOwnerKind::FragmentCopy,
        }
    }

    fn is_nil(self) -> bool {
        match self {
            Self::Employee(id) | Self::EquipmentCopy(id) | Self::FragmentCopy(id) => id.is_nil(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ResourceFaultCode {
    UnknownDefinition,
    InvalidOwner,
    OwnerKindMismatch,
    StaleScope,
    WrongLifetime,
    StaleLease,
    StaleRevision,
    StaleReservation,
    Insufficient,
    ReservationConflict,
    Limit,
    CounterOverflow,
    Invariant,
}

impl ResourceFaultCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownDefinition => "resource_unknown_definition",
            Self::InvalidOwner => "resource_invalid_owner",
            Self::OwnerKindMismatch => "resource_owner_kind_mismatch",
            Self::StaleScope => "resource_stale_scope",
            Self::WrongLifetime => "resource_wrong_lifetime",
            Self::StaleLease => "resource_stale_lease",
            Self::StaleRevision => "resource_stale_revision",
            Self::StaleReservation => "resource_stale_reservation",
            Self::Insufficient => "resource_insufficient",
            Self::ReservationConflict => "resource_reservation_conflict",
            Self::Limit => "resource_limit",
            Self::CounterOverflow => "resource_counter_overflow",
            Self::Invariant => "resource_invariant",
        }
    }
}

#[derive(Debug)]
pub enum ResourceError {
    Definition(ResourceDefinitionError),
    Fault(ResourceFaultCode),
    Control(GameError),
}

impl From<ResourceFaultCode> for ResourceError {
    fn from(value: ResourceFaultCode) -> Self {
        Self::Fault(value)
    }
}
impl From<GameError> for ResourceError {
    fn from(value: GameError) -> Self {
        Self::Control(value)
    }
}

impl ResourceError {
    pub fn into_game_error(self) -> GameError {
        match self {
            ResourceError::Control(error) => error,
            ResourceError::Definition(error) => GameError::InvalidStaticData(error.to_string()),
            ResourceError::Fault(code) => {
                runtime_control::reject(runtime_control::BattleRuntimeFault {
                    code: runtime_control::RuntimeFaultCode::Resource(code),
                    site: "typed_resource",
                    limit: None,
                    requested: None,
                })
            }
        }
    }
}

type ResourceResult<T> = std::result::Result<T, ResourceError>;

pub(crate) fn preflight_resource_clone<T: crate::authoring::clone_shape::CloneShape>(
    value: &T,
) -> ResourceResult<()> {
    // The general helper meters authoring-controlled clones. Resource operations
    // also run directly under the native battle budget, where their own graph
    // must be admitted before copying, including reservation vectors.
    if runtime_control::is_active() {
        let mut meter = crate::authoring::control::CloneMeter::for_native_scan()?;
        value.visit_clone_shape(&mut meter)?;
        crate::authoring::control::charge(meter.native_scan_slots() as u64, "resource_clone_scan")?;
        let slots = meter.native_copy_slots().ok_or(ResourceFaultCode::Limit)?;
        runtime_control::charge_clone(slots, "resource_clone_copy")?;
    }
    Ok(())
}

pub(crate) fn clone_resource_value<T: Clone + crate::authoring::clone_shape::CloneShape>(
    value: &T,
) -> ResourceResult<T> {
    preflight_resource_clone(value)?;
    Ok(controlled_clone(value)?)
}

/// Borrowed description of the owned terminal projection. Price the destination
/// vector and copied IDs before allocating either of them.
struct RunProjectionShape<'a> {
    entries: &'a BTreeMap<ResourceKey, ResourceValue>,
    scope: u64,
    count: usize,
}

impl crate::authoring::clone_shape::CloneShape for RunProjectionShape<'_> {
    fn visit_clone_shape(
        &self,
        meter: &mut crate::authoring::control::CloneMeter,
    ) -> Result<(), GameError> {
        // RunResourceState and its revision/closed fields, then Vec storage.
        meter.scan_storage(self.count.checked_add(4))?;
        for (key, value) in self.entries {
            if key.scope != self.scope {
                continue;
            }
            meter.scan_storage(Some(1))?;
            key.resource.visit_clone_shape(meter)?;
            key.owner.visit_clone_shape(meter)?;
            value.balance.visit_clone_shape(meter)?;
            value.revision.visit_clone_shape(meter)?;
        }
        Ok(())
    }
}

impl RunProjectionShape<'_> {
    fn admit(&self) -> ResourceResult<()> {
        use crate::authoring::clone_shape::CloneShape;
        if runtime_control::is_active() {
            let mut meter = crate::authoring::control::CloneMeter::for_native_scan()?;
            self.visit_clone_shape(&mut meter)?;
            crate::authoring::control::charge(
                meter.native_scan_slots() as u64,
                "resource_projection_scan",
            )?;
            runtime_control::charge_clone(
                meter.native_copy_slots().ok_or(ResourceFaultCode::Limit)?,
                "resource_projection_copy",
            )?;
        }
        if crate::authoring::control::is_active() {
            let mut meter = crate::authoring::control::CloneMeter::default();
            self.visit_clone_shape(&mut meter)?;
            meter.finish("resource_projection_storage", "resource_projection_copy")?;
        }
        Ok(())
    }
}

/// Engineering limits, separate from each authored gameplay cap.
#[derive(Debug, Clone, Copy)]
pub struct ResourceLimits {
    pub owners: usize,
    pub scopes: usize,
    pub entries: usize,
    pub reservations: usize,
    pub holds: usize,
    pub batch: usize,
    pub retained_bytes: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            owners: 16384,
            scopes: 16384,
            entries: 65536,
            reservations: 8192,
            holds: 32768,
            batch: 4096,
            retained_bytes: 8 * 1024 * 1024,
        }
    }
}

#[derive(Debug)]
struct OwnerToken;
#[derive(Debug)]
struct ScopeToken {
    ordinal: u64,
    lifetime: ResourceLifetime,
}
#[derive(Debug)]
struct HoldToken;
#[derive(Debug)]
struct ReservationToken;

#[derive(Debug, Clone)]
pub struct ResourceScope {
    token: Arc<ScopeToken>,
}

impl PartialEq for ResourceScope {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.token, &other.token)
    }
}
impl Eq for ResourceScope {}

impl ResourceScope {
    pub fn lifetime(&self) -> ResourceLifetime {
        self.token.lifetime
    }
    pub fn ordinal(&self) -> u64 {
        self.token.ordinal
    }
}

#[derive(Debug, Clone)]
pub struct ScopeHold {
    scope: ResourceScope,
    ordinal: u64,
    token: Arc<HoldToken>,
}

impl PartialEq for ScopeHold {
    fn eq(&self, other: &Self) -> bool {
        self.scope == other.scope
            && self.ordinal == other.ordinal
            && Arc::ptr_eq(&self.token, &other.token)
    }
}
impl Eq for ScopeHold {}

#[derive(Debug, Clone)]
pub struct ResourceLease {
    key: ResourceKey,
    scope: ResourceScope,
    owner: Arc<OwnerToken>,
}

impl ResourceLease {
    pub(crate) fn scope(&self) -> &ResourceScope {
        &self.scope
    }
}

#[derive(Debug, Clone)]
pub struct ResourceReservation {
    ordinal: u64,
    issued_revision: u64,
    token: Arc<ReservationToken>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct ResourceKey {
    pub resource: ResourceId,
    pub owner: ResourceOwner,
    pub scope: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ResourceValue {
    pub balance: u32,
    pub reserved: u32,
    pub revision: u64,
}

impl ResourceValue {
    pub fn available(self) -> u32 {
        self.balance - self.reserved
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceMutation {
    Grant(u32),
    Decay(u32),
    Consume(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResourceReceipt {
    pub key: ResourceKey,
    pub operation: ResourceMutation,
    pub before: ResourceValue,
    pub after: ResourceValue,
    pub applied: u32,
}

#[derive(Debug, Clone)]
struct ScopeRecord {
    scope: ResourceScope,
    root_live: bool,
    holds: BTreeMap<u64, Arc<HoldToken>>,
    next_hold: u64,
}

#[derive(Debug, Clone)]
struct ReservationRecord {
    handle: ResourceReservation,
    costs: Vec<(ResourceLease, u32)>,
}

#[derive(Debug, Clone)]
pub struct ResourceStore {
    catalog: Arc<ResourceCatalog>,
    owners: BTreeMap<ResourceOwner, Arc<OwnerToken>>,
    scopes: BTreeMap<u64, ScopeRecord>,
    entries: BTreeMap<ResourceKey, ResourceValue>,
    reservations: BTreeMap<u64, ReservationRecord>,
    next_scope: u64,
    next_reservation: u64,
    revision: u64,
    limits: ResourceLimits,
}

/// Borrowed diagnostics only. Scope ordinals cannot create spending authority.
/// Missing entries use the explicit catalog initial value for that live scope.
#[derive(Debug, Clone, Copy)]
pub struct ResourceView<'a>(&'a ResourceStore);

impl<'a> ResourceView<'a> {
    pub fn catalog(self) -> &'a ResourceCatalog {
        &self.0.catalog
    }
    pub fn revision(self) -> u64 {
        self.0.revision()
    }
    /// Read sparse initial values without materialization or lease creation.
    pub fn value(
        self,
        id: &ResourceId,
        owner: ResourceOwner,
        ordinal: u64,
    ) -> Option<ResourceValue> {
        let scope = self.0.scopes.get(&ordinal)?;
        let definition = self
            .0
            .catalog
            .resources
            .iter()
            .find(|definition| definition.id == *id)?;
        if !self.0.owners.contains_key(&owner)
            || definition.owner != owner.kind()
            || definition.lifetime != scope.scope.lifetime()
        {
            return None;
        }
        self.0
            .entries
            .iter()
            .find(|(key, _)| key.scope == ordinal && key.owner == owner && key.resource == *id)
            .map(|(_, value)| *value)
            .or(Some(ResourceValue {
                balance: definition.initial,
                reserved: 0,
                revision: 0,
            }))
    }
    pub fn entries(self) -> impl Iterator<Item = (&'a ResourceKey, &'a ResourceValue)> {
        self.0.entries.iter()
    }
    pub fn owners(self) -> impl Iterator<Item = ResourceOwner> + 'a {
        self.0.owners.keys().copied()
    }
    pub fn scopes(self) -> impl Iterator<Item = (u64, ResourceLifetime, bool, usize)> + 'a {
        self.0.scopes.values().map(|record| {
            (
                record.scope.ordinal(),
                record.scope.lifetime(),
                record.root_live,
                record.holds.len(),
            )
        })
    }
    pub fn reservation_count(self) -> usize {
        self.0.reservations.len()
    }
}

impl Serialize for ResourceView<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        struct Sequence<I>(I);
        impl<I: Iterator + Clone> Serialize for Sequence<I>
        where
            I::Item: Serialize,
        {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.collect_seq(self.0.clone())
            }
        }
        let mut state = serializer.serialize_struct("ResourceView", 6)?;
        state.serialize_field("catalog", self.catalog())?;
        state.serialize_field("revision", &self.0.revision)?;
        state.serialize_field("owners", &Sequence(self.0.owners.keys()))?;
        state.serialize_field(
            "scopes",
            &Sequence(self.0.scopes.values().map(|record| {
                (
                    record.scope.ordinal(),
                    record.scope.lifetime(),
                    record.root_live,
                    record.holds.len(),
                )
            })),
        )?;
        state.serialize_field("entries", &Sequence(self.0.entries.iter()))?;
        state.serialize_field("reservation_count", &self.reservation_count())?;
        state.end()
    }
}

fn next(value: u64) -> ResourceResult<u64> {
    value
        .checked_add(1)
        .ok_or_else(|| ResourceFaultCode::CounterOverflow.into())
}

impl ResourceStore {
    pub fn view(&self) -> ResourceView<'_> {
        ResourceView(self)
    }
    pub fn new(catalog: Arc<ResourceCatalog>) -> ResourceResult<Self> {
        catalog.validate().map_err(ResourceError::Definition)?;
        Ok(Self {
            catalog,
            owners: BTreeMap::new(),
            scopes: BTreeMap::new(),
            entries: BTreeMap::new(),
            reservations: BTreeMap::new(),
            next_scope: 0,
            next_reservation: 0,
            revision: 0,
            limits: ResourceLimits::default(),
        })
    }

    /// Used inside the enclosing world/battle candidate. A failed store edit
    /// publishes neither IDs nor revisions; operation fuel remains consumed.
    pub fn transaction<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> ResourceResult<T>,
    ) -> ResourceResult<T> {
        self.check_extent()?;
        let mut candidate = clone_resource_value(self)?;
        let result = operation(&mut candidate)?;
        candidate.check_extent()?;
        runtime_control::poll()?;
        *self = candidate;
        Ok(result)
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn mount_run(
        &mut self,
        state: &super::durable::RunResourceState,
    ) -> ResourceResult<super::durable::RunResourceOverlay> {
        self.transaction(|store| {
            if state.closed
                || store
                    .scopes
                    .values()
                    .any(|record| record.scope.lifetime() == ResourceLifetime::Run)
            {
                return Err(ResourceFaultCode::StaleScope.into());
            }
            crate::authoring::control::charge(
                store.owners.len() as u64,
                "resource_mount_owner_graph",
            )?;
            let owners = store.owners.keys().copied().collect();
            state.validate(&store.catalog, &owners)?;
            let basis = clone_resource_value(state)?;
            let scope = store.open_scope(ResourceLifetime::Run)?;
            for entry in &state.entries {
                store.entries.insert(
                    ResourceKey {
                        resource: entry.resource.clone(),
                        owner: entry.owner,
                        scope: scope.ordinal(),
                    },
                    ResourceValue {
                        balance: entry.balance,
                        reserved: 0,
                        revision: entry.revision,
                    },
                );
            }
            Ok(super::durable::RunResourceOverlay { basis, scope })
        })
    }

    pub fn capture_run(
        &self,
        overlay: &super::durable::RunResourceOverlay,
    ) -> ResourceResult<super::durable::RunResourceResult> {
        self.scope_record(&overlay.scope)?;
        if overlay.scope.lifetime() != ResourceLifetime::Run {
            return Err(ResourceFaultCode::WrongLifetime.into());
        }
        self.check_extent()?;
        crate::authoring::control::charge(self.entries.len() as u64, "resource_run_projection")?;
        let count = self
            .entries
            .keys()
            .filter(|key| key.scope == overlay.scope.ordinal())
            .count();
        crate::authoring::control::charge(
            self.entries.len() as u64,
            "resource_run_projection_shape",
        )?;
        RunProjectionShape {
            entries: &self.entries,
            scope: overlay.scope.ordinal(),
            count,
        }
        .admit()?;
        // The pending result owns two new vectors while the live store still
        // exists. Admit that allocation before constructing either projection.
        let mut extent = std::mem::size_of::<super::durable::RunResourceResult>()
            .checked_add(overlay.basis.retained_extent()?)
            .and_then(|n| n.checked_add(std::mem::size_of::<super::durable::RunResourceState>()))
            .and_then(|n| {
                n.checked_add(
                    count.checked_mul(std::mem::size_of::<super::durable::RunResourceEntry>())?,
                )
            })
            .ok_or(ResourceFaultCode::Limit)?;
        crate::authoring::control::charge(self.entries.len() as u64, "resource_result_extent")?;
        for key in self
            .entries
            .keys()
            .filter(|key| key.scope == overlay.scope.ordinal())
        {
            extent = extent
                .checked_add(key.resource.as_str().len())
                .ok_or(ResourceFaultCode::Limit)?;
        }
        runtime_control::admit_retained_growth(0, Some(extent), "resource_result_allocate")?;
        let mut entries = Vec::with_capacity(count);
        for (key, value) in &self.entries {
            if key.scope != overlay.scope.ordinal() {
                continue;
            }
            if value.reserved != 0 {
                return Err(ResourceFaultCode::ReservationConflict.into());
            }
            entries.push(super::durable::RunResourceEntry {
                resource: key.resource.clone(),
                owner: key.owner,
                balance: value.balance,
                revision: value.revision,
            });
        }
        let after = super::durable::RunResourceState {
            revision: next(overlay.basis.revision)?,
            closed: false,
            entries,
        };
        Ok(super::durable::RunResourceResult {
            before: clone_resource_value(&overlay.basis)?,
            after,
            limits: Default::default(),
        })
    }

    /// The adapter supplies the final inventory/roster graph, never a partial
    /// starter grant. Retained UUIDs retain authority through equip/transfer.
    pub fn reconcile_owners(&mut self, owners: &BTreeSet<ResourceOwner>) -> ResourceResult<()> {
        self.transaction(|store| {
            crate::authoring::control::charge(owners.len() as u64, "resource_owner_graph")?;
            if owners.len() > store.limits.owners {
                return Err(ResourceFaultCode::Limit.into());
            }
            if owners.iter().any(|owner| owner.is_nil()) {
                return Err(ResourceFaultCode::InvalidOwner.into());
            }
            if store.owners.keys().copied().eq(owners.iter().copied()) {
                return Ok(());
            }
            let revision = next(store.revision)?;
            // Abort each affected multi-cost reservation, including locks on its
            // surviving owners. Deleting one copy must not strand another lock.
            let removed: Vec<_> = store
                .reservations
                .iter()
                .filter(|(_, record)| {
                    record
                        .costs
                        .iter()
                        .any(|(lease, _)| !owners.contains(&lease.key.owner))
                })
                .map(|(id, _)| *id)
                .collect();
            for id in removed {
                store.abort_record(id)?;
            }
            store.entries.retain(|key, _| owners.contains(&key.owner));
            store.owners.retain(|owner, _| owners.contains(owner));
            for owner in owners {
                store
                    .owners
                    .entry(*owner)
                    .or_insert_with(|| Arc::new(OwnerToken));
            }
            store.revision = revision;
            Ok(())
        })
    }

    pub fn open_scope(&mut self, lifetime: ResourceLifetime) -> ResourceResult<ResourceScope> {
        self.transaction(|store| {
            if store.scopes.len() >= store.limits.scopes {
                return Err(ResourceFaultCode::Limit.into());
            }
            let ordinal = next(store.next_scope)?;
            let revision = next(store.revision)?;
            let scope = ResourceScope {
                token: Arc::new(ScopeToken { ordinal, lifetime }),
            };
            store.scopes.insert(
                ordinal,
                ScopeRecord {
                    scope: scope.clone(),
                    root_live: true,
                    holds: BTreeMap::new(),
                    next_hold: 0,
                },
            );
            store.next_scope = ordinal;
            store.revision = revision;
            Ok(scope)
        })
    }

    fn scope_record(&self, scope: &ResourceScope) -> ResourceResult<&ScopeRecord> {
        self.scopes
            .get(&scope.ordinal())
            .filter(|record| Arc::ptr_eq(&record.scope.token, &scope.token))
            .ok_or_else(|| ResourceFaultCode::StaleScope.into())
    }

    pub fn retain_chain(&mut self, scope: &ResourceScope) -> ResourceResult<ScopeHold> {
        self.retain_scope(scope, ResourceLifetime::Chain)
    }

    pub fn retain_delivery(&mut self, scope: &ResourceScope) -> ResourceResult<ScopeHold> {
        self.retain_scope(scope, ResourceLifetime::Delivery)
    }

    fn retain_scope(
        &mut self,
        scope: &ResourceScope,
        lifetime: ResourceLifetime,
    ) -> ResourceResult<ScopeHold> {
        self.transaction(|store| {
            let record = store.scope_record(scope)?;
            if scope.lifetime() != lifetime {
                return Err(ResourceFaultCode::WrongLifetime.into());
            }
            let ordinal = next(record.next_hold)?;
            let revision = next(store.revision)?;
            let hold = ScopeHold {
                scope: scope.clone(),
                ordinal,
                token: Arc::new(HoldToken),
            };
            let record = store.scopes.get_mut(&scope.ordinal()).unwrap();
            record.holds.insert(ordinal, hold.token.clone());
            record.next_hold = ordinal;
            store.revision = revision;
            Ok(hold)
        })
    }

    /// Root completion preserves already-admitted finite children/completions.
    pub fn finish_scope(&mut self, scope: &ResourceScope) -> ResourceResult<()> {
        self.transaction(|store| {
            let record = store.scope_record(scope)?;
            if !record.root_live {
                return Err(ResourceFaultCode::StaleScope.into());
            }
            if record.holds.is_empty() {
                return store.close_scope_inner(scope);
            }
            let revision = next(store.revision)?;
            store.scopes.get_mut(&scope.ordinal()).unwrap().root_live = false;
            store.revision = revision;
            Ok(())
        })
    }

    pub fn release_chain(&mut self, hold: &ScopeHold) -> ResourceResult<()> {
        self.release_hold(hold, ResourceLifetime::Chain)
    }

    pub fn release_delivery(&mut self, hold: &ScopeHold) -> ResourceResult<()> {
        self.release_hold(hold, ResourceLifetime::Delivery)
    }

    fn release_hold(&mut self, hold: &ScopeHold, lifetime: ResourceLifetime) -> ResourceResult<()> {
        self.transaction(|store| {
            if hold.scope.lifetime() != lifetime {
                return Err(ResourceFaultCode::WrongLifetime.into());
            }
            let record = store.scope_record(&hold.scope)?;
            if !record
                .holds
                .get(&hold.ordinal)
                .is_some_and(|token| Arc::ptr_eq(token, &hold.token))
            {
                return Err(ResourceFaultCode::StaleScope.into());
            }
            let close = !record.root_live && record.holds.len() == 1;
            let revision = next(store.revision)?;
            store
                .scopes
                .get_mut(&hold.scope.ordinal())
                .unwrap()
                .holds
                .remove(&hold.ordinal);
            if close {
                store.close_scope_inner(&hold.scope)?;
            } else {
                store.revision = revision;
            }
            Ok(())
        })
    }

    /// Forced lifetime end (e.g. attempt teardown). Closing a scope removes its
    /// values and aborts whole reservations; it creates no refund receipt.
    pub fn close_scope(&mut self, scope: &ResourceScope) -> ResourceResult<()> {
        self.transaction(|store| store.close_scope_inner(scope))
    }

    fn close_scope_inner(&mut self, scope: &ResourceScope) -> ResourceResult<()> {
        self.scope_record(scope)?;
        let revision = next(self.revision)?;
        let removed: Vec<_> = self
            .reservations
            .iter()
            .filter(|(_, record)| {
                record
                    .costs
                    .iter()
                    .any(|(lease, _)| lease.key.scope == scope.ordinal())
            })
            .map(|(id, _)| *id)
            .collect();
        for id in removed {
            self.abort_record(id)?;
        }
        self.entries.retain(|key, _| key.scope != scope.ordinal());
        self.scopes.remove(&scope.ordinal());
        self.revision = revision;
        Ok(())
    }

    pub fn bind(
        &self,
        resource: &ResourceId,
        owner: ResourceOwner,
        scope: &ResourceScope,
    ) -> ResourceResult<ResourceLease> {
        self.scope_record(scope)?;
        let definition = self.definition(resource)?;
        if definition.owner != owner.kind() {
            return Err(ResourceFaultCode::OwnerKindMismatch.into());
        }
        if definition.lifetime != scope.lifetime() {
            return Err(ResourceFaultCode::WrongLifetime.into());
        }
        let token = self
            .owners
            .get(&owner)
            .ok_or(ResourceFaultCode::InvalidOwner)?;
        Ok(ResourceLease {
            key: ResourceKey {
                resource: resource.clone(),
                owner,
                scope: scope.ordinal(),
            },
            scope: scope.clone(),
            owner: token.clone(),
        })
    }

    fn definition(&self, id: &ResourceId) -> ResourceResult<&super::ResourceDefinition> {
        crate::authoring::control::charge(
            self.catalog.resources.len() as u64,
            "resource_definition_lookup",
        )?;
        self.catalog
            .resources
            .iter()
            .find(|definition| definition.id == *id)
            .ok_or_else(|| ResourceFaultCode::UnknownDefinition.into())
    }

    pub fn value(&self, lease: &ResourceLease) -> ResourceResult<ResourceValue> {
        self.scope_record(&lease.scope)?;
        if !self
            .owners
            .get(&lease.key.owner)
            .is_some_and(|token| Arc::ptr_eq(token, &lease.owner))
        {
            return Err(ResourceFaultCode::StaleLease.into());
        }
        let definition = self.definition(&lease.key.resource)?;
        Ok(self
            .entries
            .get(&lease.key)
            .copied()
            .unwrap_or(ResourceValue {
                balance: definition.initial,
                reserved: 0,
                revision: 0,
            }))
    }

    pub fn mutate(
        &mut self,
        lease: &ResourceLease,
        expected_revision: u64,
        operation: ResourceMutation,
    ) -> ResourceResult<ResourceReceipt> {
        self.transaction(|store| store.mutate_inner(lease, expected_revision, operation))
    }

    fn mutate_inner(
        &mut self,
        lease: &ResourceLease,
        expected_revision: u64,
        operation: ResourceMutation,
    ) -> ResourceResult<ResourceReceipt> {
        let store = self;
        let before = store.value(lease)?;
        if before.revision != expected_revision {
            return Err(ResourceFaultCode::StaleRevision.into());
        }
        if let ResourceMutation::Consume(amount) = operation {
            let reservation = store.reserve_inner(&[(lease.clone(), expected_revision, amount)])?;
            let mut receipts = store.commit_inner(&reservation)?;
            // Expose the public operation boundary, excluding its internal lock step.
            let receipt = receipts.first_mut().ok_or(ResourceFaultCode::Invariant)?;
            receipt.before = before;
            return Ok(receipts.remove(0));
        }
        let cap = store.definition(&lease.key.resource)?.cap;
        let balance = match operation {
            ResourceMutation::Grant(amount) => before.balance.saturating_add(amount).min(cap),
            ResourceMutation::Decay(amount) => {
                let after = before.balance.saturating_sub(amount);
                if after < before.reserved {
                    return Err(ResourceFaultCode::ReservationConflict.into());
                }
                after
            }
            ResourceMutation::Consume(_) => unreachable!(),
        };
        let after = ResourceValue {
            balance,
            reserved: before.reserved,
            revision: next(before.revision)?,
        };
        store.revision = next(store.revision)?;
        store.entries.insert(lease.key.clone(), after);
        Ok(ResourceReceipt {
            key: lease.key.clone(),
            operation,
            before,
            after,
            applied: before.balance.abs_diff(after.balance),
        })
    }

    /// Authored order is preserved, and every cost commits or the whole batch
    /// is discarded. Scope and owner capabilities are checked again here.
    pub fn apply_operations(
        &mut self,
        operations: &[(ResourceId, ResourceOwner, ResourceScope, ResourceMutation)],
    ) -> ResourceResult<Vec<ResourceReceipt>> {
        if operations.is_empty() || operations.len() > self.limits.batch {
            return Err(ResourceFaultCode::Limit.into());
        }
        crate::authoring::control::charge(operations.len() as u64, "resource_operation_batch")?;
        self.transaction(|store| {
            let mut receipts = Vec::with_capacity(operations.len());
            for (id, owner, scope, operation) in operations {
                let lease = store.bind(id, *owner, scope)?;
                let revision = store.value(&lease)?.revision;
                receipts.push(store.mutate_inner(&lease, revision, *operation)?);
            }
            Ok(receipts)
        })
    }

    /// An owning battle adapter may resolve exact occurrence leases before the
    /// atomic batch. Revalidation still happens inside the transaction clone.
    pub(crate) fn apply_leased_operations(
        &mut self,
        operations: &[(ResourceLease, ResourceMutation)],
    ) -> ResourceResult<Vec<ResourceReceipt>> {
        if operations.is_empty() || operations.len() > self.limits.batch {
            return Err(ResourceFaultCode::Limit.into());
        }
        crate::authoring::control::charge(operations.len() as u64, "resource_operation_batch")?;
        self.transaction(|store| {
            let mut receipts = Vec::with_capacity(operations.len());
            for (lease, operation) in operations {
                let revision = store.value(lease)?.revision;
                receipts.push(store.mutate_inner(lease, revision, *operation)?);
            }
            Ok(receipts)
        })
    }

    /// A reservation is an internal synchronous transaction capability. It is
    /// neither a serializable currency debit nor an authored delayed command.
    pub fn reserve(
        &mut self,
        costs: &[(ResourceLease, u64, u32)],
    ) -> ResourceResult<ResourceReservation> {
        self.transaction(|store| store.reserve_inner(costs))
    }

    fn reserve_inner(
        &mut self,
        costs: &[(ResourceLease, u64, u32)],
    ) -> ResourceResult<ResourceReservation> {
        if costs.is_empty()
            || costs.len() > self.limits.batch
            || self.reservations.len() >= self.limits.reservations
        {
            return Err(ResourceFaultCode::Limit.into());
        }
        let mut grouped = BTreeMap::<ResourceKey, (ResourceLease, ResourceValue, u32)>::new();
        for (lease, revision, amount) in costs {
            let before = self.value(lease)?;
            if before.revision != *revision {
                return Err(ResourceFaultCode::StaleRevision.into());
            }
            let entry = grouped
                .entry(lease.key.clone())
                .or_insert((lease.clone(), before, 0));
            entry.2 = entry
                .2
                .checked_add(*amount)
                .ok_or(ResourceFaultCode::CounterOverflow)?;
            if entry.2 > before.available() {
                return Err(ResourceFaultCode::Insufficient.into());
            }
        }
        let revision = next(self.revision)?;
        let ordinal = next(self.next_reservation)?;
        let handle = ResourceReservation {
            ordinal,
            issued_revision: revision,
            token: Arc::new(ReservationToken),
        };
        let mut locked = Vec::with_capacity(grouped.len());
        for (key, (lease, before, amount)) in grouped {
            let after = ResourceValue {
                balance: before.balance,
                reserved: before.reserved + amount,
                revision: next(before.revision)?,
            };
            self.entries.insert(key, after);
            locked.push((lease, amount));
        }
        self.reservations.insert(
            ordinal,
            ReservationRecord {
                handle: handle.clone(),
                costs: locked,
            },
        );
        self.next_reservation = ordinal;
        self.revision = revision;
        Ok(handle)
    }

    fn reservation(&self, handle: &ResourceReservation) -> ResourceResult<&ReservationRecord> {
        self.reservations
            .get(&handle.ordinal)
            .filter(|record| {
                record.handle.issued_revision == handle.issued_revision
                    && Arc::ptr_eq(&record.handle.token, &handle.token)
            })
            .ok_or_else(|| ResourceFaultCode::StaleReservation.into())
    }

    pub fn commit(&mut self, handle: &ResourceReservation) -> ResourceResult<Vec<ResourceReceipt>> {
        self.transaction(|store| store.commit_inner(handle))
    }

    fn commit_inner(
        &mut self,
        handle: &ResourceReservation,
    ) -> ResourceResult<Vec<ResourceReceipt>> {
        let record = clone_resource_value(self.reservation(handle)?)?;
        let revision = next(self.revision)?;
        let mut receipts = Vec::with_capacity(record.costs.len());
        for (lease, amount) in &record.costs {
            let before = self.value(lease)?;
            let after = ResourceValue {
                balance: before
                    .balance
                    .checked_sub(*amount)
                    .ok_or(ResourceFaultCode::Invariant)?,
                reserved: before
                    .reserved
                    .checked_sub(*amount)
                    .ok_or(ResourceFaultCode::Invariant)?,
                revision: next(before.revision)?,
            };
            receipts.push(ResourceReceipt {
                key: lease.key.clone(),
                operation: ResourceMutation::Consume(*amount),
                before,
                after,
                applied: *amount,
            });
        }
        for receipt in &receipts {
            self.entries.insert(receipt.key.clone(), receipt.after);
        }
        self.reservations.remove(&handle.ordinal);
        self.revision = revision;
        Ok(receipts)
    }

    pub fn abort(&mut self, handle: &ResourceReservation) -> ResourceResult<()> {
        self.transaction(|store| {
            store.reservation(handle)?;
            let revision = next(store.revision)?;
            store.abort_record(handle.ordinal)?;
            store.revision = revision;
            Ok(())
        })
    }

    fn abort_record(&mut self, ordinal: u64) -> ResourceResult<()> {
        let record = self
            .reservations
            .remove(&ordinal)
            .ok_or(ResourceFaultCode::StaleReservation)?;
        for (lease, amount) in record.costs {
            let value = self
                .entries
                .get_mut(&lease.key)
                .ok_or(ResourceFaultCode::Invariant)?;
            value.reserved = value
                .reserved
                .checked_sub(amount)
                .ok_or(ResourceFaultCode::Invariant)?;
            value.revision = next(value.revision)?;
        }
        Ok(())
    }

    /// Bounds apply even with event logging disabled. Conservative retained
    /// estimates include map nodes, immutable validity allocations and key bytes.
    pub fn check_extent(&self) -> ResourceResult<()> {
        self.retained_extent().map(|_| ())
    }

    pub fn retained_extent(&self) -> ResourceResult<usize> {
        if self.owners.len() > self.limits.owners
            || self.scopes.len() > self.limits.scopes
            || self.entries.len() > self.limits.entries
            || self.reservations.len() > self.limits.reservations
        {
            return Err(ResourceFaultCode::Limit.into());
        }
        let walk = self
            .scopes
            .len()
            .checked_add(self.entries.len())
            .and_then(|n| n.checked_add(self.reservations.len()))
            .ok_or(ResourceFaultCode::Limit)?;
        crate::authoring::control::charge(walk as u64, "resource_retained_walk")?;
        let mut holds = 0usize;
        fn tree<K, V>(count: usize) -> ResourceResult<usize> {
            let nodes = 1usize
                .checked_add(count.saturating_sub(1) / 5)
                .ok_or(ResourceFaultCode::Limit)?;
            let node = std::mem::size_of::<(K, V)>()
                .checked_mul(11)
                .and_then(|n| n.checked_add(12 * std::mem::size_of::<usize>() + 128))
                .ok_or(ResourceFaultCode::Limit)?;
            nodes
                .checked_mul(node)
                .ok_or_else(|| ResourceFaultCode::Limit.into())
        }
        let mut bytes = std::mem::size_of::<Self>();
        for extent in [
            tree::<ResourceOwner, Arc<OwnerToken>>(self.owners.len())?,
            tree::<u64, ScopeRecord>(self.scopes.len())?,
            tree::<ResourceKey, ResourceValue>(self.entries.len())?,
            tree::<u64, ReservationRecord>(self.reservations.len())?,
        ] {
            bytes = bytes.checked_add(extent).ok_or(ResourceFaultCode::Limit)?;
        }
        for record in self.scopes.values() {
            bytes = bytes
                .checked_add(tree::<u64, Arc<HoldToken>>(record.holds.len())?)
                .ok_or(ResourceFaultCode::Limit)?;
            holds = holds
                .checked_add(record.holds.len())
                .ok_or(ResourceFaultCode::Limit)?;
        }
        for (count, per_entry) in [
            (self.owners.len(), 64usize),
            (self.scopes.len(), 64),
            (holds, 64),
            (self.reservations.len(), 64),
        ] {
            bytes = bytes
                .checked_add(
                    count
                        .checked_mul(per_entry)
                        .ok_or(ResourceFaultCode::Limit)?,
                )
                .ok_or(ResourceFaultCode::Limit)?;
        }
        for key in self.entries.keys() {
            bytes = bytes
                .checked_add(key.resource.as_str().len())
                .ok_or(ResourceFaultCode::Limit)?;
        }
        for record in self.reservations.values() {
            if record.costs.len() > self.limits.batch {
                return Err(ResourceFaultCode::Limit.into());
            }
            crate::authoring::control::charge(
                record.costs.len() as u64,
                "resource_reserved_cost_walk",
            )?;
            for (lease, _) in &record.costs {
                bytes = bytes
                    .checked_add(256 + lease.key.resource.as_str().len())
                    .ok_or(ResourceFaultCode::Limit)?;
            }
        }
        if holds > self.limits.holds || bytes > self.limits.retained_bytes {
            return Err(ResourceFaultCode::Limit.into());
        }
        Ok(bytes)
    }
}

crate::authoring::clone_shape::copy_shape!(
    ResourceOwner,
    ResourceLimits,
    ResourceValue,
    ResourceMutation,
    ResourceFaultCode
);
crate::authoring::clone_shape::struct_shape!(ResourceScope { token });
crate::authoring::clone_shape::struct_shape!(ScopeHold {
    scope,
    ordinal,
    token
});
crate::authoring::clone_shape::struct_shape!(ResourceLease { key, scope, owner });
crate::authoring::clone_shape::struct_shape!(ResourceReservation {
    ordinal,
    issued_revision,
    token
});
crate::authoring::clone_shape::struct_shape!(ResourceKey {
    resource,
    owner,
    scope
});
crate::authoring::clone_shape::struct_shape!(ResourceReceipt {
    key,
    operation,
    before,
    after,
    applied
});
crate::authoring::clone_shape::struct_shape!(ScopeRecord {
    scope,
    root_live,
    holds,
    next_hold
});
crate::authoring::clone_shape::struct_shape!(ReservationRecord { handle, costs });
crate::authoring::clone_shape::struct_shape!(ResourceStore {
    catalog,
    owners,
    scopes,
    entries,
    reservations,
    next_scope,
    next_reservation,
    revision,
    limits
});

#[cfg(test)]
mod tests;
