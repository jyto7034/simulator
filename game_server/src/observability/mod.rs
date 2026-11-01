//! Optional host collector. Gameplay never waits for this worker, HTTP or scrape.
use game_core::{
    application::CoreApplication,
    runtime_observation::{
        self as core, ObservationReceiver, ObservationSender, SessionReader, SessionSnapshot,
        Usage, USAGE_FIELDS, WORKER_BATCH,
    },
};
use metrics::{Counter, Gauge, Histogram, Key, Label, Level, Metadata, Recorder};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::Notify;
use uuid::Uuid;

pub const RING_CAPACITY: usize = 300;
pub const MAX_OBSERVED_SESSIONS: usize = 256;
pub const ERROR_BYTE_CAP: usize = 256;
const METADATA: Metadata<'static> = Metadata::new("core_runtime", Level::INFO, None);
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct ObservationConfig {
    pub enabled: bool,
    pub metrics_listen: SocketAddr,
    pub detail_listen: SocketAddr,
    pub snapshot_ms: u64,
}
impl Default for ObservationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            metrics_listen: "127.0.0.1:9109".parse().unwrap(),
            detail_listen: "127.0.0.1:9110".parse().unwrap(),
            snapshot_ms: 200,
        }
    }
}
impl ObservationConfig {
    pub fn from_path(path: &Path) -> std::io::Result<Self> {
        use std::io::Read;
        let file = std::fs::File::open(path)?;
        if !file.metadata()?.is_file() {
            return Err(std::io::Error::other(
                "observability path must be a regular file",
            ));
        }
        let mut bytes = Vec::new();
        file.take(16 * 1024 + 1).read_to_end(&mut bytes)?;
        if bytes.len() > 16 * 1024 {
            return Err(std::io::Error::other("observability file exceeds 16KiB"));
        }
        let config: Self = ron::de::from_bytes(&bytes).map_err(std::io::Error::other)?;
        config.validate()?;
        Ok(config)
    }
    fn validate(&self) -> std::io::Result<()> {
        if !self.metrics_listen.ip().is_loopback() || !self.detail_listen.ip().is_loopback() {
            return Err(std::io::Error::other("observation HTTP must bind loopback"));
        }
        if self.snapshot_ms == 0 || self.snapshot_ms > 60_000 {
            return Err(std::io::Error::other("snapshot_ms must be in 1..=60000"));
        }
        if self.enabled && self.metrics_listen == self.detail_listen {
            return Err(std::io::Error::other(
                "metrics and detail listeners must differ",
            ));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct RingRow {
    pub wall_unix_ms: u64,
    pub generation: u64,
    pub session: Option<u64>,
    pub battle: Option<Uuid>,
    pub boundary: Option<&'static str>,
    pub usage: Usage,
    pub interval_peak: Usage,
}
#[derive(Default, Serialize)]
pub struct Detail {
    pub status: &'static str,
    pub error: Option<String>,
    pub sessions: Vec<SessionSnapshot>,
    pub selected_session: Option<u64>,
    pub selected_battle: Option<Uuid>,
    pub selected_at_unix_ms: Option<u64>,
    pub generation: u64,
    pub ring: std::collections::VecDeque<RingRow>,
    pub raw_operations: std::collections::VecDeque<core::OperationSummary>,
    pub raw_operations_received: u64,
    pub raw_operations_overwritten: u64,
    pub registration_drops: u64,
    pub queue_drops: u64,
    pub snapshot_sessions_truncated: u64,
    #[cfg(feature = "runtime-measurement")]
    pub process_heap: Option<crate::runtime_measurement::HeapSnapshot>,
    #[cfg(feature = "runtime-measurement")]
    pub process_window: Option<crate::runtime_measurement::ActiveHeapWindow>,
    #[cfg(feature = "runtime-measurement")]
    pub process_rss: Option<crate::runtime_measurement::RssSample>,
}
impl Detail {
    fn operation(&mut self, summary: core::OperationSummary) {
        self.raw_operations_received = self.raw_operations_received.saturating_add(1);
        if self.raw_operations.len() == RING_CAPACITY {
            self.raw_operations.pop_front();
            self.raw_operations_overwritten = self.raw_operations_overwritten.saturating_add(1);
        }
        self.raw_operations.push_back(summary);
    }
    fn push(&mut self, row: RingRow) {
        if self.ring.len() == RING_CAPACITY {
            self.ring.pop_front();
        }
        self.ring.push_back(row);
    }
    fn select(&mut self, session: Option<u64>) -> Result<(), &'static str> {
        let battle = match session {
            Some(id) => Some(
                self.sessions
                    .iter()
                    .find(|s| s.session == id)
                    .and_then(|s| s.current.battle)
                    .ok_or("session has no active battle")?,
            ),
            None => None,
        };
        self.generation = self.generation.saturating_add(1);
        self.selected_session = session;
        self.selected_battle = battle;
        let now = now_ms();
        self.selected_at_unix_ms = session.map(|_| now);
        self.push(RingRow {
            wall_unix_ms: now,
            generation: self.generation,
            session,
            battle,
            boundary: Some("selection"),
            usage: Usage::default(),
            interval_peak: Usage::default(),
        });
        Ok(())
    }
}
fn bounded_error(error: impl std::fmt::Display) -> String {
    struct Writer(String);
    impl std::fmt::Write for Writer {
        fn write_str(&mut self, value: &str) -> std::fmt::Result {
            let remaining = ERROR_BYTE_CAP.saturating_sub(self.0.len());
            let mut end = remaining.min(value.len());
            while !value.is_char_boundary(end) {
                end -= 1;
            }
            self.0.push_str(&value[..end]);
            Ok(())
        }
    }
    let mut output = Writer(String::new());
    let _ = std::fmt::write(&mut output, format_args!("{error}"));
    output.0
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}
struct Control {
    sender: ObservationSender,
    sessions: Mutex<Vec<SessionReader>>,
    next_session: AtomicU64,
    registration_drops: AtomicU64,
    detail: Mutex<Detail>,
    stop: AtomicBool,
    stopped: Notify,
    #[cfg(test)]
    stalled: AtomicBool,
}
/// Shared attachment/diagnostic handle. Drop the separate guard to stop workers.
#[derive(Clone)]
pub struct Observability {
    control: Arc<Control>,
}
pub struct ObservationGuard {
    control: Arc<Control>,
}
impl Drop for ObservationGuard {
    fn drop(&mut self) {
        self.control.stop.store(true, Ordering::Release);
        self.control.stopped.notify_one();
    }
}
impl Observability {
    pub fn start(config: ObservationConfig) -> std::io::Result<Option<(Self, ObservationGuard)>> {
        config.validate()?;
        if !config.enabled {
            return Ok(None);
        }
        let (sender, receiver) = core::channel();
        let control = Arc::new(Control {
            sender,
            sessions: Mutex::new(Vec::new()),
            next_session: AtomicU64::new(1),
            registration_drops: AtomicU64::new(0),
            detail: Mutex::new(Detail {
                status: "starting",
                ..Default::default()
            }),
            stop: AtomicBool::new(false),
            stopped: Notify::new(),
            #[cfg(test)]
            stalled: AtomicBool::new(false),
        });
        let worker = control.clone();
        std::thread::Builder::new()
            .name("runtime-observation".into())
            .spawn(move || {
                let result = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .max_blocking_threads(1)
                    .build()
                    .and_then(|runtime| runtime.block_on(run(worker.clone(), receiver, &config)));
                let mut detail = worker.detail.lock().unwrap_or_else(|e| e.into_inner());
                match result {
                    Ok(()) => detail.status = "stopped",
                    Err(error) => {
                        tracing::error!(%error,"Runtime observation collector stopped");
                        detail.status = "failed";
                        detail.error = Some(bounded_error(&error));
                    }
                }
            })?;
        Ok(Some((
            Self {
                control: control.clone(),
            },
            ObservationGuard { control },
        )))
    }
    /// Registration happens once per application. A contended observer registry
    /// skips attachment; it never waits for collection or a scrape.
    pub fn attach(&self, application: &mut CoreApplication) -> bool {
        let Ok(mut sessions) = self.control.sessions.try_lock() else {
            self.control
                .registration_drops
                .fetch_add(1, Ordering::Relaxed);
            return false;
        };
        sessions.retain(SessionReader::is_alive);
        if sessions.len() >= MAX_OBSERVED_SESSIONS {
            self.control
                .registration_drops
                .fetch_add(1, Ordering::Relaxed);
            return false;
        }
        if sessions.capacity() > sessions.len().saturating_mul(4).saturating_add(64) {
            sessions.shrink_to_fit();
        }
        let Ok(id) =
            self.control
                .next_session
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        else {
            return false;
        };
        let (observer, reader) = self.control.sender.session(id);
        application.set_runtime_observer(Some(observer));
        sessions.push(reader);
        true
    }
    pub fn detail_json(&self) -> serde_json::Value {
        serde_json::to_value(
            &*self
                .control
                .detail
                .lock()
                .unwrap_or_else(|e| e.into_inner()),
        )
        .expect("finite diagnostic values")
    }
    pub fn select(&self, session: Option<u64>) -> Result<(), &'static str> {
        self.control
            .detail
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .select(session)
    }
}
/// Optional path is separate from content/safety settings and their identities.
/// Invalid observation configuration disables collection, never Core startup.
pub fn from_environment() -> Option<(Observability, ObservationGuard)> {
    let path = std::env::var_os("GAME_OBSERVABILITY_PATH")?;
    match ObservationConfig::from_path(Path::new(&path)).and_then(Observability::start) {
        Ok(value) => value,
        Err(error) => {
            tracing::error!(%error,"Runtime observation configuration disabled");
            None
        }
    }
}

struct Handles {
    operations: Vec<Counter>,
    rejections: Vec<Counter>,
    commits: Counter,
    drops: Counter,
    registration_drops: Counter,
    expired: Counter,
    last_snapshot: Gauge,
    active: Gauge,
    up: Gauge,
    selected_active: Gauge,
    selected_generation: Gauge,
    aggregate: Vec<[Gauge; 4]>,
    selected: Vec<[Gauge; 2]>,
    histograms: Vec<[Histogram; 4]>,
}
impl Handles {
    fn new(recorder: &impl Recorder) -> Self {
        let counter = |name: &'static str, labels: Vec<Label>| {
            recorder.register_counter(&Key::from_parts(name, labels), &METADATA)
        };
        let gauge = |name: &'static str, labels: Vec<Label>| {
            recorder.register_gauge(&Key::from_parts(name, labels), &METADATA)
        };
        let labels = core::Operation::labels();
        let operations = labels
            .iter()
            .flat_map(|(_, op)| {
                core::Outcome::ALL.map(|outcome| {
                    counter(
                        "core_runtime_operations_total",
                        vec![
                            Label::new("operation", op.clone()),
                            Label::new("outcome", outcome.label()),
                        ],
                    )
                })
            })
            .collect();
        let rejections = core::reason_labels()
            .into_iter()
            .map(|reason| {
                counter(
                    "core_runtime_rejections_total",
                    vec![Label::new("reason", reason)],
                )
            })
            .collect();
        let histograms = labels
            .iter()
            .map(|(_, op)| {
                [
                    "core_runtime_operation_seconds",
                    "core_runtime_work",
                    "core_runtime_clone_slots",
                    "core_runtime_admitted_bytes",
                ]
                .map(|name| {
                    recorder.register_histogram(
                        &Key::from_parts(name, vec![Label::new("operation", op.clone())]),
                        &METADATA,
                    )
                })
            })
            .collect();
        let aggregate = core::USAGE_NAMES
            .iter()
            .map(|name| {
                ["sum", "max", "interval_peak_sum", "interval_peak_max"].map(|stat| {
                    gauge(
                        "core_runtime_usage",
                        vec![Label::new("quantity", *name), Label::new("statistic", stat)],
                    )
                })
            })
            .collect();
        let selected = core::USAGE_NAMES
            .iter()
            .map(|name| {
                ["current", "interval_peak"].map(|stat| {
                    gauge(
                        "core_runtime_selected_usage",
                        vec![
                            Label::new("slot", "0"),
                            Label::new("quantity", *name),
                            Label::new("statistic", stat),
                        ],
                    )
                })
            })
            .collect();
        Self {
            operations,
            rejections,
            histograms,
            aggregate,
            selected,
            commits: counter("core_runtime_commits_total", vec![]),
            drops: counter("core_runtime_observation_dropped_total", vec![]),
            registration_drops: counter("core_runtime_registration_dropped_total", vec![]),
            expired: counter("core_runtime_observation_expired_total", vec![]),
            last_snapshot: gauge("core_runtime_last_snapshot_unix_seconds", vec![]),
            active: gauge("core_runtime_active_battles", vec![]),
            up: gauge("core_runtime_collector_up", vec![]),
            selected_active: gauge(
                "core_runtime_selected_active",
                vec![Label::new("slot", "0")],
            ),
            selected_generation: gauge(
                "core_runtime_selection_generation",
                vec![Label::new("slot", "0")],
            ),
        }
    }
    fn counters(&self, receiver: &ObservationReceiver, registration_drops: u64) {
        let counters = receiver.counters();
        for (handle, value) in self.operations.iter().zip(counters.attempts) {
            handle.absolute(value);
        }
        for (handle, value) in self.rejections.iter().zip(counters.reasons) {
            handle.absolute(value);
        }
        self.commits.absolute(counters.commits);
        self.drops.absolute(counters.dropped);
        self.registration_drops.absolute(registration_drops);
    }
}
#[derive(Default)]
struct RetirementGate {
    ready: Option<Arc<AtomicBool>>,
}
impl RetirementGate {
    fn poll(&mut self) -> bool {
        {
            let guard = crossbeam_epoch::pin();
            guard.flush();
        }
        if self
            .ready
            .as_ref()
            .is_some_and(|ready| !ready.load(Ordering::Acquire))
        {
            return false;
        }
        self.ready = None;
        true
    }
    fn after_upkeep(&mut self) {
        let ready = Arc::new(AtomicBool::new(false));
        let completion = ready.clone();
        let guard = crossbeam_epoch::pin();
        guard.defer(move || completion.store(true, Ordering::Release));
        guard.flush();
        self.ready = Some(ready);
    }
}
#[derive(Default)]
struct SessionOwner {
    sequence: u64,
    latest: Option<SessionSnapshot>,
}
impl SessionOwner {
    fn sample(&mut self, current: Option<SessionSnapshot>) -> Option<SessionSnapshot> {
        if let Some(current) = current {
            self.latest = Some(current);
            return Some(current);
        }
        // A bounded atomic-read retry can miss a concurrent writer. Keep the
        // last committed gauge and selection; only dead weak leases retire it.
        self.latest.map(|mut previous| {
            previous.interval_peak = previous.current.usage;
            previous
        })
    }
}
struct Worker {
    handles: Handles,
    owners: HashMap<u64, SessionOwner>,
    gate: RetirementGate,
}
impl Worker {
    fn consume(&mut self, summary: core::OperationSummary) -> bool {
        let Some(owner) = self.owners.get_mut(&summary.session) else {
            self.handles.expired.increment(1);
            return false;
        };
        if summary.sequence <= owner.sequence {
            return false;
        }
        owner.sequence = summary.sequence;
        let histograms = &self.handles.histograms[summary.operation.index()];
        for (metric, value) in histograms.iter().zip([
            summary.wall_seconds,
            summary.work as f64,
            summary.clone_slots as f64,
            summary.admitted_bytes as f64,
        ]) {
            metric.record(value);
        }
        true
    }
    fn batch(
        &mut self,
        receiver: &ObservationReceiver,
        handle: &PrometheusHandle,
        control: &Control,
    ) -> usize {
        if !self.gate.poll() {
            return 0;
        }
        let mut recorded = 0;
        for _ in 0..WORKER_BATCH {
            let Ok(summary) = receiver.try_recv() else {
                break;
            };
            // Raw queue history includes retired leases and their terminal
            // refusal. It holds only Copy metadata, never application owners.
            control
                .detail
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .operation(summary);
            recorded += usize::from(self.consume(summary));
        }
        let mut detail = control.detail.lock().unwrap_or_else(|e| e.into_inner());
        detail.queue_drops = receiver.counters().dropped;
        detail.registration_drops = control.registration_drops.load(Ordering::Relaxed);
        drop(detail);
        handle.run_upkeep();
        if recorded != 0 {
            self.gate.after_upkeep();
        }
        recorded
    }
    fn sessions(&mut self, control: &Control, take_peaks: bool) -> Vec<SessionSnapshot> {
        let mut readers = control.sessions.lock().unwrap_or_else(|e| e.into_inner());
        readers.retain(SessionReader::is_alive);
        if readers.capacity() > readers.len().saturating_mul(4).saturating_add(64) {
            readers.shrink_to_fit();
        }
        let live: std::collections::HashSet<_> = readers.iter().map(SessionReader::id).collect();
        self.owners.retain(|id, _| live.contains(id));
        let snapshots = readers
            .iter()
            .filter_map(|reader| {
                self.owners
                    .entry(reader.id())
                    .or_default()
                    .sample(reader.snapshot(take_peaks))
            })
            .collect();
        if self.owners.capacity() > self.owners.len().saturating_mul(4).saturating_add(64) {
            self.owners.shrink_to_fit();
        }
        snapshots
    }

    fn snapshot(&self, control: &Control, mut sessions: Vec<SessionSnapshot>) {
        let truncated = sessions.len().saturating_sub(MAX_OBSERVED_SESSIONS);
        sessions.truncate(MAX_OBSERVED_SESSIONS);
        self.handles.last_snapshot.set(now_ms() as f64 / 1000.0);
        let mut aggregates = [[0_u64; 4]; USAGE_FIELDS];
        let mut active = 0;
        for session in &sessions {
            if session.current.battle.is_some() {
                active += 1;
                for (i, row) in aggregates.iter_mut().enumerate() {
                    row[0] = row[0].saturating_add(session.current.usage.0[i]);
                    row[1] = row[1].max(session.current.usage.0[i]);
                    row[2] = row[2].saturating_add(session.interval_peak.0[i]);
                    row[3] = row[3].max(session.interval_peak.0[i]);
                }
            }
        }
        self.handles.active.set(active as f64);
        for (handles, values) in self.handles.aggregate.iter().zip(aggregates) {
            for (handle, value) in handles.iter().zip(values) {
                handle.set(value as f64);
            }
        }
        let mut detail = control.detail.lock().unwrap_or_else(|e| e.into_inner());
        detail.snapshot_sessions_truncated = detail
            .snapshot_sessions_truncated
            .saturating_add(truncated as u64);
        #[cfg(feature = "runtime-measurement")]
        {
            detail.process_heap = Some(crate::runtime_measurement::process_heap());
            detail.process_window = crate::runtime_measurement::process_active_window();
            detail.process_rss = Some(crate::runtime_measurement::process_rss());
        }
        detail.sessions = sessions;
        let selected = detail
            .selected_session
            .and_then(|id| detail.sessions.iter().find(|s| s.session == id).copied());
        if detail.selected_session.is_some() {
            let same = selected.is_some_and(|s| {
                s.current.battle.is_some() && s.current.battle == detail.selected_battle
            });
            let row = RingRow {
                wall_unix_ms: now_ms(),
                generation: detail.generation,
                session: detail.selected_session,
                battle: detail.selected_battle,
                boundary: (!same).then_some("ended"),
                usage: selected.map_or(Usage::default(), |s| s.current.usage),
                interval_peak: selected.map_or(Usage::default(), |s| s.interval_peak),
            };
            detail.push(row);
            if !same {
                detail.selected_session = None;
                detail.selected_battle = None;
                detail.selected_at_unix_ms = None;
            }
        }
        let selected = selected.filter(|s| Some(s.session) == detail.selected_session);
        self.handles
            .selected_active
            .set(f64::from(selected.is_some()));
        self.handles
            .selected_generation
            .set(detail.generation as f64);
        for (i, handles) in self.handles.selected.iter().enumerate() {
            handles[0].set(selected.map_or(0, |s| s.current.usage.0[i]) as f64);
            handles[1].set(selected.map_or(0, |s| s.interval_peak.0[i]) as f64);
        }
    }
}
async fn run(
    control: Arc<Control>,
    receiver: ObservationReceiver,
    config: &ObservationConfig,
) -> std::io::Result<()> {
    let builder = PrometheusBuilder::new()
        .with_http_listener(config.metrics_listen)
        .set_buckets(&[
            0., 1., 8., 64., 512., 4096., 32768., 262144., 2097152., 16777216.,
        ])
        .map_err(std::io::Error::other)?
        .set_buckets_for_metric(
            Matcher::Full("core_runtime_operation_seconds".into()),
            &[
                0.00001, 0.00005, 0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1., 5., 30.,
            ],
        )
        .map_err(std::io::Error::other)?;
    let (recorder, exporter) = builder.build().map_err(std::io::Error::other)?;
    let handle = recorder.handle();
    let handles = Handles::new(&recorder);
    handles.up.set(1.);
    let mut worker = Worker {
        handles,
        owners: HashMap::new(),
        gate: RetirementGate::default(),
    };
    let mut exporter = tokio::spawn(exporter);
    // Actix owns diagnostic HTTP; the standard metrics exporter owns /metrics.
    let detail_control = control.clone();
    let detail_address = config.detail_listen;
    let (detail_stop_tx, detail_stop_rx) = tokio::sync::oneshot::channel::<()>();
    let detail_thread=std::thread::Builder::new().name("runtime-observation-detail".into()).spawn(move||{
        actix_web::rt::System::new().block_on(async move {
            let app=Observability {control:detail_control.clone()};
            let server=actix_web::HttpServer::new(move||actix_web::App::new().app_data(actix_web::web::Data::new(app.clone())).route("/state",actix_web::web::get().to(detail_state)).route("/selection",actix_web::web::post().to(detail_select))).workers(1).bind(detail_address);
            match server {Ok(server)=>{let server=server.run();let handle=server.handle();tokio::select!{_ = detail_stop_rx=>{handle.stop(false).await;},result=server=>{if let Err(error)=result{tracing::error!(%error,"Observation detail server failed");}}}},Err(error)=>{tracing::error!(%error,"Observation detail bind failed");detail_control.detail.lock().unwrap_or_else(|e|e.into_inner()).error=Some(bounded_error(format_args!("detail: {error}")));}}
        });
    })?;
    control
        .detail
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .status = "running";
    let mut interval = tokio::time::interval(Duration::from_millis(10));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut next_snapshot = std::time::Instant::now();
    let result = loop {
        if control.stop.load(Ordering::Acquire) {
            break Ok(());
        }
        tokio::select! {
            _=control.stopped.notified()=>{if control.stop.load(Ordering::Acquire){break Ok(());}},
            result=&mut exporter=>{break Err(std::io::Error::other(format!("metrics exporter stopped: {result:?}")));},
            _=interval.tick()=>{
                #[cfg(test)] if control.stalled.load(Ordering::Acquire){continue;}
                let snapshot=std::time::Instant::now()>=next_snapshot;
                let sessions=worker.sessions(&control,snapshot);
                worker.batch(&receiver,&handle,&control);
                worker.handles.counters(&receiver,control.registration_drops.load(Ordering::Relaxed));
                if snapshot{worker.snapshot(&control,sessions);next_snapshot=std::time::Instant::now()+Duration::from_millis(config.snapshot_ms);}
            }
        }
    };
    worker.handles.up.set(0.);
    let _ = detail_stop_tx.send(());
    exporter.abort();
    let _ = detail_thread.join();
    result
}
async fn detail_state(app: actix_web::web::Data<Observability>) -> actix_web::HttpResponse {
    actix_web::HttpResponse::Ok().json(app.detail_json())
}
#[derive(Deserialize)]
struct Selection {
    session: Option<u64>,
}
async fn detail_select(
    app: actix_web::web::Data<Observability>,
    body: actix_web::web::Json<Selection>,
) -> actix_web::HttpResponse {
    match app.select(body.session) {
        Ok(()) => actix_web::HttpResponse::Ok().json(app.detail_json()),
        Err(error) => {
            actix_web::HttpResponse::BadRequest().json(serde_json::json!({"error":error}))
        }
    }
}

// Legacy raw-data fixtures require Core fixture APIs in release tests.
#[cfg(all(test, any(debug_assertions, feature = "test-fixtures")))]
mod tests;
