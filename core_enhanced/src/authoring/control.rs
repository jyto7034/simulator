//! Accounting belongs to the synchronous authoring attempt, never to a cloned
//! gameplay authority. Ordinary core calls outside a scope remain uncontrolled.

use crate::game::behavior::GameError;
use serde::Serialize;
use std::{
    cell::RefCell,
    fmt,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

#[derive(Debug, Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

/// Trusted host settings, not deserializable content or scenario authority.
#[derive(Debug, Clone, Serialize)]
pub struct ExecutionLimits {
    pub max_work: u64,
    pub max_pending_events: usize,
    pub max_bucket_events: usize,
    pub max_effect_commands: usize,
    pub max_geometry_cells: usize,
    pub max_retained_entities: usize,
    pub max_definitions: usize,
    pub max_metadata_bytes: usize,
    pub max_result_bytes: usize,
    pub max_clone_slots: usize,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        crate::runtime_safety::RuntimeSafetySettings::embedded().authoring_execution()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPhase {
    Acquisition,
    Parse,
    Resolution,
    FinalValidation,
    GeneratedSetup,
    TrialPreparation,
    TrialExecution,
    Receipt,
    Inspection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExecutionFailure {
    pub code: &'static str,
    pub phase: ExecutionPhase,
    pub site: &'static str,
    pub limit: Option<u64>,
    pub requested: Option<u64>,
}

impl fmt::Display for ExecutionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {} ({:?})", self.code, self.site, self.phase)
    }
}
impl std::error::Error for ExecutionFailure {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ExecutionUsage {
    pub work: u64,
    pub pending_events_peak: usize,
    pub bucket_events_peak: usize,
    pub effect_commands_peak: usize,
    pub geometry_cells_peak: usize,
    pub retained_entities_peak: usize,
    pub definitions_peak: usize,
    pub metadata_bytes_peak: usize,
    pub result_bytes_peak: usize,
    pub clone_slots_peak: usize,
}

impl ExecutionLimits {
    pub(crate) fn validate(&self, phase: ExecutionPhase) -> Result<(), ExecutionFailure> {
        for (site, value, ceiling) in [
            ("max_work", self.max_work, 1_u64 << 30),
            ("max_clone_slots", self.max_clone_slots as u64, 1 << 20),
            (
                "max_pending_events",
                self.max_pending_events as u64,
                1 << 20,
            ),
            ("max_bucket_events", self.max_bucket_events as u64, 1 << 20),
            (
                "max_effect_commands",
                self.max_effect_commands as u64,
                1 << 20,
            ),
            (
                "max_geometry_cells",
                self.max_geometry_cells as u64,
                1 << 20,
            ),
            (
                "max_retained_entities",
                self.max_retained_entities as u64,
                65_536,
            ),
            ("max_definitions", self.max_definitions as u64, 65_536),
            (
                "max_result_bytes",
                self.max_result_bytes as u64,
                64 * 1024 * 1024,
            ),
            (
                "max_metadata_bytes",
                self.max_metadata_bytes as u64,
                64 * 1024 * 1024,
            ),
        ] {
            if value == 0 || value > ceiling {
                return Err(ExecutionFailure {
                    code: "authoring_limits_invalid",
                    phase,
                    site,
                    limit: Some(ceiling),
                    requested: Some(value),
                });
            }
        }
        Ok(())
    }
}

/// Private provenance carried by an opaque trial's complete candidate. This is
/// storage history, not fuel or gameplay authority. One conservative Monolith
/// bound applies separately to every native copy in the same construction
/// family, including captured/restored checkpoints and pending completion plans.
#[derive(Clone, Copy, Default)]
pub(crate) struct NativeStorageProvenance {
    monolith: Option<crate::owned_collections::MapCloneHistory>,
}

impl NativeStorageProvenance {
    /// Only a new opaque session whose game state is constructed inside the
    /// scope has a proven zero. General controlled calls retain unknown history.
    pub(crate) fn fresh() -> Self {
        Self {
            monolith: Some(crate::owned_collections::MapCloneHistory::default()),
        }
    }
}

struct Attempt {
    limits: ExecutionLimits,
    cancellation: CancellationToken,
    phase: ExecutionPhase,
    usage: ExecutionUsage,
    failure: Option<ExecutionFailure>,
    native_storage: NativeStorageProvenance,
}

thread_local! { static ACTIVE: RefCell<Option<Attempt>> = const { RefCell::new(None) }; }

struct ScopeGuard;
impl Drop for ScopeGuard {
    fn drop(&mut self) {
        ACTIVE.with(|slot| {
            slot.borrow_mut().take();
        });
    }
}

/// No borrow or callback escapes the accounting helpers. Nested top-level
/// attempts cannot replace the active budget, even when the nested error is ignored.
pub(crate) fn run<T>(
    limits: &ExecutionLimits,
    cancellation: &CancellationToken,
    phase: ExecutionPhase,
    operation: impl FnOnce() -> T,
) -> Result<(T, ExecutionUsage), ExecutionFailure> {
    run_with_provenance(
        limits,
        cancellation,
        phase,
        NativeStorageProvenance::default(),
        operation,
    )
    .map(|(value, usage, _)| (value, usage))
}

/// Returning a candidate ledger does not commit it. In particular, T may itself
/// be a domain Result::Err. The session owner must retain it only with successful
/// domain state, mandatory receipt and the final acceptance poll, as one swap.
pub(crate) fn run_with_provenance<T>(
    limits: &ExecutionLimits,
    cancellation: &CancellationToken,
    phase: ExecutionPhase,
    native_storage: NativeStorageProvenance,
    operation: impl FnOnce() -> T,
) -> Result<(T, ExecutionUsage, NativeStorageProvenance), ExecutionFailure> {
    limits.validate(phase)?;
    ACTIVE.with(|slot| {
        let mut slot = slot.borrow_mut();
        if slot.is_some() {
            return Err(ExecutionFailure {
                code: "authoring_nested_attempt",
                phase,
                site: "attempt_entry",
                limit: None,
                requested: None,
            });
        }
        *slot = Some(Attempt {
            limits: limits.clone(),
            cancellation: cancellation.clone(),
            phase,
            usage: ExecutionUsage::default(),
            failure: None,
            native_storage,
        });
        Ok(())
    })?;
    let _guard = ScopeGuard;
    inspect("attempt_entry", |_| Ok(()))?;
    let value = operation();
    inspect("attempt_acceptance", |_| Ok(()))?;
    let (usage, native_storage) = ACTIVE.with(|slot| {
        let slot = slot.borrow();
        let attempt = slot.as_ref().expect("live scope");
        (attempt.usage.clone(), attempt.native_storage)
    });
    Ok((value, usage, native_storage))
}

/// Actual insert hook; no gameplay API/error/ordering changes and no table scan.
/// Even an internally rolled back insertion keeps credit within this candidate.
pub(crate) fn monolith_inserted() {
    ACTIVE.with(|slot| {
        if let Some(attempt) = slot.borrow_mut().as_mut() {
            if let Some(history) = attempt.native_storage.monolith.as_mut() {
                history.inserted();
            }
        }
    });
}

/// Only the closed fresh checkpoint parser may seed this raw table's capacity. Arbitrary
/// existing GameCore/RunState/table imports are not part of the opaque family.
pub(crate) fn monolith_imported_fresh(capacity: usize) {
    ACTIVE.with(|slot| {
        if let Some(attempt) = slot.borrow_mut().as_mut() {
            if let Some(history) = attempt.native_storage.monolith.as_mut() {
                history.include_tombstone_free_capacity(capacity);
            }
        }
    });
}

pub(crate) fn monolith_clone_bucket_bound() -> Option<usize> {
    ACTIVE.with(|slot| {
        slot.borrow()
            .as_ref()?
            .native_storage
            .monolith?
            .bucket_bound()
    })
}

pub(crate) struct PhaseGuard(Option<ExecutionPhase>);
impl Drop for PhaseGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.0 {
            ACTIVE.with(|slot| {
                if let Some(attempt) = slot.borrow_mut().as_mut() {
                    attempt.phase = previous;
                }
            });
        }
    }
}
pub(crate) fn phase(next: ExecutionPhase) -> PhaseGuard {
    PhaseGuard(ACTIVE.with(|slot| {
        slot.borrow_mut()
            .as_mut()
            .map(|attempt| std::mem::replace(&mut attempt.phase, next))
    }))
}

fn inspect(
    site: &'static str,
    work: impl FnOnce(&mut Attempt) -> Result<(), (&'static str, u64, u64)>,
) -> Result<(), ExecutionFailure> {
    #[cfg(test)]
    test_cancellation_site(site);
    ACTIVE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(attempt) = slot.as_mut() else {
            return Ok(());
        };
        if let Some(failure) = &attempt.failure {
            return Err(failure.clone());
        }
        let failure = if attempt.cancellation.is_cancelled() {
            Some(ExecutionFailure {
                code: "authoring_cancelled",
                phase: attempt.phase,
                site,
                limit: None,
                requested: None,
            })
        } else {
            work(attempt)
                .err()
                .map(|(code, limit, requested)| ExecutionFailure {
                    code,
                    phase: attempt.phase,
                    site,
                    limit: Some(limit),
                    requested: Some(requested),
                })
        };
        match failure {
            Some(failure) => {
                attempt.failure = Some(failure.clone());
                Err(failure)
            }
            None => Ok(()),
        }
    })
}

fn domain(failure: ExecutionFailure) -> GameError {
    GameError::InvalidStaticData(failure.to_string())
}

pub(crate) fn poll(site: &'static str) -> Result<(), GameError> {
    inspect(site, |_| Ok(())).map_err(domain)?;
    crate::game::battle::runtime_control::poll()
}

/// Infallible application error adapters use this before formatting a second
/// diagnostic. The outer scope still owns and returns the typed first failure.
pub(crate) fn interruption(site: &'static str) -> Option<ExecutionFailure> {
    inspect(site, |_| Ok(())).err()
}

/// Latches a checked runtime invariant failure at an infallible producer seam.
/// A disabled/general caller still receives an explicit domain error.
pub(crate) fn reject(code: &'static str, site: &'static str) -> GameError {
    if let Some(failure) = interruption(site) {
        return domain(failure);
    }
    let failure = ACTIVE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let attempt = slot.as_mut()?;
        let failure = ExecutionFailure {
            code,
            phase: attempt.phase,
            site,
            limit: None,
            requested: None,
        };
        attempt.failure = Some(failure.clone());
        Some(failure)
    });
    if let Some(failure) = failure {
        return domain(failure);
    }
    if crate::game::battle::runtime_control::is_active() {
        return crate::game::battle::runtime_control::reject(
            crate::game::battle::runtime_control::BattleRuntimeFault {
                code: crate::game::battle::runtime_control::RuntimeFaultCode::Invariant,
                site,
                limit: None,
                requested: None,
            },
        );
    }
    GameError::InvalidStaticData(code.into())
}

pub(crate) fn is_active() -> bool {
    ACTIVE.with(|slot| slot.borrow().is_some())
}

/// Read-only headroom for choosing a reservation before its first admission.
/// The eventual charge remains authoritative and may still fail on cancellation.
pub(crate) fn growth_work_allowance() -> Option<u64> {
    ACTIVE.with(|slot| {
        slot.borrow()
            .as_ref()
            .map(|attempt| attempt.limits.max_work.saturating_sub(attempt.usage.work))
    })
}

pub(crate) fn container_capacity_limit(resource: Resource) -> Option<usize> {
    ACTIVE.with(|slot| {
        slot.borrow().as_ref().and_then(|attempt| match resource {
            Resource::PendingEvents => Some(attempt.limits.max_pending_events),
            Resource::BucketEvents => Some(attempt.limits.max_bucket_events),
            _ => None,
        })
    })
}

/// Accumulates one actual copy's independently owned storage and strings. It
/// owns no gameplay references or attempt budget; nested owners contribute to
/// the same meter. Callers admit/charge a container scan before visiting it and
/// finish before the native clone. Unknown bounds can never become zero cost.
#[derive(Default)]
pub(crate) struct CloneMeter {
    slots: usize,
    string_bytes: usize,
    native_scan: Option<crate::game::battle::runtime_control::CloneScanAllowance>,
}

impl CloneMeter {
    pub(crate) fn for_native_scan() -> Result<Self, GameError> {
        Ok(Self {
            native_scan: Some(crate::game::battle::runtime_control::clone_scan_allowance()?),
            ..Default::default()
        })
    }

    /// Native accounting shares the ownership visitor, while keeping its fuel
    /// outside the authoring budget and gameplay snapshots.
    pub(crate) fn native_scan_slots(&self) -> usize {
        self.slots
    }

    pub(crate) fn native_copy_slots(&self) -> Option<usize> {
        self.slots.checked_add(self.string_bytes)
    }

    /// Admit native storage before walking it. The structural scan is distinct
    /// from the actual copy charged by finish; overflow is a rejected bound.
    pub(crate) fn scan_storage(&mut self, slots: Option<usize>) -> Result<(), GameError> {
        self.add_storage(slots);
        self.admit_storage("owned_clone_storage")?;
        if self.native_scan.is_some() {
            Ok(())
        } else {
            charge(slots.unwrap_or(usize::MAX) as u64, "owned_clone_scan")
        }
    }

    pub(crate) fn scan_string(&mut self, value: &str) -> Result<(), GameError> {
        self.scan_storage(Some(1))?;
        self.add_string_bytes(value.len());
        if self.native_scan.is_some() {
            self.admit_storage("owned_clone_string")?;
        }
        Ok(())
    }

    pub(crate) fn add_storage(&mut self, slots: Option<usize>) {
        self.slots = slots
            .and_then(|slots| self.slots.checked_add(slots))
            .unwrap_or(usize::MAX);
    }

    pub(crate) fn add_string_bytes(&mut self, bytes: usize) {
        self.string_bytes = self.string_bytes.saturating_add(bytes);
    }

    pub(crate) fn admit_storage(&self, site: &'static str) -> Result<(), GameError> {
        if let Some(allowance) = self.native_scan {
            allowance.admit(self.slots, self.string_bytes)
        } else {
            admit(Resource::CloneSlots, 0, self.slots, site)
        }
    }

    pub(crate) fn finish(
        self,
        storage_site: &'static str,
        copy_site: &'static str,
    ) -> Result<(), GameError> {
        self.admit_storage(storage_site)?;
        charge(
            (self.slots as u64).saturating_add(self.string_bytes as u64),
            copy_site,
        )
    }
}

/// Fixed scalar counters only. Receipt preparation calls this after reserving
/// the work for filling its fixed-width header, before the final acceptance poll.
pub(super) fn usage_snapshot() -> Option<ExecutionUsage> {
    ACTIVE.with(|slot| slot.borrow().as_ref().map(|attempt| attempt.usage.clone()))
}

/// Count encoded metadata without allocating the serialized document. The
/// writer admits every chunk before counting it, independently of trace output.
pub(crate) fn metadata_size(value: &impl Serialize) -> Result<(), GameError> {
    if !is_active() {
        return Ok(());
    }
    struct Counter(usize);
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            admit(
                Resource::MetadataBytes,
                self.0,
                bytes.len(),
                "source_metadata",
            )
            .and_then(|()| charge(bytes.len() as u64, "source_metadata_encode"))
            .map_err(|error| std::io::Error::other(format!("{error:?}")))?;
            self.0 += bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Counter(0), value)
        .map_err(|error| GameError::InvalidStaticData(error.to_string()))
}

/// Price a typed payload before cloning it, without retaining an encoded copy.
/// This is deterministic encoded-shape work, not an allocator byte measurement.
pub(crate) fn charge_serialized(
    value: &impl Serialize,
    site: &'static str,
) -> Result<(), GameError> {
    if !is_active() && !crate::game::battle::runtime_control::is_active() {
        return Ok(());
    }
    struct Counter(&'static str);
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            charge(bytes.len() as u64, self.0)
                .map_err(|_| std::io::Error::from(std::io::ErrorKind::Other))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let result = serde_json::to_writer(Counter(site), value);
    poll(site)?;
    result.map_err(|error| GameError::InvalidStaticData(error.to_string()))
}

pub(crate) fn charge(units: u64, site: &'static str) -> Result<(), GameError> {
    inspect(site, |attempt| {
        let requested = attempt.usage.work.checked_add(units).ok_or((
            "authoring_work_limit",
            attempt.limits.max_work,
            u64::MAX,
        ))?;
        if requested > attempt.limits.max_work {
            return Err(("authoring_work_limit", attempt.limits.max_work, requested));
        }
        attempt.usage.work = requested;
        Ok(())
    })
    .map_err(domain)?;
    crate::game::battle::runtime_control::charge(units, site)
}

#[derive(Clone, Copy)]
pub(crate) enum Resource {
    PendingEvents,
    BucketEvents,
    EffectCommands,
    GeometryCells,
    RetainedEntities,
    Definitions,
    MetadataBytes,
    ResultBytes,
    CloneSlots,
}

pub(crate) fn admit(
    resource: Resource,
    current: usize,
    additional: usize,
    site: &'static str,
) -> Result<(), GameError> {
    inspect(site, |attempt| {
        let (code, limit, peak) = match resource {
            Resource::PendingEvents => (
                "authoring_queue_limit",
                attempt.limits.max_pending_events,
                &mut attempt.usage.pending_events_peak,
            ),
            Resource::BucketEvents => (
                "authoring_bucket_limit",
                attempt.limits.max_bucket_events,
                &mut attempt.usage.bucket_events_peak,
            ),
            Resource::EffectCommands => (
                "authoring_effect_command_limit",
                attempt.limits.max_effect_commands,
                &mut attempt.usage.effect_commands_peak,
            ),
            Resource::GeometryCells => (
                "authoring_geometry_limit",
                attempt.limits.max_geometry_cells,
                &mut attempt.usage.geometry_cells_peak,
            ),
            Resource::RetainedEntities => (
                "authoring_entity_limit",
                attempt.limits.max_retained_entities,
                &mut attempt.usage.retained_entities_peak,
            ),
            Resource::Definitions => (
                "authoring_definition_limit",
                attempt.limits.max_definitions,
                &mut attempt.usage.definitions_peak,
            ),
            Resource::MetadataBytes => (
                "authoring_metadata_limit",
                attempt.limits.max_metadata_bytes,
                &mut attempt.usage.metadata_bytes_peak,
            ),
            Resource::ResultBytes => (
                "authoring_result_limit",
                attempt.limits.max_result_bytes,
                &mut attempt.usage.result_bytes_peak,
            ),
            Resource::CloneSlots => (
                "authoring_clone_limit",
                attempt.limits.max_clone_slots,
                &mut attempt.usage.clone_slots_peak,
            ),
        };
        let requested = current
            .checked_add(additional)
            .ok_or((code, limit as u64, u64::MAX))?;
        if requested > limit {
            return Err((code, limit as u64, requested as u64));
        }
        *peak = (*peak).max(requested);
        Ok(())
    })
    .map_err(domain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuel_and_poison_survive_discarded_inner_state_and_swallowed_errors() {
        let limits = ExecutionLimits {
            max_work: 3,
            ..Default::default()
        };
        let failure = run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || {
                let before = vec![1];
                let mut candidate = before.clone();
                charge(2, "first").unwrap();
                candidate.push(2);
                assert!(charge(2, "failed_inner").is_err());
                candidate = before;
                assert_eq!(candidate, vec![1]);
                assert!(charge(1, "after_rollback").is_err());
                "domain normalized failure into success"
            },
        )
        .unwrap_err();
        assert_eq!(failure.code, "authoring_work_limit");
        assert_eq!(failure.site, "failed_inner");
        let (_, usage) = run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::TrialExecution,
            || charge(3, "exact").unwrap(),
        )
        .unwrap();
        assert_eq!(usage.work, 3);
    }

    #[test]
    fn nested_attempt_cannot_reset_accounting_and_cancel_restores_thread_scope() {
        let limits = ExecutionLimits {
            max_work: 2,
            ..Default::default()
        };
        let token = CancellationToken::default();
        let failure = run(&limits, &token, ExecutionPhase::Acquisition, || {
            charge(1, "outer").unwrap();
            let nested = run(
                &ExecutionLimits::default(),
                &token,
                ExecutionPhase::Receipt,
                || panic!("nested body"),
            );
            assert_eq!(nested.unwrap_err().code, "authoring_nested_attempt");
            token.cancel();
        })
        .unwrap_err();
        assert_eq!(failure.code, "authoring_cancelled");
        assert!(poll("ordinary_call_after_cancel").is_ok());
        let (_, usage) = run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::Acquisition,
            || charge(2, "new_attempt").unwrap(),
        )
        .unwrap();
        assert_eq!(usage.work, 2);
    }

    #[test]
    fn resource_caps_are_inclusive_and_phases_restore_before_acceptance() {
        let limits = ExecutionLimits {
            max_pending_events: 2,
            ..Default::default()
        };
        let failure = run(
            &limits,
            &CancellationToken::default(),
            ExecutionPhase::Acquisition,
            || {
                let _phase = phase(ExecutionPhase::GeneratedSetup);
                admit(Resource::PendingEvents, 1, 1, "exact").unwrap();
                assert!(admit(Resource::PendingEvents, 2, 1, "over").is_err());
            },
        )
        .unwrap_err();
        assert_eq!(failure.phase, ExecutionPhase::GeneratedSetup);
        assert_eq!(failure.requested, Some(3));
        let invalid = ExecutionLimits {
            max_work: 0,
            ..Default::default()
        };
        assert_eq!(
            run(
                &invalid,
                &CancellationToken::default(),
                ExecutionPhase::Acquisition,
                || ()
            )
            .unwrap_err()
            .code,
            "authoring_limits_invalid"
        );
    }
}
// Deterministic same-thread scheduling seam for cancellation acceptance tests.
// It cancels the real attempt token; normal admission and rollback remain active.
#[cfg(test)]
thread_local! { static CANCEL_SITE: RefCell<Option<(&'static str, usize)>> = const { RefCell::new(None) }; }
#[cfg(test)]
pub(crate) fn with_test_cancellation<T>(
    site: &'static str,
    occurrence: usize,
    operation: impl FnOnce() -> T,
) -> T {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            CANCEL_SITE.with(|slot| {
                slot.borrow_mut().take();
            });
        }
    }
    assert!(occurrence > 0);
    CANCEL_SITE.with(|slot| {
        assert!(slot.borrow().is_none());
        *slot.borrow_mut() = Some((site, occurrence));
    });
    let _reset = Reset;
    operation()
}
#[cfg(test)]
pub(crate) fn test_cancellation_site(site: &'static str) {
    let cancel = CANCEL_SITE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some((expected, left)) = slot.as_mut() else {
            return false;
        };
        if *expected != site {
            return false;
        }
        *left -= 1;
        if *left == 0 {
            slot.take();
            true
        } else {
            false
        }
    });
    if cancel {
        ACTIVE.with(|slot| {
            if let Some(attempt) = slot.borrow().as_ref() {
                attempt.cancellation.cancel();
            }
        });
    }
}
