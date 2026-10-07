use super::*;
use crate::game::resource::ResourceDefinition;

fn id(name: &str) -> ResourceId {
    ResourceId::new(name.to_owned()).unwrap()
}
fn employee(n: u128) -> ResourceOwner {
    ResourceOwner::Employee(Uuid::from_u128(n))
}
fn equipment(n: u128) -> ResourceOwner {
    ResourceOwner::EquipmentCopy(Uuid::from_u128(n))
}

fn store() -> ResourceStore {
    let mut resources = Vec::new();
    for lifetime in [
        ResourceLifetime::Run,
        ResourceLifetime::Attempt,
        ResourceLifetime::Deployment,
        ResourceLifetime::Activation,
        ResourceLifetime::Delivery,
        ResourceLifetime::Chain,
    ] {
        for owner in [
            ResourceOwnerKind::Employee,
            ResourceOwnerKind::EquipmentCopy,
        ] {
            resources.push(ResourceDefinition {
                id: id(&format!("{owner:?}_{lifetime:?}")),
                owner,
                lifetime,
                initial: 5,
                cap: 10,
            });
        }
    }
    let mut store = ResourceStore::new(Arc::new(ResourceCatalog { resources })).unwrap();
    store
        .reconcile_owners(&[employee(1), employee(2), equipment(3), equipment(4)].into())
        .unwrap();
    store
}

fn bind(store: &ResourceStore, owner: ResourceOwner, scope: &ResourceScope) -> ResourceLease {
    store
        .bind(
            &id(&format!("{:?}_{:?}", owner.kind(), scope.lifetime())),
            owner,
            scope,
        )
        .unwrap()
}

/// Snapshot every mutable authoritative field, excluding immutable pointer
/// addresses. Tests separately challenge identity with old handles.
fn snapshot(store: &ResourceStore) -> serde_json::Value {
    serde_json::json!({
        "owners": store.owners.keys().collect::<Vec<_>>(),
        "scopes": store.scopes.values().map(|r| (r.scope.ordinal(), r.scope.lifetime(), r.root_live,
            r.next_hold, r.holds.keys().copied().collect::<Vec<_>>())).collect::<Vec<_>>(),
        "entries": store.entries.iter().collect::<Vec<_>>(),
        "reservations": store.reservations.values().map(|r| (r.handle.ordinal, r.handle.issued_revision,
            r.costs.iter().map(|(lease, amount)| (&lease.key, amount)).collect::<Vec<_>>())).collect::<Vec<_>>(),
        "next_scope": store.next_scope, "next_reservation": store.next_reservation, "revision": store.revision,
    })
}

fn fault<T: std::fmt::Debug>(result: ResourceResult<T>, expected: ResourceFaultCode) {
    assert!(
        matches!(&result, Err(ResourceError::Fault(code)) if *code == expected),
        "{result:?}"
    );
}

#[test]
fn each_lifetime_closes_its_own_entries_and_rejects_old_lease() {
    let mut store = store();
    for lifetime in [
        ResourceLifetime::Run,
        ResourceLifetime::Attempt,
        ResourceLifetime::Deployment,
        ResourceLifetime::Activation,
        ResourceLifetime::Delivery,
        ResourceLifetime::Chain,
    ] {
        let scope = store.open_scope(lifetime).unwrap();
        let a = bind(&store, equipment(3), &scope);
        let b = bind(&store, equipment(4), &scope);
        assert_eq!(store.value(&a).unwrap().balance, 5);
        store.mutate(&a, 0, ResourceMutation::Consume(3)).unwrap();
        assert_eq!(store.value(&a).unwrap().balance, 2);
        assert_eq!(store.value(&b).unwrap().balance, 5);
        store.finish_scope(&scope).unwrap();
        let ended = snapshot(&store);
        fault(store.value(&a), ResourceFaultCode::StaleScope);
        fault(
            store.mutate(&a, 2, ResourceMutation::Grant(10)),
            ResourceFaultCode::StaleScope,
        );
        fault(store.finish_scope(&scope), ResourceFaultCode::StaleScope);
        assert_eq!(snapshot(&store), ended);
        let replacement = store.open_scope(lifetime).unwrap();
        let fresh = bind(&store, equipment(3), &replacement);
        assert_eq!(store.value(&fresh).unwrap().balance, 5);
        store.finish_scope(&replacement).unwrap();
    }
    assert!(store.entries.is_empty());
    assert!(store.scopes.is_empty());
}

#[test]
fn closing_deployment_preserves_run_attempt_and_other_owners() {
    let mut store = store();
    let run = store.open_scope(ResourceLifetime::Run).unwrap();
    let attempt = store.open_scope(ResourceLifetime::Attempt).unwrap();
    let deployment = store.open_scope(ResourceLifetime::Deployment).unwrap();
    let leases = [
        bind(&store, employee(1), &run),
        bind(&store, equipment(3), &attempt),
        bind(&store, equipment(3), &deployment),
    ];
    for lease in &leases {
        store
            .mutate(lease, 0, ResourceMutation::Consume(2))
            .unwrap();
    }
    store.close_scope(&deployment).unwrap();
    assert_eq!(store.value(&leases[0]).unwrap().balance, 3);
    assert_eq!(store.value(&leases[1]).unwrap().balance, 3);
    fault(store.value(&leases[2]), ResourceFaultCode::StaleScope);
    let deployment = store.open_scope(ResourceLifetime::Deployment).unwrap();
    assert_eq!(
        store
            .value(&bind(&store, equipment(3), &deployment))
            .unwrap()
            .balance,
        5
    );
}

#[test]
fn unknown_or_wrong_owner_and_lifetime_do_not_create_state() {
    let mut store = store();
    let run = store.open_scope(ResourceLifetime::Run).unwrap();
    let before = snapshot(&store);
    fault(
        store.bind(&id("missing"), equipment(3), &run),
        ResourceFaultCode::UnknownDefinition,
    );
    fault(
        store.bind(&id("EquipmentCopy_Run"), equipment(999), &run),
        ResourceFaultCode::InvalidOwner,
    );
    fault(
        store.bind(&id("EquipmentCopy_Run"), employee(1), &run),
        ResourceFaultCode::OwnerKindMismatch,
    );
    fault(
        store.bind(&id("EquipmentCopy_Attempt"), equipment(3), &run),
        ResourceFaultCode::WrongLifetime,
    );
    fault(
        store.reconcile_owners(&[employee(0)].into()),
        ResourceFaultCode::InvalidOwner,
    );
    assert_eq!(snapshot(&store), before);
}

#[test]
fn discarded_candidate_and_new_store_cannot_reuse_old_ordinal_capability() {
    let mut original = store();
    let mut candidate = original.clone();
    let old = candidate.open_scope(ResourceLifetime::Attempt).unwrap();
    let fresh = original.open_scope(ResourceLifetime::Attempt).unwrap();
    assert_eq!(old.ordinal(), fresh.ordinal());
    fault(
        original.bind(&id("EquipmentCopy_Attempt"), equipment(3), &old),
        ResourceFaultCode::StaleScope,
    );
    let mut unrelated = store();
    let unrelated_scope = unrelated.open_scope(ResourceLifetime::Attempt).unwrap();
    assert_eq!(fresh.ordinal(), unrelated_scope.ordinal());
    fault(unrelated.close_scope(&fresh), ResourceFaultCode::StaleScope);
    // Existing capabilities work in a clone while mutations remain local.
    let lease = bind(&original, equipment(3), &fresh);
    let mut cloned = original.clone();
    cloned
        .mutate(&lease, 0, ResourceMutation::Consume(1))
        .unwrap();
    assert_eq!(original.value(&lease).unwrap().balance, 5);
    assert_eq!(cloned.value(&lease).unwrap().balance, 4);
}

#[test]
fn chain_waits_for_finite_children_and_duplicate_release_cannot_close_a_new_child() {
    let mut store = store();
    let chain = store.open_scope(ResourceLifetime::Chain).unwrap();
    let lease = bind(&store, equipment(3), &chain);
    let child = store.retain_chain(&chain).unwrap();
    store.finish_scope(&chain).unwrap();
    store
        .mutate(&lease, 0, ResourceMutation::Consume(1))
        .unwrap();
    let grandchild = store.retain_chain(&chain).unwrap();
    store.release_chain(&child).unwrap();
    let before = snapshot(&store);
    fault(store.release_chain(&child), ResourceFaultCode::StaleScope);
    assert_eq!(snapshot(&store), before);
    assert_eq!(store.value(&lease).unwrap().balance, 4);
    store.release_chain(&grandchild).unwrap();
    fault(store.value(&lease), ResourceFaultCode::StaleScope);
    assert!(store.scopes.is_empty());
}

#[test]
fn activation_end_does_not_close_released_delivery_or_independent_chain() {
    let mut store = store();
    let activation = store.open_scope(ResourceLifetime::Activation).unwrap();
    let delivery = store.open_scope(ResourceLifetime::Delivery).unwrap();
    let chain = store.open_scope(ResourceLifetime::Chain).unwrap();
    let other = store.open_scope(ResourceLifetime::Chain).unwrap();
    let child = store.retain_chain(&chain).unwrap();
    let activation_lease = bind(&store, equipment(3), &activation);
    let delivery_lease = bind(&store, equipment(3), &delivery);
    store.close_scope(&activation).unwrap();
    store.finish_scope(&chain).unwrap();
    fault(
        store.value(&activation_lease),
        ResourceFaultCode::StaleScope,
    );
    store
        .mutate(&delivery_lease, 0, ResourceMutation::Consume(1))
        .unwrap();
    store.finish_scope(&delivery).unwrap();
    store.release_chain(&child).unwrap();
    assert_eq!(
        store
            .value(&bind(&store, equipment(3), &other))
            .unwrap()
            .balance,
        5
    );
}

#[test]
fn multi_cost_shortage_and_duplicate_key_aggregation_preserve_every_field() {
    let mut store = store();
    let scope = store.open_scope(ResourceLifetime::Attempt).unwrap();
    let a = bind(&store, equipment(3), &scope);
    let b = bind(&store, equipment(4), &scope);
    let before = snapshot(&store);
    fault(
        store.reserve(&[(a.clone(), 0, 2), (b.clone(), 0, 6)]),
        ResourceFaultCode::Insufficient,
    );
    assert_eq!(snapshot(&store), before);
    fault(
        store.reserve(&[(a.clone(), 0, 3), (a.clone(), 0, 3)]),
        ResourceFaultCode::Insufficient,
    );
    assert_eq!(snapshot(&store), before);
    let reservation = store
        .reserve(&[(a.clone(), 0, 2), (b.clone(), 0, 1), (a.clone(), 0, 1)])
        .unwrap();
    assert_eq!(store.value(&a).unwrap().balance, 5);
    assert_eq!(store.value(&a).unwrap().reserved, 3);
    let receipts = store.commit(&reservation).unwrap();
    assert_eq!(receipts.len(), 2);
    assert_eq!(store.value(&a).unwrap().balance, 2);
    assert_eq!(store.value(&b).unwrap().balance, 4);
    assert_eq!(receipts[0].key.owner, equipment(3));
    assert_eq!(
        (
            receipts[0].before.balance,
            receipts[0].after.balance,
            receipts[0].applied
        ),
        (5, 2, 3)
    );
    let after = snapshot(&store);
    fault(
        store.commit(&reservation),
        ResourceFaultCode::StaleReservation,
    );
    fault(
        store.abort(&reservation),
        ResourceFaultCode::StaleReservation,
    );
    assert_eq!(snapshot(&store), after);
}

#[test]
fn competing_reservations_lock_available_and_abort_never_grants_currency() {
    let mut store = store();
    let scope = store.open_scope(ResourceLifetime::Run).unwrap();
    let a = bind(&store, equipment(3), &scope);
    let one = store.reserve(&[(a.clone(), 0, 3)]).unwrap();
    let two = store.reserve(&[(a.clone(), 1, 2)]).unwrap();
    assert_eq!(store.value(&a).unwrap().available(), 0);
    let before = snapshot(&store);
    fault(
        store.mutate(&a, 2, ResourceMutation::Consume(1)),
        ResourceFaultCode::Insufficient,
    );
    fault(
        store.mutate(&a, 2, ResourceMutation::Decay(1)),
        ResourceFaultCode::ReservationConflict,
    );
    fault(
        store.reserve(&[(a.clone(), 0, 1)]),
        ResourceFaultCode::StaleRevision,
    );
    assert_eq!(snapshot(&store), before);
    store.commit(&one).unwrap();
    assert_eq!(store.value(&a).unwrap().balance, 2);
    store.abort(&two).unwrap();
    assert_eq!(store.value(&a).unwrap().balance, 2);
    assert_eq!(store.value(&a).unwrap().reserved, 0);
    let before = snapshot(&store);
    fault(store.abort(&two), ResourceFaultCode::StaleReservation);
    assert_eq!(snapshot(&store), before);
}

#[test]
fn stale_reservation_revision_and_rolled_back_handle_cannot_spend_new_reservation() {
    let mut store = store();
    let scope = store.open_scope(ResourceLifetime::Run).unwrap();
    let lease = bind(&store, employee(1), &scope);
    let mut candidate = store.clone();
    let old = candidate.reserve(&[(lease.clone(), 0, 1)]).unwrap();
    let fresh = store.reserve(&[(lease, 0, 1)]).unwrap();
    assert_eq!(old.ordinal, fresh.ordinal);
    let before = snapshot(&store);
    fault(store.commit(&old), ResourceFaultCode::StaleReservation);
    let mut altered = fresh.clone();
    altered.issued_revision -= 1;
    fault(store.commit(&altered), ResourceFaultCode::StaleReservation);
    assert_eq!(snapshot(&store), before);
    store.commit(&fresh).unwrap();
}

#[test]
fn owner_retention_preserves_values_and_removal_releases_other_cost_locks_without_refund() {
    let mut store = store();
    let run = store.open_scope(ResourceLifetime::Run).unwrap();
    let a = bind(&store, equipment(3), &run);
    let b = bind(&store, employee(1), &run);
    store.mutate(&a, 0, ResourceMutation::Consume(2)).unwrap();
    let before = snapshot(&store);
    store
        .reconcile_owners(&[employee(1), employee(2), equipment(3), equipment(4)].into())
        .unwrap();
    assert_eq!(snapshot(&store), before);
    let reservation = store
        .reserve(&[(a.clone(), 2, 1), (b.clone(), 0, 2)])
        .unwrap();
    store
        .reconcile_owners(&[employee(1), employee(2), equipment(4)].into())
        .unwrap();
    fault(store.value(&a), ResourceFaultCode::StaleLease);
    fault(
        store.commit(&reservation),
        ResourceFaultCode::StaleReservation,
    );
    assert_eq!(store.value(&b).unwrap().balance, 5);
    assert_eq!(store.value(&b).unwrap().reserved, 0);
    assert!(store.entries.keys().all(|key| key.owner != equipment(3)));
    // Even a forged reintroduction of the same UUID cannot revive an old lease.
    store
        .reconcile_owners(&[employee(1), equipment(3)].into())
        .unwrap();
    fault(store.value(&a), ResourceFaultCode::StaleLease);
}

#[test]
fn closing_one_lifetime_aborts_whole_reservation_and_retains_other_balance() {
    let mut store = store();
    let run = store.open_scope(ResourceLifetime::Run).unwrap();
    let activation = store.open_scope(ResourceLifetime::Activation).unwrap();
    let a = bind(&store, employee(1), &run);
    let b = bind(&store, employee(1), &activation);
    let reservation = store.reserve(&[(a.clone(), 0, 2), (b, 0, 3)]).unwrap();
    store.close_scope(&activation).unwrap();
    assert_eq!(store.value(&a).unwrap().balance, 5);
    assert_eq!(store.value(&a).unwrap().reserved, 0);
    fault(
        store.commit(&reservation),
        ResourceFaultCode::StaleReservation,
    );
}

#[test]
fn grant_decay_and_consume_report_actual_changes_at_arithmetic_boundaries() {
    let mut store = store();
    let scope = store.open_scope(ResourceLifetime::Run).unwrap();
    let lease = bind(&store, equipment(3), &scope);
    let receipt = store
        .mutate(&lease, 0, ResourceMutation::Grant(u32::MAX))
        .unwrap();
    assert_eq!(
        (
            receipt.before.balance,
            receipt.after.balance,
            receipt.applied
        ),
        (5, 10, 5)
    );
    let receipt = store
        .mutate(&lease, 1, ResourceMutation::Decay(u32::MAX))
        .unwrap();
    assert_eq!(
        (
            receipt.before.balance,
            receipt.after.balance,
            receipt.applied
        ),
        (10, 0, 10)
    );
    let before = snapshot(&store);
    fault(
        store.mutate(&lease, 2, ResourceMutation::Consume(1)),
        ResourceFaultCode::Insufficient,
    );
    assert_eq!(snapshot(&store), before);
    store.mutate(&lease, 2, ResourceMutation::Grant(4)).unwrap();
    let receipt = store
        .mutate(&lease, 3, ResourceMutation::Consume(4))
        .unwrap();
    assert_eq!(receipt.before.reserved, 0);
    assert_eq!(
        (
            receipt.before.balance,
            receipt.after.balance,
            receipt.applied
        ),
        (4, 0, 4)
    );
}

#[test]
fn failures_after_local_mutation_restore_entries_ids_and_revisions() {
    let mut store = store();
    let scope = store.open_scope(ResourceLifetime::Run).unwrap();
    let lease = bind(&store, equipment(3), &scope);
    let before = snapshot(&store);
    let result: ResourceResult<()> = store.transaction(|candidate| {
        candidate.mutate(&lease, 0, ResourceMutation::Consume(2))?;
        candidate.open_scope(ResourceLifetime::Delivery)?;
        Err(ResourceFaultCode::Limit.into())
    });
    fault(result, ResourceFaultCode::Limit);
    assert_eq!(snapshot(&store), before);
    store.limits.entries = 0;
    fault(
        store.mutate(&lease, 0, ResourceMutation::Grant(1)),
        ResourceFaultCode::Limit,
    );
    assert_eq!(snapshot(&store), before);
    store.limits.entries = 65536;
    store.limits.reservations = 0;
    fault(store.reserve(&[(lease, 0, 1)]), ResourceFaultCode::Limit);
    assert_eq!(snapshot(&store), before);
    store.limits.scopes = 1;
    fault(
        store.open_scope(ResourceLifetime::Delivery),
        ResourceFaultCode::Limit,
    );
    assert_eq!(snapshot(&store), before);
}

#[test]
fn hold_retention_and_revision_overflow_reject_without_publishing_ids() {
    let mut store = store();
    let scope = store.open_scope(ResourceLifetime::Chain).unwrap();
    store.limits.holds = 0;
    let before = snapshot(&store);
    fault(store.retain_chain(&scope), ResourceFaultCode::Limit);
    assert_eq!(snapshot(&store), before);
    store.revision = u64::MAX;
    let before = snapshot(&store);
    fault(
        store.close_scope(&scope),
        ResourceFaultCode::CounterOverflow,
    );
    assert_eq!(snapshot(&store), before);
}

#[test]
fn runtime_work_failure_rejects_before_store_commit() {
    use runtime_control::{RuntimeFactLimits, RuntimeFaultCode};
    let mut store = store();
    let scope = store.open_scope(ResourceLifetime::Run).unwrap();
    let lease = bind(&store, equipment(3), &scope);
    let before = snapshot(&store);
    let result: std::result::Result<(), GameError> = runtime_control::run(
        RuntimeFactLimits {
            max_work: 1,
            ..Default::default()
        },
        || match store.mutate(&lease, 0, ResourceMutation::Consume(1)) {
            Err(ResourceError::Control(error)) => Err(error),
            other => panic!("expected control failure, got {other:?}"),
        },
    );
    assert!(
        matches!(result, Err(GameError::BattleRuntimeFault(fault)) if fault.code == RuntimeFaultCode::WorkLimit)
    );
    assert_eq!(snapshot(&store), before);
}

#[test]
fn native_clone_budget_prices_store_before_copying_or_mutating() {
    use runtime_control::{RuntimeFactLimits, RuntimeFaultCode};
    let mut store = store();
    let scope = store.open_scope(ResourceLifetime::Run).unwrap();
    let lease = bind(&store, equipment(3), &scope);
    let before = snapshot(&store);
    let result: std::result::Result<(), GameError> = runtime_control::run(
        RuntimeFactLimits {
            max_clone_slots: 1,
            ..Default::default()
        },
        || match store.mutate(&lease, 0, ResourceMutation::Consume(1)) {
            Err(ResourceError::Control(error)) => Err(error),
            other => panic!("expected clone admission failure, got {other:?}"),
        },
    );
    assert!(
        matches!(result, Err(GameError::BattleRuntimeFault(fault)) if fault.code == RuntimeFaultCode::CloneLimit)
    );
    assert_eq!(snapshot(&store), before);
}

#[test]
fn catalog_validation_and_new_run_projection_are_admitted_before_work_and_copy() {
    use runtime_control::{RuntimeFactLimits, RuntimeFaultCode};
    let mut store = store();
    let root = crate::game::resource::RunResourceState::default();
    let overlay = store.mount_run(&root).unwrap();
    for owner in [employee(1), equipment(3)] {
        let lease = bind(&store, owner, &overlay.scope);
        store
            .mutate(&lease, 0, ResourceMutation::Consume(2))
            .unwrap();
    }
    let before = snapshot(&store);
    let denied = runtime_control::run(
        RuntimeFactLimits {
            max_work: 1,
            ..Default::default()
        },
        || {
            store
                .catalog
                .validate()
                .map_err(|error| ResourceError::Definition(error).into_game_error())
        },
    );
    assert!(
        matches!(denied, Err(GameError::BattleRuntimeFault(fault)) if fault.code == RuntimeFaultCode::WorkLimit)
    );
    let denied = runtime_control::run(
        RuntimeFactLimits {
            max_clone_slots: 16,
            ..Default::default()
        },
        || {
            store
                .capture_run(&overlay)
                .map_err(ResourceError::into_game_error)
        },
    );
    assert!(
        matches!(denied, Err(GameError::BattleRuntimeFault(fault)) if fault.code == RuntimeFaultCode::CloneLimit)
    );
    assert_eq!(snapshot(&store), before);
    assert!(root.entries.is_empty());
    let admitted = store.capture_run(&overlay).unwrap();
    let mut committed = root;
    admitted.apply(&mut committed).unwrap();
    assert_eq!(committed.entries.len(), 2);
    assert!(committed.entries.iter().all(|entry| entry.balance == 3));
}

#[test]
fn deterministic_order_and_sparse_initial_survive_run_projection_and_new_store() {
    use crate::game::resource::durable::RunResourceState;
    let mut root = RunResourceState::default();
    let mut first = store();
    let overlay = first.mount_run(&root).unwrap();
    let a = bind(&first, equipment(3), &overlay.scope);
    let b = bind(&first, equipment(4), &overlay.scope);
    first.mutate(&a, 0, ResourceMutation::Consume(2)).unwrap();
    assert_eq!(first.value(&b).unwrap().balance, 5);
    let result = first.capture_run(&overlay).unwrap();
    result.apply(&mut root).unwrap();
    assert_eq!(root.revision, 1);
    assert_eq!(root.entries.len(), 1);
    let encoded = serde_json::to_vec(&root).unwrap();
    let decoded: RunResourceState = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded, root);
    let mut second = store();
    let second_overlay = second.mount_run(&decoded).unwrap();
    let second_a = bind(&second, equipment(3), &second_overlay.scope);
    let second_b = bind(&second, equipment(4), &second_overlay.scope);
    assert_eq!(second.value(&second_a).unwrap().balance, 3);
    assert_eq!(second.value(&second_b).unwrap().balance, 5);
    fault(second.value(&a), ResourceFaultCode::StaleScope);
    second
        .mutate(&second_a, 2, ResourceMutation::Consume(1))
        .unwrap();
    second
        .capture_run(&second_overlay)
        .unwrap()
        .apply(&mut root)
        .unwrap();
    assert_eq!(root.entries[0].balance, 2);
    assert_eq!(root.revision, 2);
    let mut replay = store();
    let replay_overlay = replay.mount_run(&decoded).unwrap();
    let replay_a = bind(&replay, equipment(3), &replay_overlay.scope);
    replay
        .mutate(&replay_a, 2, ResourceMutation::Consume(1))
        .unwrap();
    let mut replay_root = decoded;
    replay
        .capture_run(&replay_overlay)
        .unwrap()
        .apply(&mut replay_root)
        .unwrap();
    assert_eq!(
        serde_json::to_vec(&replay_root).unwrap(),
        serde_json::to_vec(&root).unwrap()
    );
}

#[test]
fn run_result_rejects_duplicate_wrong_basis_and_pending_reservations() {
    use crate::game::resource::durable::RunResourceState;
    let mut root = RunResourceState::default();
    let mut store = store();
    let overlay = store.mount_run(&root).unwrap();
    let lease = bind(&store, equipment(3), &overlay.scope);
    let reservation = store.reserve(&[(lease.clone(), 0, 2)]).unwrap();
    fault(
        store.capture_run(&overlay),
        ResourceFaultCode::ReservationConflict,
    );
    store.commit(&reservation).unwrap();
    let result = store.capture_run(&overlay).unwrap();
    result.apply(&mut root).unwrap();
    let before = root.clone();
    fault(result.apply(&mut root), ResourceFaultCode::StaleRevision);
    assert_eq!(root, before);
    let mut wrong = RunResourceState {
        revision: 7,
        ..Default::default()
    };
    let before = wrong.clone();
    fault(result.apply(&mut wrong), ResourceFaultCode::StaleRevision);
    assert_eq!(wrong, before);
    let mut empty = RunResourceState::default();
    let mut second = super::tests::store();
    let empty_overlay = second.mount_run(&empty).unwrap();
    let empty_result = second.capture_run(&empty_overlay).unwrap();
    empty_result.apply(&mut empty).unwrap();
    fault(
        empty_result.apply(&mut empty),
        ResourceFaultCode::StaleRevision,
    );
}

#[test]
fn durable_missing_fields_unknown_owner_definition_cap_and_duplicates_are_rejected() {
    use crate::game::resource::durable::{RunResourceEntry, RunResourceState};
    let store = store();
    let owners = store.owners.keys().copied().collect();
    let valid = RunResourceState {
        revision: 1,
        closed: false,
        entries: vec![RunResourceEntry {
            resource: id("EquipmentCopy_Run"),
            owner: equipment(3),
            balance: 3,
            revision: 2,
        }],
    };
    valid.validate(&store.catalog, &owners).unwrap();
    for field in ["revision", "closed", "entries"] {
        let mut json = serde_json::to_value(&valid).unwrap();
        json.as_object_mut().unwrap().remove(field);
        assert!(
            serde_json::from_value::<RunResourceState>(json).is_err(),
            "{field}"
        );
    }
    for field in ["resource", "owner", "balance", "revision"] {
        let mut json = serde_json::to_value(&valid).unwrap();
        json["entries"][0].as_object_mut().unwrap().remove(field);
        assert!(
            serde_json::from_value::<RunResourceState>(json).is_err(),
            "{field}"
        );
    }
    for (changed, expected) in [
        (
            RunResourceEntry {
                owner: equipment(999),
                ..valid.entries[0].clone()
            },
            ResourceFaultCode::InvalidOwner,
        ),
        (
            RunResourceEntry {
                owner: employee(1),
                ..valid.entries[0].clone()
            },
            ResourceFaultCode::OwnerKindMismatch,
        ),
        (
            RunResourceEntry {
                resource: id("missing"),
                ..valid.entries[0].clone()
            },
            ResourceFaultCode::UnknownDefinition,
        ),
        (
            RunResourceEntry {
                resource: id("EquipmentCopy_Attempt"),
                ..valid.entries[0].clone()
            },
            ResourceFaultCode::WrongLifetime,
        ),
        (
            RunResourceEntry {
                balance: 11,
                ..valid.entries[0].clone()
            },
            ResourceFaultCode::Invariant,
        ),
    ] {
        let mut invalid = valid.clone();
        invalid.entries[0] = changed;
        fault(invalid.validate(&store.catalog, &owners), expected);
    }
    let mut duplicate = valid.clone();
    duplicate.entries.push(valid.entries[0].clone());
    fault(
        duplicate.validate(&store.catalog, &owners),
        ResourceFaultCode::Invariant,
    );
}

#[test]
fn durable_owner_removal_does_not_refund_or_close_surviving_employee_run_values() {
    use crate::game::resource::durable::RunResourceState;
    let mut root = RunResourceState::default();
    let mut store = store();
    let overlay = store.mount_run(&root).unwrap();
    for owner in [employee(1), equipment(3)] {
        let lease = bind(&store, owner, &overlay.scope);
        store
            .mutate(&lease, 0, ResourceMutation::Consume(2))
            .unwrap();
    }
    store
        .capture_run(&overlay)
        .unwrap()
        .apply(&mut root)
        .unwrap();
    let before = root.clone();
    root.reconcile_owners(&[employee(1), employee(2), equipment(3), equipment(4)].into())
        .unwrap();
    assert_eq!(root, before);
    root.reconcile_owners(&[employee(1), employee(2), equipment(4)].into())
        .unwrap();
    assert_eq!(root.entries.len(), 1);
    assert_eq!(
        (root.entries[0].owner, root.entries[0].balance),
        (employee(1), 3)
    );
    root.close().unwrap();
    let closed = root.clone();
    fault(root.close(), ResourceFaultCode::StaleScope);
    assert_eq!(root, closed);
    fault(
        super::tests::store().mount_run(&root),
        ResourceFaultCode::StaleScope,
    );
}

#[test]
fn resource_fault_is_typed_and_latched_when_a_cleanup_caller_ignores_it() {
    let mut store = store();
    let scope = store.open_scope(ResourceLifetime::Attempt).unwrap();
    let lease = bind(&store, equipment(3), &scope);
    let before = snapshot(&store);
    let result = runtime_control::run(Default::default(), || {
        let _ignored = store
            .mutate(&lease, 0, ResourceMutation::Consume(6))
            .map_err(ResourceError::into_game_error);
        Ok(())
    });
    let error = result.unwrap_err();
    assert_eq!(error.code(), "resource_insufficient");
    assert!(matches!(error, GameError::BattleRuntimeFault(fault)
        if fault.code == runtime_control::RuntimeFaultCode::Resource(ResourceFaultCode::Insufficient)));
    assert_eq!(snapshot(&store), before);
}

#[test]
fn authoring_work_budget_also_prices_resource_walks_and_preserves_values() {
    use crate::authoring::{control, CancellationToken, ExecutionLimits, ExecutionPhase};
    let mut store = store();
    let scope = store.open_scope(ResourceLifetime::Run).unwrap();
    let lease = bind(&store, equipment(3), &scope);
    let before = snapshot(&store);
    let result = control::run(
        &ExecutionLimits {
            max_work: 1,
            ..Default::default()
        },
        &CancellationToken::default(),
        ExecutionPhase::TrialExecution,
        || store.mutate(&lease, 0, ResourceMutation::Consume(1)),
    );
    let error = result.unwrap_err();
    assert_eq!(error.code, "authoring_work_limit");
    assert_eq!(snapshot(&store), before);
}

#[test]
fn retained_byte_bound_includes_sparse_map_nodes_and_rejects_growth_atomically() {
    let mut store = store();
    let scope = store.open_scope(ResourceLifetime::Run).unwrap();
    let lease = bind(&store, equipment(3), &scope);
    let empty_bytes = store.retained_extent().unwrap();
    assert!(
        empty_bytes > std::mem::size_of::<ResourceStore>() + std::mem::size_of::<ResourceValue>()
    );
    store.limits.retained_bytes = empty_bytes;
    let before = snapshot(&store);
    fault(
        store.mutate(&lease, 0, ResourceMutation::Grant(1)),
        ResourceFaultCode::Limit,
    );
    assert_eq!(snapshot(&store), before);
}
