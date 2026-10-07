//! Bind resource authority to actual battle occurrences. Event/source adapters
//! carry capabilities explicitly; observation sources drop them.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use super::{
    durable::{RunResourceOverlay, RunResourceResult, RunResourceState},
    store::{
        ResourceError, ResourceFaultCode, ResourceOwner, ResourceScope, ResourceStore, ScopeHold,
    },
    ResourceCatalog, ResourceLifetime,
};
use crate::game::{
    battle::{
        ids::UnitInstanceId,
        runtime_facts::{ActionHandle, ActionOrdinal},
    },
    behavior::GameError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceContext {
    pub activation: Option<ResourceScope>,
    pub chain: ResourceScope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDelivery {
    pub scope: ResourceScope,
    hold: ScopeHold,
    continuation: Option<ScopeHold>,
}

#[derive(Debug, Clone)]
enum ChainCompletion {
    Root(ResourceScope),
    Child(ScopeHold),
}

#[derive(Debug, Clone)]
struct ResourceAction {
    action: ActionHandle,
    context: Arc<ResourceContext>,
    completion: Option<ChainCompletion>,
}

#[derive(Debug, Clone)]
pub struct RuntimeResources {
    pub store: ResourceStore,
    pub attempt: ResourceScope,
    pub run: Option<RunResourceOverlay>,
    pub deployments: BTreeMap<UnitInstanceId, ResourceScope>,
    actions: BTreeMap<ActionOrdinal, ResourceAction>,
}

impl RuntimeResources {
    pub fn new(catalog: Arc<ResourceCatalog>) -> Result<Self, GameError> {
        let mut store = ResourceStore::new(catalog).map_err(ResourceError::into_game_error)?;
        let attempt = store
            .open_scope(ResourceLifetime::Attempt)
            .map_err(ResourceError::into_game_error)?;
        Ok(Self {
            store,
            attempt,
            run: None,
            deployments: BTreeMap::new(),
            actions: BTreeMap::new(),
        })
    }

    pub fn mount_run(
        &mut self,
        state: &RunResourceState,
        owners: &BTreeSet<ResourceOwner>,
    ) -> Result<(), GameError> {
        if self.run.is_some() {
            return Err(ResourceError::from(ResourceFaultCode::StaleScope).into_game_error());
        }
        self.store
            .reconcile_owners(owners)
            .map_err(ResourceError::into_game_error)?;
        self.run = Some(
            self.store
                .mount_run(state)
                .map_err(ResourceError::into_game_error)?,
        );
        Ok(())
    }

    pub fn capture_run(&self) -> Result<Option<RunResourceResult>, GameError> {
        self.run
            .as_ref()
            .map(|overlay| {
                self.store
                    .capture_run(overlay)
                    .map_err(ResourceError::into_game_error)
            })
            .transpose()
    }

    pub fn deploy(&mut self, unit: UnitInstanceId) -> Result<(), GameError> {
        if self.deployments.contains_key(&unit) {
            return Err(ResourceError::from(ResourceFaultCode::Invariant).into_game_error());
        }
        let scope = self
            .store
            .open_scope(ResourceLifetime::Deployment)
            .map_err(ResourceError::into_game_error)?;
        self.deployments.insert(unit, scope);
        Ok(())
    }

    pub fn end_deployment(&mut self, unit: UnitInstanceId) -> Result<(), GameError> {
        if let Some(scope) = self.deployments.remove(&unit) {
            self.store
                .close_scope(&scope)
                .map_err(ResourceError::into_game_error)?;
        }
        Ok(())
    }

    pub fn context(&self, action: &ActionHandle) -> Option<&Arc<ResourceContext>> {
        self.actions
            .get(&action.ordinal())
            .filter(|record| record.action == *action)
            .map(|record| &record.context)
    }

    pub fn accept(
        &mut self,
        action: &ActionHandle,
        parent: Option<&ResourceContext>,
    ) -> Result<(), GameError> {
        if self.actions.contains_key(&action.ordinal()) {
            return Err(ResourceError::from(ResourceFaultCode::Invariant).into_game_error());
        }
        let (context, completion) = self
            .store
            .transaction(|store| {
                let activation = Some(store.open_scope(ResourceLifetime::Activation)?);
                let (chain, completion) = match parent {
                    Some(parent) => (
                        parent.chain.clone(),
                        ChainCompletion::Child(store.retain_chain(&parent.chain)?),
                    ),
                    None => {
                        let chain = store.open_scope(ResourceLifetime::Chain)?;
                        (chain.clone(), ChainCompletion::Root(chain))
                    }
                };
                crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    Some(std::mem::size_of::<ResourceContext>() + 64),
                    "resource_context_owner",
                )?;
                Ok((Arc::new(ResourceContext { activation, chain }), completion))
            })
            .map_err(ResourceError::into_game_error)?;
        self.actions.insert(
            action.ordinal(),
            ResourceAction {
                action: action.clone(),
                context,
                completion: Some(completion),
            },
        );
        Ok(())
    }

    pub fn terminal(&mut self, action: &ActionHandle) -> Result<(), GameError> {
        let Some(record) = self
            .actions
            .get(&action.ordinal())
            .filter(|record| record.action == *action)
        else {
            return Ok(());
        };
        let Some(completion) = &record.completion else {
            return Ok(());
        };
        self.store
            .transaction(|store| {
                if let Some(activation) = &record.context.activation {
                    store.close_scope(activation)?;
                }
                match completion {
                    ChainCompletion::Root(chain) => store.finish_scope(chain),
                    ChainCompletion::Child(hold) => store.release_chain(hold),
                }
            })
            .map_err(ResourceError::into_game_error)?;
        self.actions
            .get_mut(&action.ordinal())
            .expect("checked action")
            .completion = None;
        Ok(())
    }

    pub fn settled(&mut self, action: &ActionHandle) {
        if self
            .actions
            .get(&action.ordinal())
            .is_some_and(|record| record.action == *action)
        {
            self.actions.remove(&action.ordinal());
        }
    }

    /// A root finite occurrence may have no R04 action (e.g. a queued trigger).
    /// Its initial root reference ends after admission; the finite hold owns it.
    pub fn finite(
        &mut self,
        parent: Option<&ResourceContext>,
    ) -> Result<(Arc<ResourceContext>, Arc<ResourceDelivery>), GameError> {
        self.finite_with_scope(parent, None)
    }

    /// A carrier's queued impact/tick continues that payload's Delivery rather
    /// than starting an independent payload. Its finite hold survives removal
    /// of the motion/area root until the admitted completion actually ends.
    pub fn finite_completion(
        &mut self,
        parent: Option<&ResourceContext>,
        delivery: &ResourceDelivery,
    ) -> Result<(Arc<ResourceContext>, Arc<ResourceDelivery>), GameError> {
        self.finite_with_scope(parent, Some(&delivery.scope))
    }

    fn finite_with_scope(
        &mut self,
        parent: Option<&ResourceContext>,
        delivery: Option<&ResourceScope>,
    ) -> Result<(Arc<ResourceContext>, Arc<ResourceDelivery>), GameError> {
        self.store
            .transaction(|store| {
                let context = match parent {
                    Some(parent) => parent.clone(),
                    None => ResourceContext {
                        activation: None,
                        chain: store.open_scope(ResourceLifetime::Chain)?,
                    },
                };
                let hold = store.retain_chain(&context.chain)?;
                let (scope, continuation) = match delivery {
                    Some(scope) => (scope.clone(), Some(store.retain_delivery(scope)?)),
                    None => (store.open_scope(ResourceLifetime::Delivery)?, None),
                };
                if parent.is_none() {
                    store.finish_scope(&context.chain)?;
                }
                crate::game::battle::runtime_control::admit_retained_growth(
                    0,
                    Some(
                        std::mem::size_of::<ResourceContext>()
                            + std::mem::size_of::<ResourceDelivery>()
                            + 128,
                    ),
                    "resource_finite_source_owners",
                )?;
                Ok((
                    Arc::new(context),
                    Arc::new(ResourceDelivery {
                        scope,
                        hold,
                        continuation,
                    }),
                ))
            })
            .map_err(ResourceError::into_game_error)
    }

    pub fn close_finite(&mut self, delivery: &ResourceDelivery) -> Result<(), GameError> {
        self.store
            .transaction(|store| {
                match &delivery.continuation {
                    Some(hold) => store.release_delivery(hold)?,
                    None => store.finish_scope(&delivery.scope)?,
                }
                store.release_chain(&delivery.hold)
            })
            .map_err(ResourceError::into_game_error)
    }

    pub fn retained_extent(&self) -> Result<usize, GameError> {
        fn tree<K, V>(len: usize) -> Option<usize> {
            let nodes = 1usize.checked_add(len.saturating_sub(1) / 5)?;
            let node = std::mem::size_of::<(K, V)>()
                .checked_mul(11)?
                .checked_add(12 * std::mem::size_of::<usize>() + 128)?;
            nodes.checked_mul(node)
        }
        if let Some(run) = &self.run {
            crate::authoring::control::charge(
                run.basis.entries.len() as u64,
                "resource_basis_extent",
            )?;
        }
        let result =
            || -> Option<usize> {
                let mut bytes = std::mem::size_of::<Self>()
                    .checked_add(tree::<UnitInstanceId, ResourceScope>(
                        self.deployments.len(),
                    )?)?
                    .checked_add(tree::<ActionOrdinal, ResourceAction>(self.actions.len())?)?
                    .checked_add(self.deployments.len().checked_mul(64)?)?
                    .checked_add(self.actions.len().checked_mul(512)?)?;
                if let Some(run) = &self.run {
                    bytes = bytes.checked_add(std::mem::size_of::<RunResourceOverlay>() + 64)?;
                    bytes =
                        bytes.checked_add(run.basis.entries.capacity().checked_mul(
                            std::mem::size_of::<super::durable::RunResourceEntry>(),
                        )?)?;
                    for entry in &run.basis.entries {
                        bytes = bytes.checked_add(entry.resource.as_str().len())?;
                    }
                }
                Some(bytes)
            };
        let store_bytes = self
            .store
            .retained_extent()
            .map_err(ResourceError::into_game_error)?;
        result()
            .and_then(|bytes| bytes.checked_add(store_bytes))
            .ok_or_else(|| ResourceError::from(ResourceFaultCode::Limit).into_game_error())
    }
}

crate::authoring::clone_shape::struct_shape!(ResourceContext { activation, chain });
crate::authoring::clone_shape::struct_shape!(ResourceDelivery {
    scope,
    hold,
    continuation
});
crate::authoring::clone_shape::enum_shape!(ChainCompletion {
    units: [],
    tuples: [Root(scope), Child(hold)],
    records: []
});
crate::authoring::clone_shape::struct_shape!(ResourceAction {
    action,
    context,
    completion
});
crate::authoring::clone_shape::struct_shape!(RuntimeResources {
    store,
    attempt,
    run,
    deployments,
    actions
});
