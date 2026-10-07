//! Finite, trusted local playback through the public Core application boundary.
//! Measurement/export failure never converts a completed operation to a fault.
use super::{begin_process_window, process_active_window, process_heap, process_rss};
use game_core::{
    application::*,
    game::{
        behavior::{GameStateContextDto, RunSnapshotDto},
        data::{
            content_pack::{self, ContentSelection, InputLimits},
            ValidatedGameData,
        },
        map::{MapNodeCategory, MapNodeState},
    },
    runtime_observation::{self as observation, SessionReader},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    time::Instant,
};
use uuid::Uuid;
mod script;

pub const MANIFEST_BYTE_CAP: usize = 1024 * 1024;
pub const STEP_CAP: usize = 10000;
pub const REPEAT_CAP: usize = 64;
pub const SESSION_CAP: usize = 16;
pub const SAMPLE_CAP: usize = 4096;
pub const EXPORT_BYTE_CAP: usize = 16 * 1024 * 1024;
const RECORD_BYTE_CAP: usize = 64 * 1024;
const FOOTER_RESERVE: usize = RECORD_BYTE_CAP;
type Error = Box<dyn std::error::Error>;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObservationMode {
    None,
    Normal,
    Detached,
    Full,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionSpec {
    pub player: Uuid,
    pub seed: u64,
    pub runtime_safety_path: Option<PathBuf>,
    pub expected_effective_profile_sha256: String,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    pub case_id: String,
    pub pack_root: PathBuf,
    pub selection_path: PathBuf,
    pub expected_content_sha256: Option<String>,
    /// Provenance supplied by the invoker, distinct from compile-time identity.
    pub source_sha256: String,
    pub observation: ObservationMode,
    pub repeats: usize,
    pub sessions: Vec<SessionSpec>,
    pub steps: Vec<Step>,
    pub max_samples: usize,
    pub max_export_bytes: usize,
    #[serde(default)]
    pub stop_on_rejection: bool,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Step {
    Script {
        step: game_core::authoring::TrialStep,
    },
    Command {
        document: Value,
        #[serde(default)]
        current_battle_authority: bool,
    },
    Admin {
        document: Value,
    },
    PrepareDiscard {
        document: Value,
    },
    Elapsed {
        delta_ms: u64,
        #[serde(default)]
        after_seq: u64,
    },
    NextEvent,
    HostTick {
        delta_ms: u64,
        #[serde(default)]
        after_seq: u64,
    },
    DisconnectPause,
    Reconnect {
        #[serde(default)]
        after_seq: u64,
    },
    Snapshot {
        snapshot: ApplicationSnapshotKind,
    },
    Checkpoint,
    RestoreCheckpoint,
    CaptureRollback,
    RestoreRollback,
    /// Bounded navigation of the existing public run to one ordinary combat.
    EnterOrdinaryCombat,
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}
fn digest_text(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub fn read_bounded(path: &Path, cap: usize) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > cap as u64 {
        return Err(invalid("input must be a bounded regular file"));
    }
    let mut bytes = Vec::new();
    file.take(cap as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > cap {
        return Err(invalid("input grew beyond its byte cap"));
    }
    Ok(bytes)
}
impl Manifest {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MANIFEST_BYTE_CAP {
            return Err(invalid("manifest byte cap exceeded").into());
        }
        let value: Self = serde_json::from_slice(bytes)?;
        value.validate()?;
        Ok(value)
    }
    fn validate(&self) -> Result<(), Error> {
        if self.schema_version != 1
            || self.case_id.is_empty()
            || self.case_id.len() > 64
            || !self
                .case_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
            || !digest_text(&self.source_sha256)
            || self
                .expected_content_sha256
                .as_deref()
                .is_some_and(|v| !digest_text(v))
            || !(1..=REPEAT_CAP).contains(&self.repeats)
            || !(1..=SESSION_CAP).contains(&self.sessions.len())
            || self.steps.is_empty()
            || self.steps.len() > STEP_CAP
            || !(1..=SAMPLE_CAP).contains(&self.max_samples)
            || !(2 * RECORD_BYTE_CAP..=EXPORT_BYTE_CAP).contains(&self.max_export_bytes)
        {
            return Err(invalid("invalid playback identity or finite bound").into());
        }
        let mut weighted_steps = ApplicationSnapshotKind::ALL.len();
        for (index, step) in self.steps.iter().enumerate() {
            let weight = match step {
                Step::EnterOrdinaryCombat => 514,
                Step::Command {
                    current_battle_authority: true,
                    ..
                } => 2,
                Step::Script { step } => script::validate(step, index)?,
                _ => 1,
            };
            weighted_steps = weighted_steps
                .checked_add(weight)
                .ok_or_else(|| invalid("step bound overflow"))?;
            match step {
                Step::Command { document, .. } | Step::PrepareDiscard { document } => {
                    ApplicationCommand::from_document(document.clone().into())?;
                }
                Step::Admin { document } => {
                    ApplicationAdminCommand::from_document(document.clone().into())?;
                }
                _ => {}
            }
        }
        if weighted_steps
            .checked_mul(self.repeats)
            .and_then(|n| n.checked_mul(self.sessions.len()))
            .is_none_or(|n| n > STEP_CAP)
        {
            return Err(invalid("total public step cap exceeded").into());
        }
        for (i, session) in self.sessions.iter().enumerate() {
            if !digest_text(&session.expected_effective_profile_sha256)
                || self.sessions[..i]
                    .iter()
                    .any(|s| s.player == session.player)
            {
                return Err(invalid("invalid/duplicate session identity").into());
            }
        }
        for path in std::iter::once(&self.pack_root)
            .chain(std::iter::once(&self.selection_path))
            .chain(
                self.sessions
                    .iter()
                    .filter_map(|s| s.runtime_safety_path.as_ref()),
            )
        {
            if path.as_os_str().len() > 4096 {
                return Err(invalid("path metadata cap exceeded").into());
            }
        }
        Ok(())
    }
}

/// All profile gates run before constructing any CoreApplication or output.
pub fn load_inputs(manifest: &Manifest) -> Result<(Vec<ValidatedGameData>, String), Error> {
    manifest.validate()?;
    let mut profiles = Vec::with_capacity(manifest.sessions.len());
    for session in &manifest.sessions {
        let profile =
            crate::bootstrap::load_runtime_safety(session.runtime_safety_path.as_deref())?;
        profile.verify_effective_sha256(&session.expected_effective_profile_sha256)?;
        profiles.push(profile);
    }
    let selection_bytes = read_bounded(&manifest.selection_path, 64 * 1024)?;
    let selection_sha256 = format!("{:x}", Sha256::digest(&selection_bytes));
    let selection: ContentSelection = serde_json::from_slice(&selection_bytes)?;
    let mut inputs = Vec::with_capacity(profiles.len());
    for profile in profiles {
        let data = content_pack::load_with_runtime_safety(
            &manifest.pack_root,
            &selection,
            &InputLimits::default(),
            profile,
        )?;
        if manifest
            .expected_content_sha256
            .as_deref()
            .is_some_and(|hash| hash != data.identity().semantic_sha256)
        {
            return Err(invalid("content identity mismatch before execution").into());
        }
        inputs.push(data);
    }
    Ok((inputs, selection_sha256))
}

struct DigestWriter {
    digest: Sha256,
    written: usize,
    cap: usize,
}
impl Write for DigestWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let next = self
            .written
            .checked_add(bytes.len())
            .ok_or_else(|| invalid("digest overflow"))?;
        if next > self.cap {
            return Err(invalid("semantic digest scope exceeds finite cap"));
        }
        self.digest.update(bytes);
        self.written = next;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn semantic_digest<T: Serialize>(value: &T) -> Option<String> {
    let mut stream = DigestWriter {
        digest: Sha256::new(),
        written: 0,
        cap: EXPORT_BYTE_CAP,
    };
    serde_json::to_writer(&mut stream, value).ok()?;
    Some(format!("{:x}", stream.digest.finalize()))
}
fn executable_sha256() -> Option<&'static str> {
    static DIGEST: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    DIGEST
        .get_or_init(|| {
            let path = std::env::current_exe().ok()?;
            let mut file = File::open(path).ok()?;
            let metadata = file.metadata().ok()?;
            if !metadata.is_file() || metadata.len() > 1024 * 1024 * 1024 {
                return None;
            }
            let mut digest = Sha256::new();
            let mut bytes = [0; 16 * 1024];
            let mut total = 0u64;
            loop {
                let count = file.read(&mut bytes).ok()?;
                if count == 0 {
                    break;
                }
                total = total.checked_add(count as u64)?;
                if total > 1024 * 1024 * 1024 {
                    return None;
                }
                digest.update(&bytes[..count]);
            }
            Some(format!("{:x}", digest.finalize()))
        })
        .as_deref()
}
struct ByteBuffer {
    bytes: Vec<u8>,
    cap: usize,
}
impl Write for ByteBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self
            .bytes
            .len()
            .checked_add(bytes.len())
            .is_none_or(|n| n > self.cap)
        {
            return Err(invalid("record byte cap exceeded"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub struct Export {
    file: File,
    bytes: usize,
    cap: usize,
    sample_cap: usize,
    pub retained: usize,
    pub dropped: usize,
    pub records_seen: usize,
    pub io_failed: bool,
}
impl Export {
    pub fn create(path: &Path, cap: usize, sample_cap: usize) -> io::Result<Self> {
        if !(2 * RECORD_BYTE_CAP..=EXPORT_BYTE_CAP).contains(&cap)
            || !(1..=SAMPLE_CAP).contains(&sample_cap)
        {
            return Err(invalid("invalid export bounds"));
        }
        Ok(Self {
            file: OpenOptions::new().write(true).create_new(true).open(path)?,
            bytes: 0,
            cap,
            sample_cap,
            retained: 0,
            dropped: 0,
            records_seen: 0,
            io_failed: false,
        })
    }
    fn line<T: Serialize>(&mut self, value: &T, reserve: usize) -> bool {
        if self.io_failed {
            return false;
        }
        let mut buffer = ByteBuffer {
            bytes: Vec::new(),
            cap: RECORD_BYTE_CAP - 1,
        };
        if serde_json::to_writer(&mut buffer, value).is_err() {
            return false;
        }
        buffer.bytes.push(b'\n');
        if self
            .bytes
            .checked_add(buffer.bytes.len())
            .and_then(|n| n.checked_add(reserve))
            .is_none_or(|n| n > self.cap)
        {
            return false;
        }
        if self.file.write_all(&buffer.bytes).is_err() {
            self.io_failed = true;
            return false;
        }
        self.bytes += buffer.bytes.len();
        true
    }
    fn sample<T: Serialize>(&mut self, value: &T) -> bool {
        self.records_seen += 1;
        if self.retained == self.sample_cap || !self.line(value, FOOTER_RESERVE) {
            self.dropped += 1;
            return false;
        }
        self.retained += 1;
        true
    }
}

#[derive(Debug, Serialize)]
pub struct Distribution {
    pub retained_count: usize,
    pub mean: Option<f64>,
    pub p95: Option<u64>,
    pub max: Option<u64>,
}
fn distribution(values: &mut [u64]) -> Distribution {
    values.sort_unstable();
    let count = values.len();
    let sum: u128 = values.iter().map(|n| u128::from(*n)).sum();
    Distribution {
        retained_count: count,
        mean: (count != 0).then(|| sum as f64 / count as f64),
        p95: (count != 0).then(|| values[(count * 95).div_ceil(100) - 1]),
        max: values.last().copied(),
    }
}
struct Session {
    app: CoreApplication,
    player: Uuid,
    checkpoint: Option<ApplicationCheckpoint>,
    rollback: Option<ApplicationPersistenceRollback>,
    reader: Option<SessionReader>,
    history: script::History,
}
fn command(value: Value) -> Result<ApplicationCommand, ApplicationError> {
    ApplicationCommand::from_document(value.into())
}
fn context(projection: &ApplicationResult) -> Result<ApplicationExecutionContext, Error> {
    let update = &projection.payload().as_value()["battle_update"];
    Ok(ApplicationExecutionContext {
        expected_battle_authority: Some(ApplicationBattleAuthority {
            battle_uuid: serde_json::from_value(update["battle_uuid"].clone())?,
            battle_state_revision: update["battle_state_revision"]
                .as_u64()
                .ok_or_else(|| invalid("missing battle authority"))?,
        }),
        ..Default::default()
    })
}
fn enter_combat(session: &mut Session) -> StepOutcome {
    let mut operations = 0;
    macro_rules! call {
        ($operation:expr) => {{
            operations += 1;
            match $operation {
                Ok(value) => value,
                Err(error) => {
                    let mut outcome = result::<()>(Err(error));
                    outcome.public_operations = operations;
                    return outcome;
                }
            }
        }};
    }
    macro_rules! execute {
        ($value:expr) => {{
            let input = match command($value) {
                Ok(input) => input,
                Err(error) => {
                    let mut outcome = result::<()>(Err(error));
                    outcome.public_operations = operations;
                    return outcome;
                }
            };
            call!(session.app.execute(
                session.player,
                input,
                ApplicationExecutionContext::default()
            ))
        }};
    }
    execute!(json!({"type":"start_new_game","game_mode":"Standard","replace_active_run":false}));
    for _ in 0..128 {
        let snapshot = call!(session.app.snapshot(ApplicationSnapshotKind::Run));
        let run: RunSnapshotDto =
            match serde_json::from_value(snapshot.read_model().as_value().clone()) {
                Ok(run) => run,
                Err(_) => return driver_outcome("public run schema unavailable", operations),
            };
        let value = if let Some(vn) = run.visual_novel {
            if let Some(choice) = vn.choices.iter().find(|choice| choice.selectable) {
                json!({"type":"select_visual_novel_choice","session_id":vn.session_id,"current_scene_id":vn.current_scene_id,"choice_id":choice.choice_id})
            } else {
                json!({"type":"advance_visual_novel","session_id":vn.session_id,"current_scene_id":vn.current_scene_id})
            }
        } else if let GameStateContextDto::SelectingStarterEmployees {
            required_count,
            candidates,
        } = run.game_state_context
        {
            let ids: Vec<_> = candidates
                .into_iter()
                .take(required_count)
                .map(|c| c.employee_definition_id)
                .collect();
            json!({"type":"select_starter_employees","employee_definition_ids":ids})
        } else {
            let Some(map) = run.map else {
                return driver_outcome("ordinary combat requires a live map", operations);
            };
            let Some(node) = map
                .nodes
                .iter()
                .find(|n| {
                    n.state == MapNodeState::Available && n.category == MapNodeCategory::Combat
                })
                .or_else(|| {
                    map.nodes.iter().find(|n| {
                        n.state == MapNodeState::Available && n.category == MapNodeCategory::Start
                    })
                })
            else {
                return driver_outcome("seed has no adjacent ordinary combat", operations);
            };
            execute!(json!({"type":"select_map_node","node_id":node.id}));
            execute!(json!({"type":"confirm_enter_node"}));
            if node.category == MapNodeCategory::Combat {
                let snapshot = call!(session.app.snapshot(ApplicationSnapshotKind::Run));
                let mut outcome = result(Ok(snapshot));
                outcome.public_operations = operations;
                return outcome;
            }
            json!({"type":"complete_node"})
        };
        execute!(value);
    }
    driver_outcome("ordinary combat frontmatter bound exceeded", operations)
}

#[derive(Serialize, Default)]
struct StepOutcome {
    result_digest: Option<String>,
    error_kind: Option<String>,
    error_code: Option<String>,
    error_details_digest: Option<String>,
    public_operations: usize,
    resolved_command: Option<Value>,
    resolved_command_sha256: Option<String>,
    #[serde(skip)]
    public_result: Option<Value>,
    #[serde(skip)]
    public_result_bytes: usize,
}
fn driver_outcome(message: &str, operations: usize) -> StepOutcome {
    StepOutcome {
        result_digest: None,
        error_kind: Some("PlaybackInputOrDriver".into()),
        error_code: Some(message.chars().take(256).collect()),
        error_details_digest: None,
        public_operations: operations,
        ..Default::default()
    }
}
fn result<T: Serialize>(value: Result<T, ApplicationError>) -> StepOutcome {
    match value {
        Ok(value) => {
            let mut encoded = ByteBuffer {
                bytes: Vec::new(),
                cap: script::HISTORY_BYTE_CAP,
            };
            let public_result = if serde_json::to_writer(&mut encoded, &value).is_ok() {
                serde_json::from_slice(&encoded.bytes).ok()
            } else {
                None
            };
            StepOutcome {
                result_digest: semantic_digest(&value),
                error_kind: None,
                error_code: None,
                error_details_digest: None,
                public_operations: 1,
                public_result,
                public_result_bytes: encoded.bytes.len(),
                ..Default::default()
            }
        }
        Err(error) => StepOutcome {
            result_digest: None,
            error_kind: Some(format!("{:?}", error.kind())),
            error_code: Some(error.code().to_owned()),
            error_details_digest: semantic_digest(error.details()),
            public_operations: 1,
            ..Default::default()
        },
    }
}
fn apply(session: &mut Session, step: &Step) -> Result<StepOutcome, Error> {
    let outcome = match step {
        Step::Script { step } => {
            use game_core::authoring::{TrialClock, TrialStep};
            match step {
                TrialStep::Command { .. } => {
                    let (command, context, resolved) = script::resolve(step, &session.history)?;
                    let mut outcome = result(session.app.execute(session.player, command, context));
                    outcome.resolved_command_sha256 = semantic_digest(&resolved);
                    outcome.resolved_command = Some(resolved);
                    outcome
                }
                TrialStep::Snapshot { snapshot } => result(session.app.snapshot(*snapshot)),
                TrialStep::Clock { clock } => match clock {
                    TrialClock::Elapsed {
                        delta_ms,
                        after_seq,
                    } => result(session.app.advance_active_battle_by(*delta_ms, *after_seq)),
                    TrialClock::HostTick {
                        delta_ms,
                        after_seq,
                    } => result(
                        session
                            .app
                            .advance_active_battle_for_host_tick(*delta_ms, *after_seq),
                    ),
                    TrialClock::NextEventBucket => {
                        result(session.app.advance_active_battle_to_next_event_bucket())
                    }
                },
                TrialStep::Assert { from, assertion } => {
                    script::assertion(from, assertion, &session.history)?
                }
            }
        }
        Step::Command {
            document,
            current_battle_authority,
        } => {
            let context = if *current_battle_authority {
                match session.app.active_battle_reconnect(0) {
                    Ok(projection) => match context(&projection) {
                        Ok(context) => context,
                        Err(_) => {
                            return Ok(driver_outcome(
                                "public battle authority projection is unavailable",
                                1,
                            ))
                        }
                    },
                    Err(error) => return Ok(result::<()>(Err(error))),
                }
            } else {
                ApplicationExecutionContext::default()
            };
            let mut outcome = result(session.app.execute(
                session.player,
                command(document.clone())?,
                context,
            ));
            outcome.public_operations += usize::from(*current_battle_authority);
            outcome
        }
        Step::Admin { document } => result(session.app.execute_admin(
            ApplicationAdminCommand::from_document(document.clone().into())?,
        )),
        Step::PrepareDiscard { document } => {
            let prepared = session.app.prepare_command(
                session.player,
                command(document.clone())?,
                ApplicationExecutionContext::default(),
            );
            let digest = match prepared {
                Ok(prepared) => {
                    let digest = semantic_digest(prepared.result());
                    drop(prepared);
                    Ok(digest)
                }
                Err(error) => Err(error),
            };
            result(digest)
        }
        Step::Elapsed {
            delta_ms,
            after_seq,
        } => result(session.app.advance_active_battle_by(*delta_ms, *after_seq)),
        Step::HostTick {
            delta_ms,
            after_seq,
        } => result(
            session
                .app
                .advance_active_battle_for_host_tick(*delta_ms, *after_seq),
        ),
        Step::NextEvent => result(session.app.advance_active_battle_to_next_event_bucket()),
        Step::DisconnectPause => result(session.app.pause_active_battle_for_disconnect()),
        Step::Reconnect { after_seq } => result(session.app.active_battle_reconnect(*after_seq)),
        Step::Snapshot { snapshot } => result(session.app.snapshot(*snapshot)),
        Step::Checkpoint => match session.app.export_active_run_checkpoint() {
            Ok(checkpoint) => {
                let outcome = result(Ok(checkpoint.as_ref().map(|c| {
                    (
                        semantic_digest(&c.core_payload_json()),
                        c.loads_used(),
                        c.run_peak_certified_power(),
                    )
                })));
                session.checkpoint = checkpoint;
                outcome
            }
            Err(error) => result::<()>(Err(error)),
        },
        Step::RestoreCheckpoint => {
            let checkpoint = session
                .checkpoint
                .as_ref()
                .ok_or_else(|| invalid("no checkpoint was exported"))?;
            result(
                session
                    .app
                    .continue_from_active_run_checkpoint(session.player, checkpoint),
            )
        }
        Step::CaptureRollback => match session.app.capture_persistence_rollback() {
            Ok(token) => {
                let present = token.is_some();
                session.rollback = token;
                result(Ok(present))
            }
            Err(error) => result::<()>(Err(error)),
        },
        Step::RestoreRollback => {
            let token = session
                .rollback
                .take()
                .ok_or_else(|| invalid("no rollback token was captured"))?;
            result(session.app.restore_persistence_rollback(token))
        }
        Step::EnterOrdinaryCombat => enter_combat(session),
    };
    Ok(outcome)
}

pub fn execute(manifest_bytes: &[u8], output: &Path) -> Result<Value, Error> {
    let manifest = Manifest::from_bytes(manifest_bytes)?;
    let (inputs, selection_sha256) = load_inputs(&manifest)?;
    let mut export = Export::create(output, manifest.max_export_bytes, manifest.max_samples)?;
    let window = begin_process_window().ok();
    let started = Instant::now();
    let identities: Vec<_> = inputs.iter().zip(&manifest.sessions).map(|(data, s)| json!({"player":s.player,"seed":s.seed,"content":data.identity(),"source_receipt_sha256":semantic_digest(data.source_receipt()),"raw_profile":data.runtime_safety().raw_sha256(),"effective_profile":data.runtime_safety().effective_sha256(),"nominal_warning":data.runtime_safety().nominal_native_runtime().max_retained_bytes,"selected_hard":data.runtime_safety().native_runtime().max_retained_bytes})).collect();
    export.line(&json!({"record":"header","schema_version":1,"case_id":manifest.case_id,"manifest_sha256":format!("{:x}",Sha256::digest(manifest_bytes)),"selection_sha256":selection_sha256,"source_sha256_supplied":manifest.source_sha256,"source_sha256_compiled":option_env!("RUNTIME_MEASUREMENT_SOURCE_SHA256"),"binary":std::env::current_exe()?.file_name().map(|s|s.to_string_lossy().into_owned()),"binary_sha256":executable_sha256(),"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"runtime_measurement":true,"observation":manifest.observation,"full_mode_scope":"deferred drain; queue saturation is observed only when publication exceeds 1024 pending records","concurrency":"interleaved live applications in one process; allocator includes other process threads","sessions":identities,"limits":{"steps":STEP_CAP,"sessions":SESSION_CAP,"repeats":REPEAT_CAP,"samples":manifest.max_samples,"export_bytes":manifest.max_export_bytes,"response_history_records":script::HISTORY_CAP,"response_history_bytes_per_session":script::HISTORY_BYTE_CAP},"scope":"native operation accounting, owner extent, process requested heap and OS RSS are separate; non-native rollback capture heap is included only in process measurements"}), FOOTER_RESERVE);
    let (sender, receiver) = observation::channel();
    let mut receiver = Some(receiver);
    if manifest.observation == ObservationMode::Detached {
        receiver.take();
    }
    let mut accepted = 0usize;
    let mut rejected = 0usize;
    let mut public_operations = 0usize;
    let mut raw_seen = 0usize;
    let mut accepted_bytes = Vec::new();
    let mut step_times = Vec::new();
    let mut final_digest = DigestWriter {
        digest: Sha256::new(),
        written: 0,
        cap: EXPORT_BYTE_CAP,
    };
    let mut final_digest_available = true;
    let mut final_states_seen = 0usize;
    let mut measurement_errors = 0usize;
    let mut stop = false;
    for repeat in 0..manifest.repeats {
        let mut sessions: Vec<_> = inputs
            .iter()
            .zip(&manifest.sessions)
            .enumerate()
            .map(|(i, (data, spec))| {
                let mut app = CoreApplication::new(data.clone(), spec.seed);
                let reader = if manifest.observation != ObservationMode::None {
                    let (observer, reader) = sender.session((repeat * SESSION_CAP + i + 1) as u64);
                    app.set_runtime_observer(Some(observer));
                    Some(reader)
                } else {
                    None
                };
                Session {
                    app,
                    player: spec.player,
                    checkpoint: None,
                    rollback: None,
                    reader,
                    history: script::History::default(),
                }
            })
            .collect();
        for (step_index, step) in manifest.steps.iter().enumerate() {
            for (session_index, session) in sessions.iter_mut().enumerate() {
                let time = Instant::now();
                let operation = apply(session, step);
                let elapsed_ns = u64::try_from(time.elapsed().as_nanos()).unwrap_or(u64::MAX);
                let mut outcome = match operation {
                    Ok(outcome) => outcome,
                    Err(error) => StepOutcome {
                        result_digest: None,
                        error_kind: Some("PlaybackInputOrDriver".into()),
                        error_code: Some(error.to_string().chars().take(256).collect()),
                        error_details_digest: None,
                        public_operations: 0,
                        ..Default::default()
                    },
                };
                public_operations += outcome.public_operations;
                session.history.push(
                    step_index,
                    outcome.public_result.take(),
                    outcome.public_result_bytes,
                );
                if outcome.error_kind.as_deref() == Some("PlaybackInputOrDriver") {
                    measurement_errors += 1;
                }
                if outcome.error_code.is_some() {
                    rejected += 1;
                    stop |= manifest.stop_on_rejection;
                } else {
                    accepted += 1;
                }
                let record = json!({"record":"step","repeat":repeat,"step":step_index,"session":session_index,"outcome":outcome,"wall_ns":elapsed_ns,"identity":session.app.measurement_identity(),"committed":session.reader.as_ref().and_then(|r|r.snapshot(false)),"process_window":process_active_window(),"process_heap":process_heap(),"process_rss":process_rss()});
                if export.sample(&record) {
                    step_times.push(elapsed_ns);
                }
                if manifest.observation == ObservationMode::Normal {
                    if let Some(receiver) = &receiver {
                        drain(receiver, &mut export, &mut raw_seen, &mut accepted_bytes);
                    }
                }
            }
            if stop {
                break;
            }
        }
        for (i, session) in sessions.iter().enumerate() {
            let snapshots: Vec<_> = ApplicationSnapshotKind::ALL
                .iter()
                .map(|kind| session.app.snapshot(*kind))
                .collect();
            public_operations += ApplicationSnapshotKind::ALL.len();
            let final_state = json!({"record":"final_state","repeat":repeat,"session":i,"public_snapshots_sha256":semantic_digest(&snapshots),"identity":session.app.measurement_identity(),"active_battle":session.app.has_active_battle(),"response_history_usage_records_bytes_drops":session.history.usage(),"checkpoint_original_sha256":session.checkpoint.as_ref().map(|c|format!("{:x}",Sha256::digest(c.core_payload_json())))});
            final_states_seen += 1;
            if serde_json::to_writer(&mut final_digest, &final_state).is_err() {
                final_digest_available = false;
            }
            export.sample(&final_state);
        }
        drop(sessions);
        if stop {
            break;
        }
    }
    if let Some(receiver) = &receiver {
        drain(receiver, &mut export, &mut raw_seen, &mut accepted_bytes);
    }
    let counters = receiver.as_ref().map(|r| {
        let counters = r.counters();
        json!({"attempts":counters.attempts.as_slice(),"reasons":counters.reasons.as_slice(),"commits":counters.commits,"dropped":counters.dropped})
    });
    let window = window.map(|w| w.finish());
    let summary = json!({"record":"summary","accepted_steps":accepted,"rejected_steps":rejected,"public_operations":public_operations,"driver_errors":measurement_errors,"wall_ns":started.elapsed().as_nanos(),"records_seen":export.records_seen,"samples_retained":export.retained,"samples_dropped":export.dropped,"raw_operations_seen":raw_seen,"queue_counters":counters,"export_bytes_before_summary":export.bytes,"export_io_failed":export.io_failed,"step_wall_ns":distribution(&mut step_times),"accepted_native_bytes":distribution(&mut accepted_bytes),"process_window":window,"process_heap":process_heap(),"process_rss":process_rss(),"final_states_seen":final_states_seen,"final_states_sha256":final_digest_available.then(||format!("{:x}",final_digest.digest.finalize())),"truncated":export.dropped!=0||export.io_failed,"statistics_scope":"retained exported samples only; queue/export drop counts are separate; p95 is not a guaranteed worst case"});
    let summary_written = export.line(&summary, 0);
    // Reporting failure is diagnostic. All Core operations have already run.
    Ok(
        json!({"summary":summary,"summary_written":summary_written,"summary_missing":!summary_written,"output_bytes":export.bytes,"export_io_failed":export.io_failed}),
    )
}
fn drain(
    receiver: &observation::ObservationReceiver,
    export: &mut Export,
    raw_seen: &mut usize,
    accepted_bytes: &mut Vec<u64>,
) {
    while let Ok(summary) = receiver.try_recv() {
        *raw_seen += 1;
        if export.sample(&json!({"record":"native_operation","summary":summary})) {
            if let Some(retained) = summary.retained.filter(|r| !r.aggregation_overflowed) {
                accepted_bytes.push(retained.accepted_total_bytes);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn manifest() -> Value {
        json!({"schema_version":1,"case_id":"cmrp-test","pack_root":"not-opened","selection_path":"not-opened","source_sha256":"a".repeat(64),"observation":"normal","repeats":1,"sessions":[{"player":Uuid::from_u128(1),"seed":23,"runtime_safety_path":null,"expected_effective_profile_sha256":game_core::runtime_safety::RuntimeSafetySettings::embedded().effective_sha256()}],"steps":[{"kind":"admin","document":{"type":"admin_set_enkephalin","amount":1}}],"max_samples":4096,"max_export_bytes":EXPORT_BYTE_CAP})
    }
    #[test]
    fn cmrp_playback_authority_query_rejection_retains_public_type_and_count() {
        let data: ValidatedGameData = game_core::game::data::GameDataBuilder::empty()
            .build_arc()
            .into();
        let mut session = Session {
            app: CoreApplication::new(data, 1),
            player: Uuid::from_u128(1),
            checkpoint: None,
            rollback: None,
            reader: None,
            history: script::History::default(),
        };
        let outcome = apply(
            &mut session,
            &Step::Command {
                document: json!({"type":"pause_battle"}),
                current_battle_authority: true,
            },
        )
        .unwrap();
        assert_eq!(outcome.public_operations, 1);
        assert_eq!(outcome.error_kind.as_deref(), Some("InvalidAction"));
        assert!(outcome.error_code.is_some());
        assert!(outcome.error_details_digest.is_some());
    }
    #[test]
    fn cmrp_playback_gate_precedes_content_open_and_application_execution() {
        let mut value = manifest();
        value["sessions"][0]["expected_effective_profile_sha256"] = json!("0".repeat(64));
        let parsed = Manifest::from_bytes(&serde_json::to_vec(&value).unwrap()).unwrap();
        let error = load_inputs(&parsed).err().unwrap().to_string();
        assert!(
            error.contains("profile") || error.contains("SHA"),
            "{error}"
        );
        assert!(!error.contains("No such file"));
    }
    #[test]
    fn cmrp_playback_manifest_finite_bounds_and_commands_fail_before_execution() {
        for (field, value) in [
            ("repeats", json!(65)),
            ("max_samples", json!(4097)),
            ("max_export_bytes", json!(EXPORT_BYTE_CAP + 1)),
            ("schema_version", json!(2)),
        ] {
            let mut m = manifest();
            m[field] = value;
            assert!(Manifest::from_bytes(&serde_json::to_vec(&m).unwrap()).is_err());
        }
        let mut m = manifest();
        m["steps"][0] = json!({"kind":"command","document":{"type":"unknown"}});
        assert!(Manifest::from_bytes(&serde_json::to_vec(&m).unwrap()).is_err());
        assert!(Manifest::from_bytes(&vec![b' '; MANIFEST_BYTE_CAP + 1]).is_err());
        let mut m = manifest();
        m["repeats"] = json!(64);
        m["steps"] = json!([{"kind":"enter_ordinary_combat"},{"kind":"enter_ordinary_combat"}]);
        assert!(Manifest::from_bytes(&serde_json::to_vec(&m).unwrap()).is_err());
    }
    #[test]
    fn cmrp_playback_export_create_new_and_saturation_keep_complete_summary() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("samples.jsonl");
        let mut export = Export::create(&path, 2 * RECORD_BYTE_CAP, 2).unwrap();
        assert!(export.sample(&json!({"sample":1})));
        assert!(export.sample(&json!({"sample":2})));
        assert!(!export.sample(&json!({"sample":3})));
        assert_eq!(
            (export.retained, export.dropped, export.records_seen),
            (2, 1, 3)
        );
        assert!(export.line(&json!({"record":"summary","dropped":export.dropped}), 0));
        assert!(Export::create(&path, 2 * RECORD_BYTE_CAP, 2).is_err());
        for line in std::fs::read_to_string(&path).unwrap().lines() {
            serde_json::from_str::<Value>(line).unwrap();
        }
        assert!(std::fs::metadata(&path).unwrap().len() <= 2 * RECORD_BYTE_CAP as u64);
        let mut stream = ByteBuffer {
            bytes: Vec::new(),
            cap: 2,
        };
        assert!(stream.write_all(b"abc").is_err());
        assert!(stream.bytes.is_empty());
        let stats = distribution(&mut [1, 2, 3, 4, 100]);
        assert_eq!(stats.p95, Some(100));
        assert_eq!(stats.mean, Some(22.));
        assert!(distribution(&mut []).p95.is_none());
    }
    fn external_manifest(root: &Path) -> Value {
        let content = root.join("content");
        std::fs::create_dir(&content).unwrap();
        content_pack::package_official(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../game_resources"),
            &content.join("official"),
            &InputLimits::default(),
        )
        .unwrap();
        let selection = root.join("selection.json");
        std::fs::write(&selection, r#"{"packs":["official"],"winners":[]}"#).unwrap();
        let mut m = manifest();
        m["pack_root"] = json!(content);
        m["selection_path"] = json!(selection);
        m
    }
    #[test]
    fn cmrp_playback_normal_external_combat_modes_keep_typed_results_and_gameplay() {
        let directory = tempfile::tempdir().unwrap();
        let mut m = external_manifest(directory.path());
        m["steps"] = json!([
            {"kind":"prepare_discard","document":{"type":"start_new_game","game_mode":"Standard","replace_active_run":false}},
            {"kind":"enter_ordinary_combat"},
            {"kind":"command","document":{"type":"pause_battle"}},
            {"kind":"command","document":{"type":"pause_battle"},"current_battle_authority":true},
            {"kind":"reconnect"},
            {"kind":"command","document":{"type":"resume_battle"},"current_battle_authority":true},
            {"kind":"elapsed","delta_ms":10},
            {"kind":"host_tick","delta_ms":10},
            {"kind":"disconnect_pause"},
            {"kind":"command","document":{"type":"resume_battle"},"current_battle_authority":true},
            {"kind":"checkpoint"},
            {"kind":"command","document":{"type":"retreat_battle"},"current_battle_authority":true}
        ]);
        let mut expected = None;
        for mode in ["none", "normal", "detached", "full"] {
            m["observation"] = json!(mode);
            let bytes = serde_json::to_vec(&m).unwrap();
            let _manifest = Manifest::from_bytes(&bytes).unwrap();
            let output = directory.path().join(format!("{mode}.jsonl"));
            let report = execute(&bytes, &output).unwrap();
            assert_eq!(report["summary_written"], true);
            assert_eq!(report["summary"]["driver_errors"], 0, "{report}");
            assert_eq!(report["summary"]["rejected_steps"], 1, "{report}");
            let current = report["summary"]["final_states_sha256"].clone();
            assert!(!current.is_null());
            if let Some(expected) = &expected {
                assert_eq!(&current, expected, "mode {mode}");
            } else {
                expected = Some(current);
            }
            let records: Vec<Value> = std::fs::read_to_string(&output)
                .unwrap()
                .lines()
                .map(|l| serde_json::from_str(l).unwrap())
                .collect();
            let rejection = records
                .iter()
                .find(|r| r["record"] == "step" && r["step"] == 2)
                .unwrap();
            assert!(!rejection["outcome"]["error_code"].is_null());
            assert!(!rejection["outcome"]["error_details_digest"].is_null());
            if mode == "normal" || mode == "full" {
                assert!(records.iter().any(|r| r["record"] == "native_operation"
                    && r["summary"]["retained"]["accepted_total_bytes"]
                        .as_u64()
                        .is_some_and(|n| n > 0)));
            }
        }
        // A second start is a typed public rejection after the driver's first
        // operation. Its partial progress must retain both type and count.
        let parsed = Manifest::from_bytes(&serde_json::to_vec(&m).unwrap()).unwrap();
        let data = load_inputs(&parsed).unwrap().0.remove(0);
        let mut session = Session {
            app: CoreApplication::new(data, 23),
            player: Uuid::from_u128(1),
            checkpoint: None,
            rollback: None,
            reader: None,
            history: script::History::default(),
        };
        assert!(enter_combat(&mut session).error_code.is_none());
        let rejected = enter_combat(&mut session);
        assert_eq!(rejected.public_operations, 1);
        assert_ne!(
            rejected.error_kind.as_deref(),
            Some("PlaybackInputOrDriver")
        );
        assert!(rejected.error_code.is_some());
        assert!(rejected.error_details_digest.is_some());
    }
    #[test]
    fn cmrp_script_preserved_u07_actor_trap_wall_reach_normal_public_combat() {
        use game_core::authoring::ResultPointer;
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../core_enhanced/docs/goals/core_independent_objects/evidence/public_object_combat_inputs_001");
        for case in ["actor", "small", "ice"] {
            let fixture = root.join(case);
            let scenario: Value =
                serde_json::from_slice(&std::fs::read(fixture.join("scenario.json")).unwrap())
                    .unwrap();
            let selection: ContentSelection =
                serde_json::from_slice(&std::fs::read(fixture.join("selection.json")).unwrap())
                    .unwrap();
            let data = content_pack::load(
                &fixture.join("content"),
                &selection,
                &InputLimits::default(),
            )
            .unwrap();
            let (sender, receiver) = observation::channel();
            let (observer, reader) = sender.session(1);
            let mut app =
                CoreApplication::new(data, scenario["initial"]["run_seed"].as_u64().unwrap());
            app.set_runtime_observer(Some(observer));
            let mut session = Session {
                app,
                player: serde_json::from_value(scenario["initial"]["player_id"].clone()).unwrap(),
                checkpoint: None,
                rollback: None,
                reader: Some(reader),
                history: script::History::default(),
            };
            let raw = scenario["preparation_steps"]
                .as_array()
                .unwrap()
                .iter()
                .chain(scenario["steps"].as_array().unwrap());
            for (index, step) in raw.enumerate() {
                let trial = serde_json::from_value(step.clone()).unwrap();
                script::validate(&trial, index).unwrap();
                let mut outcome = apply(&mut session, &Step::Script { step: trial }).unwrap();
                assert!(
                    outcome.error_code.is_none(),
                    "case {case} step {index}: {}",
                    serde_json::to_string(&outcome).unwrap()
                );
                if step["kind"] == "command" {
                    assert!(outcome.resolved_command.is_some());
                }
                session.history.push(
                    index,
                    outcome.public_result.take(),
                    outcome.public_result_bytes,
                );
            }
            let get = |step, pointer: &str| {
                session
                    .history
                    .get(&ResultPointer {
                        step,
                        pointer: pointer.into(),
                    })
                    .unwrap()
                    .clone()
            };
            let object = get(
                27,
                "/payload/battle_update/checkpoint/independent_objects/active/0",
            );
            let before_withdraw = get(
                27,
                "/payload/battle_update/checkpoint/independent_objects/active",
            );
            assert_eq!(
                get(
                    28,
                    "/payload/battle_update/checkpoint/independent_objects/active"
                ),
                before_withdraw
            );
            let creator = &object["source"]["creator"];
            assert!(get(28, "/payload/battle_update/checkpoint/units")
                .as_array()
                .unwrap()
                .iter()
                .all(|u| &u["unit_instance_id"] != creator));
            let id = &object["object_id"];
            let events = get(31, "/payload/battle_update/events_delta/events");
            let damage: Vec<_> = events
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| {
                    e["event"]["type"] == "HpChanged"
                        && &e["event"]["source_instance_id"] == id
                        && e["event"]["delta"].as_i64().is_some_and(|n| n < 0)
                })
                .collect();
            let removed: Vec<_> = events
                .as_array()
                .unwrap()
                .iter()
                .filter(|e| {
                    e["event"]["type"] == "IndependentObjectRemoved"
                        && &e["event"]["receipt"]["object_id"] == id
                })
                .collect();
            if case == "ice" {
                assert_eq!(object["max_health"], 3);
                assert_eq!(removed.len(), 1);
                assert_eq!(removed[0]["event"]["receipt"]["reason"], "destroyed");
                assert!(get(
                    31,
                    "/payload/battle_update/checkpoint/independent_objects/active"
                )
                .as_array()
                .unwrap()
                .is_empty());
            } else if case == "small" {
                assert_eq!(damage.len(), 1);
                assert_eq!(damage[0]["event"]["final_damage"], 25);
                assert_eq!(removed[0]["event"]["receipt"]["reason"], "attack_consumed");
            } else {
                assert_eq!(damage.len(), 1);
                assert_eq!(damage[0]["event"]["final_damage"], 10);
                assert_eq!(damage[0]["event"]["damage_source"], "BasicAttack");
            }
            let summaries: Vec<_> = std::iter::from_fn(|| receiver.try_recv().ok()).collect();
            assert!(summaries
                .iter()
                .any(|s| s.retained.is_some_and(|r| r.accepted_total_bytes > 0)));
            assert!(summaries.iter().all(|s| s.started_identity.is_some()));
        }
    }
    #[test]
    fn cmrp_playback_actual_queue_full_and_absent_modes_preserve_final_state() {
        let directory = tempfile::tempdir().unwrap();
        let mut m = external_manifest(directory.path());
        m["steps"] = json!((0..1100)
            .map(|i| json!({"kind":"admin","document":{"type":"admin_set_enkephalin","amount":i}}))
            .collect::<Vec<_>>());
        m["max_samples"] = json!(2);
        let mut expected = None;
        for mode in ["none", "normal", "detached", "full"] {
            m["observation"] = json!(mode);
            let bytes = serde_json::to_vec(&m).unwrap();
            let report = execute(
                &bytes,
                &directory.path().join(format!("queue-{mode}.jsonl")),
            )
            .unwrap();
            let summary = &report["summary"];
            assert_eq!(summary["accepted_steps"], 1100);
            assert_eq!(summary["rejected_steps"], 0);
            assert_eq!(summary["driver_errors"], 0);
            assert_eq!(report["summary_written"], true);
            let digest = summary["final_states_sha256"].clone();
            if let Some(expected) = &expected {
                assert_eq!(&digest, expected, "mode {mode}");
            } else {
                expected = Some(digest);
            }
            if mode == "full" {
                assert_eq!(summary["queue_counters"]["dropped"], 76);
                assert_eq!(summary["raw_operations_seen"], 1024);
            } else if mode == "normal" {
                assert_eq!(summary["queue_counters"]["dropped"], 0);
                assert_eq!(summary["raw_operations_seen"], 1100);
            }
        }
    }
    #[test]
    fn cmrp_playback_maximum_sessions_repeats_keep_footer_and_exact_truncation() {
        let directory = tempfile::tempdir().unwrap();
        let mut m = external_manifest(directory.path());
        let template = m["sessions"][0].clone();
        m["sessions"] = json!((1..=SESSION_CAP)
            .map(|i| {
                let mut s = template.clone();
                s["player"] = json!(Uuid::from_u128(i as u128));
                s
            })
            .collect::<Vec<_>>());
        m["repeats"] = json!(REPEAT_CAP);
        m["max_samples"] = json!(2);
        m["max_export_bytes"] = json!(2 * RECORD_BYTE_CAP);
        let bytes = serde_json::to_vec(&m).unwrap();
        let _manifest = Manifest::from_bytes(&bytes).unwrap();
        let output = directory.path().join("maximum.jsonl");
        let report = execute(&bytes, &output).unwrap();
        assert_eq!(report["summary_written"], true, "{report}");
        let summary = &report["summary"];
        assert_eq!(summary["accepted_steps"], 1024);
        assert_eq!(summary["final_states_seen"], 1024);
        assert_eq!(summary["public_operations"], 8192);
        assert_eq!(summary["samples_retained"], 2);
        assert_eq!(summary["samples_dropped"], 3070);
        assert_eq!(summary["records_seen"], 3072);
        assert_eq!(summary["truncated"], true);
        assert!(std::fs::metadata(&output).unwrap().len() <= 2 * RECORD_BYTE_CAP as u64);
        let original = std::fs::read(&output).unwrap();
        assert!(execute(&bytes, &output).is_err());
        assert_eq!(std::fs::read(&output).unwrap(), original);
    }
}
