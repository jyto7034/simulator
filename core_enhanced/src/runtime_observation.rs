//! Optional, bounded observations. These values never admit or reject gameplay.
//! The host owns collection, metrics, selection and presentation. Identity is
//! metadata; it must never become a metrics label.
use crate::{
    application::{ApplicationCommandKind, ApplicationError, ApplicationErrorKind},
    game::battle::runtime_control::{ResourceFaultCode, RuntimeFaultCode},
};
use serde::Serialize;
use std::{
    cell::RefCell,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender},
        Arc, Weak,
    },
    time::Instant,
};
use uuid::Uuid;

mod retained;
pub use retained::{RetainedAccounting, RetainedExceed, RetainedRequestKind};
mod identity;
pub(crate) use identity::digest_bytes;
use identity::{decode_identity, encode_identity, IDENTITY_WORDS};
pub use identity::{MeasurementIdentity, NativeOwnerExtent};

pub const QUEUE_CAPACITY: usize = 1024;
pub const WORKER_BATCH: usize = 128;
pub const USAGE_FIELDS: usize = 9;
pub const USAGE_NAMES: [&str; USAGE_FIELDS] = [
    "accounted_extent_bytes",
    "pending_events",
    "active_actions",
    "projectiles",
    "units",
    "history_events",
    "history_accounted_encoded_bytes",
    "history_span_ms",
    "simulation_time_ms",
];
pub(crate) const EXTENT: usize = 0;
pub(crate) const EVENTS: usize = 1;
pub(crate) const ACTIONS: usize = 2;
pub(crate) const PROJECTILES: usize = 3;
pub(crate) const UNITS: usize = 4;
pub(crate) const HISTORY_EVENTS: usize = 5;
pub(crate) const HISTORY_BYTES: usize = 6;
pub(crate) const HISTORY_SPAN: usize = 7;

/// Counts and accounted extents, never process RSS or exact heap allocations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct Usage(pub [u64; USAGE_FIELDS]);
impl Usage {
    pub(crate) fn include(&mut self, other: Self) {
        for (a, b) in self.0.iter_mut().zip(other.0) {
            *a = (*a).max(b);
        }
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct CommittedSnapshot {
    pub battle: Option<Uuid>,
    pub usage: Usage,
    pub owner_extent: NativeOwnerExtent,
    pub identity: Option<MeasurementIdentity>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Operation {
    Command(ApplicationCommandKind),
    Elapsed,
    HostTick,
    NextEvent,
    DisconnectPause,
    Admin,
    ContinueCheckpoint,
    MergeProfile,
    MergeCompendium,
    ResetProfile,
    RestoreRollback,
}
pub const OPERATION_COUNT: usize = ApplicationCommandKind::ALL.len() + 10;
impl Operation {
    pub fn index(self) -> usize {
        ApplicationCommandKind::ALL.len()
            + match self {
                Self::Command(kind) => return kind as usize,
                Self::Elapsed => 0,
                Self::HostTick => 1,
                Self::NextEvent => 2,
                Self::DisconnectPause => 3,
                Self::Admin => 4,
                Self::ContinueCheckpoint => 5,
                Self::MergeProfile => 6,
                Self::MergeCompendium => 7,
                Self::ResetProfile => 8,
                Self::RestoreRollback => 9,
            }
    }
    /// Bootstrap-only finite registration set. Never called by the game hot path.
    pub fn labels() -> Vec<(Self, String)> {
        let mut values: Vec<_> = ApplicationCommandKind::ALL
            .iter()
            .map(|kind| {
                (
                    Self::Command(*kind),
                    serde_json::to_string(kind)
                        .expect("enum")
                        .trim_matches('"')
                        .to_owned(),
                )
            })
            .collect();
        values.extend(
            [
                (Self::Elapsed, "elapsed"),
                (Self::HostTick, "host_tick"),
                (Self::NextEvent, "next_event"),
                (Self::DisconnectPause, "disconnect_pause"),
                (Self::Admin, "admin"),
                (Self::ContinueCheckpoint, "continue_checkpoint"),
                (Self::MergeProfile, "merge_profile"),
                (Self::MergeCompendium, "merge_compendium"),
                (Self::ResetProfile, "reset_profile"),
                (Self::RestoreRollback, "restore_rollback"),
            ]
            .map(|(op, label)| (op, label.into())),
        );
        values
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Outcome {
    Accepted,
    Rejected,
    ReadOnly,
    Discarded,
}
impl Outcome {
    pub const ALL: [Self; 4] = [
        Self::Accepted,
        Self::Rejected,
        Self::ReadOnly,
        Self::Discarded,
    ];
    pub const fn label(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::ReadOnly => "read_only",
            Self::Discarded => "discarded",
        }
    }
}
const NATIVE_REASONS: &[RuntimeFaultCode] = &[
    RuntimeFaultCode::InvalidLimits,
    RuntimeFaultCode::WorkLimit,
    RuntimeFaultCode::CloneLimit,
    RuntimeFaultCode::ActiveActionLimit,
    RuntimeFaultCode::RetainedResultLimit,
    RuntimeFaultCode::DeliveryLimit,
    RuntimeFaultCode::ActorLimit,
    RuntimeFaultCode::ProviderLimit,
    RuntimeFaultCode::ControlStepLimit,
    RuntimeFaultCode::StrikeSegmentLimit,
    RuntimeFaultCode::PredeathBuffLimit,
    RuntimeFaultCode::RetainedByteLimit,
    RuntimeFaultCode::CounterOverflow,
    RuntimeFaultCode::Invariant,
    RuntimeFaultCode::HealthCostUnavailable,
    RuntimeFaultCode::ShieldUnavailable,
    RuntimeFaultCode::InvalidControlTarget,
    RuntimeFaultCode::Resource(ResourceFaultCode::UnknownDefinition),
    RuntimeFaultCode::Resource(ResourceFaultCode::InvalidOwner),
    RuntimeFaultCode::Resource(ResourceFaultCode::OwnerKindMismatch),
    RuntimeFaultCode::Resource(ResourceFaultCode::StaleScope),
    RuntimeFaultCode::Resource(ResourceFaultCode::WrongLifetime),
    RuntimeFaultCode::Resource(ResourceFaultCode::StaleLease),
    RuntimeFaultCode::Resource(ResourceFaultCode::StaleRevision),
    RuntimeFaultCode::Resource(ResourceFaultCode::StaleReservation),
    RuntimeFaultCode::Resource(ResourceFaultCode::Insufficient),
    RuntimeFaultCode::Resource(ResourceFaultCode::ReservationConflict),
    RuntimeFaultCode::Resource(ResourceFaultCode::Limit),
    RuntimeFaultCode::Resource(ResourceFaultCode::CounterOverflow),
    RuntimeFaultCode::Resource(ResourceFaultCode::Invariant),
];
const APP_REASONS: &[ApplicationErrorKind] = &[
    ApplicationErrorKind::InvalidCommand,
    ApplicationErrorKind::InvalidAdminCommand,
    ApplicationErrorKind::InvalidProfile,
    ApplicationErrorKind::ProjectionFailure,
];
pub const REASON_COUNT: usize =
    NATIVE_REASONS.len() + ApplicationErrorKind::DOMAIN_ERRORS.len() + APP_REASONS.len();
/// Index into the closed registration set returned by `reason_labels`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Reason(pub usize);
pub fn reason_labels() -> Vec<String> {
    NATIVE_REASONS
        .iter()
        .map(|code| code.as_str().into())
        .chain(
            ApplicationErrorKind::DOMAIN_ERRORS
                .iter()
                .chain(APP_REASONS)
                .map(|kind| {
                    serde_json::to_string(kind)
                        .expect("enum")
                        .trim_matches('"')
                        .to_owned()
                }),
        )
        .collect()
}
fn reason(error: &ApplicationError) -> Reason {
    if error.kind() == ApplicationErrorKind::BattleRuntimeFault {
        if let Some(index) = NATIVE_REASONS
            .iter()
            .position(|code| code.as_str() == error.code())
        {
            return Reason(index);
        }
    }
    Reason(
        NATIVE_REASONS.len()
            + ApplicationErrorKind::DOMAIN_ERRORS
                .iter()
                .chain(APP_REASONS)
                .position(|kind| *kind == error.kind())
                .expect("closed public error enum"),
    )
}
#[derive(Debug, Clone, Copy, Serialize)]
pub struct OperationSummary {
    pub session: u64,
    pub sequence: u64,
    pub operation: Operation,
    pub outcome: Outcome,
    pub reason: Option<Reason>,
    pub wall_seconds: f64,
    pub work: u64,
    pub clone_slots: u64,
    /// Historical allowance: a refused initial seed is included. See `retained`
    /// for strictly accepted bytes and attempted requests.
    pub admitted_bytes: u64,
    /// None means no native operation scope was measured, not zero allocation.
    pub retained: Option<RetainedAccounting>,
    /// Maximum successful whole-owner scan, separate from admission requests.
    pub max_battle_owner_extent_bytes: Option<u64>,
    pub max_pending_owner_extent_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_relic_owner_extent_bytes: Option<u64>,
    pub started_identity: Option<MeasurementIdentity>,
    /// Prepared candidate metadata is distinct from the authoritative commit.
    pub prepared: Option<CommittedSnapshot>,
    pub peak: Usage,
    pub committed: Option<CommittedSnapshot>,
}
struct Totals {
    attempts: [AtomicU64; OPERATION_COUNT * 4],
    reasons: [AtomicU64; REASON_COUNT],
    commits: AtomicU64,
    dropped: AtomicU64,
}
impl Default for Totals {
    fn default() -> Self {
        Self {
            attempts: std::array::from_fn(|_| AtomicU64::new(0)),
            reasons: std::array::from_fn(|_| AtomicU64::new(0)),
            commits: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
        }
    }
}
#[derive(Clone)]
pub struct ObservationSender {
    sender: SyncSender<OperationSummary>,
    totals: Arc<Totals>,
}
pub struct ObservationReceiver {
    receiver: Receiver<OperationSummary>,
    totals: Arc<Totals>,
}
#[derive(Debug)]
pub struct CounterSnapshot {
    pub attempts: [u64; OPERATION_COUNT * 4],
    pub reasons: [u64; REASON_COUNT],
    pub commits: u64,
    pub dropped: u64,
}
pub fn channel() -> (ObservationSender, ObservationReceiver) {
    let (sender, receiver) = mpsc::sync_channel(QUEUE_CAPACITY);
    let totals = Arc::new(Totals::default());
    (
        ObservationSender {
            sender,
            totals: totals.clone(),
        },
        ObservationReceiver { receiver, totals },
    )
}
impl ObservationReceiver {
    pub fn try_recv(&self) -> Result<OperationSummary, mpsc::TryRecvError> {
        self.receiver.try_recv()
    }
    pub fn counters(&self) -> CounterSnapshot {
        CounterSnapshot {
            attempts: std::array::from_fn(|i| self.totals.attempts[i].load(Ordering::Relaxed)),
            reasons: std::array::from_fn(|i| self.totals.reasons[i].load(Ordering::Relaxed)),
            commits: self.totals.commits.load(Ordering::Relaxed),
            dropped: self.totals.dropped.load(Ordering::Relaxed),
        }
    }
}
fn increment(value: &AtomicU64) {
    let _ = value.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
        Some(old.saturating_add(1))
    });
}
struct SessionState {
    id: u64,
    version: AtomicU64,
    commit_count: AtomicU64,
    // battle-present, UUID high/low, then Usage. A single writer owns the lease.
    latest: [AtomicU64; USAGE_FIELDS + 3],
    owner_extent: [AtomicU64; 4],
    identity: [AtomicU64; IDENTITY_WORDS],
    peaks: [AtomicU64; USAGE_FIELDS],
}
/// Single-writer capability. Attach it to one CoreApplication; never clone it.
pub struct SessionObserver {
    state: Arc<SessionState>,
    sender: ObservationSender,
    sequence: u64,
}
#[derive(Clone)]
pub struct SessionReader {
    state: Weak<SessionState>,
    id: u64,
}
#[derive(Debug, Clone, Copy, Serialize)]
pub struct SessionSnapshot {
    pub session: u64,
    pub commits: u64,
    pub current: CommittedSnapshot,
    pub interval_peak: Usage,
}
impl ObservationSender {
    /// Host-provided identity is metadata, not a metric label. Registration must
    /// use a unique identity for each simultaneously or previously queued lease.
    pub fn session(&self, id: u64) -> (SessionObserver, SessionReader) {
        let state = Arc::new(SessionState {
            id,
            version: AtomicU64::new(0),
            commit_count: AtomicU64::new(0),
            latest: std::array::from_fn(|_| AtomicU64::new(0)),
            owner_extent: std::array::from_fn(|_| AtomicU64::new(0)),
            identity: std::array::from_fn(|_| AtomicU64::new(0)),
            peaks: std::array::from_fn(|_| AtomicU64::new(0)),
        });
        let reader = SessionReader {
            state: Arc::downgrade(&state),
            id,
        };
        (
            SessionObserver {
                state,
                sender: self.clone(),
                sequence: 0,
            },
            reader,
        )
    }
}
impl SessionReader {
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn is_alive(&self) -> bool {
        self.state.strong_count() != 0
    }
    /// Bounded retries, no game lock. Peak reset is only performed by the host's
    /// one collector; diagnostic readers must use `take_peaks=false`.
    pub fn snapshot(&self, take_peaks: bool) -> Option<SessionSnapshot> {
        let state = self.state.upgrade()?;
        for _ in 0..3 {
            let before = state.version.load(Ordering::Acquire);
            if before & 1 != 0 {
                continue;
            }
            let fields: [u64; USAGE_FIELDS + 3] =
                std::array::from_fn(|i| state.latest[i].load(Ordering::Relaxed));
            let identity = std::array::from_fn(|i| state.identity[i].load(Ordering::Relaxed));
            let owner_extent =
                std::array::from_fn(|i| state.owner_extent[i].load(Ordering::Relaxed));
            let commits = state.commit_count.load(Ordering::Relaxed);
            std::sync::atomic::fence(Ordering::Acquire);
            if before != state.version.load(Ordering::Relaxed) {
                continue;
            }
            let battle = (fields[0] != 0)
                .then(|| Uuid::from_u128(((fields[1] as u128) << 64) | fields[2] as u128));
            let current = CommittedSnapshot {
                battle,
                usage: Usage(std::array::from_fn(|i| fields[i + 3])),
                owner_extent: NativeOwnerExtent::decode(owner_extent),
                identity: decode_identity(identity),
            };
            let mut peak = Usage(std::array::from_fn(|i| {
                if take_peaks {
                    state.peaks[i].swap(0, Ordering::Relaxed)
                } else {
                    state.peaks[i].load(Ordering::Relaxed)
                }
            }));
            peak.include(current.usage);
            return Some(SessionSnapshot {
                session: state.id,
                commits,
                current,
                interval_peak: peak,
            });
        }
        None
    }
}
impl SessionState {
    fn publish(&self, current: CommittedSnapshot) {
        self.version.fetch_add(1, Ordering::AcqRel);
        let uuid = current.battle.unwrap_or(Uuid::nil()).as_u128();
        self.latest[0].store(u64::from(current.battle.is_some()), Ordering::Relaxed);
        self.latest[1].store((uuid >> 64) as u64, Ordering::Relaxed);
        self.latest[2].store(uuid as u64, Ordering::Relaxed);
        for (dest, value) in self.latest[3..].iter().zip(current.usage.0) {
            dest.store(value, Ordering::Relaxed);
        }
        for (dest, value) in self.owner_extent.iter().zip(current.owner_extent.encode()) {
            dest.store(value, Ordering::Relaxed);
        }
        for (dest, value) in self.identity.iter().zip(encode_identity(current.identity)) {
            dest.store(value, Ordering::Relaxed);
        }
        increment(&self.commit_count);
        self.version.fetch_add(1, Ordering::Release);
    }
}
#[derive(Default)]
struct Accumulator {
    root_address: usize,
    started_identity: Option<MeasurementIdentity>,
    publisher: Option<(Arc<SessionState>, Arc<Totals>)>,
    work: u64,
    clones: u64,
    admitted: u64,
    retained: Option<RetainedAccounting>,
    peak: Usage,
    max_battle_extent: Option<u64>,
    max_pending_extent: Option<u64>,
    max_relic_extent: Option<u64>,
    staged: Option<CommittedSnapshot>,
    committed: Option<CommittedSnapshot>,
}
thread_local! {static ACTIVE: RefCell<Option<Accumulator>> = const { RefCell::new(None) };}
/// Owns only observation state; never a draft or a safety allowance.
pub(crate) struct OperationGuard {
    state: Arc<SessionState>,
    sender: ObservationSender,
    sequence: u64,
    operation: Operation,
    start: Instant,
    active: bool,
}
impl SessionObserver {
    pub(crate) fn begin_with_identity(
        &mut self,
        operation: Operation,
        root_address: usize,
        identity: MeasurementIdentity,
    ) -> Option<OperationGuard> {
        let guard = self.begin(operation, root_address)?;
        ACTIVE.with(|slot| {
            if let Some(summary) = slot.borrow_mut().as_mut() {
                summary.started_identity = Some(identity);
            }
        });
        Some(guard)
    }
    /// Terminal application failure retires only active gauges. The lease, first
    /// rejected operation, peaks and commit counters retain their real history.
    pub(crate) fn end_battle(&self) {
        self.state.version.fetch_add(1, Ordering::AcqRel);
        for value in &self.state.latest {
            value.store(0, Ordering::Relaxed);
        }
        self.state.version.fetch_add(1, Ordering::Release);
    }

    pub(crate) fn begin(
        &mut self,
        operation: Operation,
        root_address: usize,
    ) -> Option<OperationGuard> {
        if self.sequence == u64::MAX {
            return None;
        }
        let installed = ACTIVE.with(|s| {
            let mut s = s.borrow_mut();
            if s.is_some() {
                false
            } else {
                *s = Some(Accumulator {
                    root_address,
                    publisher: Some((self.state.clone(), self.sender.totals.clone())),
                    ..Default::default()
                });
                true
            }
        });
        if !installed {
            return None;
        }
        self.sequence += 1;
        Some(OperationGuard {
            state: self.state.clone(),
            sender: self.sender.clone(),
            sequence: self.sequence,
            operation,
            start: Instant::now(),
            active: true,
        })
    }
}
impl OperationGuard {
    fn detach(mut self) -> PendingObservation {
        let summary = ACTIVE.with(|s| s.borrow_mut().take().expect("observation guard owner"));
        self.active = false;
        PendingObservation {
            state: self.state.clone(),
            sender: self.sender.clone(),
            sequence: self.sequence,
            operation: self.operation,
            wall_seconds: self.start.elapsed().as_secs_f64(),
            summary: Some(summary),
        }
    }
    pub(crate) fn finish<T>(self, result: &Result<T, ApplicationError>) {
        let pending = self.detach();
        pending.finish(result.as_ref().err());
    }
    pub(crate) fn prepared(self) -> PendingObservation {
        self.detach()
    }
    pub(crate) fn finish_error(self, error: &ApplicationError) {
        self.detach().finish(Some(error));
    }
}
impl Drop for OperationGuard {
    fn drop(&mut self) {
        if self.active {
            ACTIVE.with(|s| {
                s.borrow_mut().take();
            });
        }
    }
}
pub(crate) struct PendingObservation {
    state: Arc<SessionState>,
    sender: ObservationSender,
    sequence: u64,
    operation: Operation,
    wall_seconds: f64,
    summary: Option<Accumulator>,
}
impl PendingObservation {
    pub(crate) fn commit(mut self) {
        if let Some(summary) = self.summary.as_mut() {
            summary.committed = summary.staged;
            if let Some(current) = summary.committed {
                self.state.publish(current);
                increment(&self.sender.totals.commits);
            }
        }
        self.finish(None);
    }
    fn finish(mut self, error: Option<&ApplicationError>) {
        let summary = self.summary.take().expect("single-use observation");
        let outcome = if error.is_some() {
            Outcome::Rejected
        } else if summary.committed.is_some() {
            Outcome::Accepted
        } else {
            Outcome::ReadOnly
        };
        self.emit(summary, outcome, error.map(reason));
    }
    fn emit(&self, summary: Accumulator, outcome: Outcome, reason: Option<Reason>) {
        for (dest, value) in self.state.peaks.iter().zip(summary.peak.0) {
            dest.fetch_max(value, Ordering::Relaxed);
        }
        increment(&self.sender.totals.attempts[self.operation.index() * 4 + outcome as usize]);
        if let Some(reason) = reason {
            increment(&self.sender.totals.reasons[reason.0]);
        }
        let record = OperationSummary {
            session: self.state.id,
            sequence: self.sequence,
            operation: self.operation,
            outcome,
            reason,
            wall_seconds: self.wall_seconds,
            work: summary.work,
            clone_slots: summary.clones,
            admitted_bytes: summary.admitted,
            retained: summary.retained,
            max_battle_owner_extent_bytes: summary.max_battle_extent,
            max_pending_owner_extent_bytes: summary.max_pending_extent,
            max_relic_owner_extent_bytes: summary.max_relic_extent,
            started_identity: summary.started_identity,
            prepared: summary.staged,
            peak: summary.peak,
            committed: summary.committed,
        };
        if self.sender.sender.try_send(record).is_err() {
            increment(&self.sender.totals.dropped);
        }
    }
}
impl Drop for PendingObservation {
    fn drop(&mut self) {
        if let Some(summary) = self.summary.take() {
            self.emit(summary, Outcome::Discarded, None);
        }
    }
}

pub(crate) fn budget(
    work: u64,
    clones: usize,
    admitted: Option<usize>,
    retained: RetainedAccounting,
) {
    ACTIVE.with(|slot| {
        if let Some(s) = slot.borrow_mut().as_mut() {
            s.work = s.work.saturating_add(work);
            s.clones = s.clones.saturating_add(clones as u64);
            s.admitted = s.admitted.saturating_add(admitted.unwrap_or(0) as u64);
            s.retained
                .get_or_insert_with(RetainedAccounting::default)
                .include(retained);
        }
    });
}
pub(crate) fn peak(field: usize, value: u64) {
    ACTIVE.with(|slot| {
        if let Some(s) = slot.borrow_mut().as_mut() {
            s.peak.0[field] = s.peak.0[field].max(value);
        }
    });
}
pub(crate) fn extent(value: usize) {
    ACTIVE.with(|slot| {
        if let Some(s) = slot.borrow_mut().as_mut() {
            s.max_battle_extent = Some(s.max_battle_extent.unwrap_or(0).max(value as u64));
            s.peak.0[EXTENT] = s.peak.0[EXTENT].max(value as u64);
        }
    });
}
pub(crate) fn pending_extent(value: usize) {
    ACTIVE.with(|slot| {
        if let Some(s) = slot.borrow_mut().as_mut() {
            s.max_pending_extent = Some(s.max_pending_extent.unwrap_or(0).max(value as u64));
        }
    });
}
pub(crate) fn relic_extent(value: usize) {
    ACTIVE.with(|slot| {
        if let Some(s) = slot.borrow_mut().as_mut() {
            s.max_relic_extent = Some(s.max_relic_extent.unwrap_or(0).max(value as u64));
        }
    });
}
pub(crate) fn sample(usage: Usage) {
    ACTIVE.with(|slot| {
        if let Some(s) = slot.borrow_mut().as_mut() {
            s.peak.include(usage);
        }
    });
}
pub(crate) fn staged(current: CommittedSnapshot) {
    ACTIVE.with(|slot| {
        if let Some(s) = slot.borrow_mut().as_mut() {
            s.peak.include(current.usage);
            s.staged = Some(current);
        }
    });
}
pub(crate) fn committed(root_address: usize) {
    ACTIVE.with(|slot| {
        if let Some(s) = slot.borrow_mut().as_mut() {
            if s.root_address != root_address {
                return;
            }
            s.committed = s.staged;
            if let (Some(current), Some((state, totals))) = (s.committed, s.publisher.as_ref()) {
                state.publish(current);
                increment(&totals.commits);
            }
        }
    });
}
