//! Common finite authoring CLI; all gameplay remains behind the public API.

use std::{
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use game_core::{
    authoring::{
        self, AmountBattle, AmountQuery, AuthoringError, AuthoringQueryResult, BattleKey,
        BattleTraceView, CancellationToken, DiagnosticLimits, ExecutionPhase, ExplainQuery,
        RequestFailure, ScenarioLimits, SimulationFailure, TraceGap, TraceLimits, TraceOptions,
        TrialLimits, TrialScenario, TrialSession,
    },
    game::data::content_pack::ContentSelection,
};
use serde::{de::DeserializeOwned, Serialize};

const USAGE: &str = "usage: author_content <validate|describe|explain|simulate> TRUSTED_ROOT SELECTION.json [REQUEST.json] [--amount-observation] [--trace-output NEW_PATH] [--runtime-safety PATH]\n";
const SMALL_INPUT: usize = 64 * 1024;

#[derive(Clone, Copy)]
enum Operation {
    Validate,
    Describe,
    Explain,
    Simulate,
}
struct Arguments {
    operation: Operation,
    root: PathBuf,
    selection: PathBuf,
    request: Option<PathBuf>,
    trace_output: Option<PathBuf>,
    amount_observation: bool,
    runtime_safety: Option<PathBuf>,
}
impl Arguments {
    fn parse() -> Result<Self, Failure> {
        let args: Vec<OsString> = std::env::args_os().skip(1).take(11).collect();
        if args.len() < 3
            || args.len() > 9
            || args.iter().any(|arg| arg.as_encoded_bytes().len() > 4096)
        {
            return Err(Failure::Input(USAGE));
        }
        let operation = match args[0].to_str() {
            Some("validate") => Operation::Validate,
            Some("describe") => Operation::Describe,
            Some("explain") => Operation::Explain,
            Some("simulate") => Operation::Simulate,
            _ => return Err(Failure::Input(USAGE)),
        };
        let needs_request = matches!(operation, Operation::Explain | Operation::Simulate);
        let expected = if needs_request { 4 } else { 3 };
        if args.len() < expected {
            return Err(Failure::Input(USAGE));
        }
        let mut runtime_safety = None;
        let mut trace_output = None;
        let mut amount_observation = false;
        let mut index = expected;
        while index < args.len() {
            if args[index] == "--runtime-safety" {
                if runtime_safety.is_some() || index + 1 >= args.len() {
                    return Err(Failure::Input(USAGE));
                }
                runtime_safety = Some(PathBuf::from(&args[index + 1]));
                index += 2;
            } else if args[index] == "--amount-observation"
                && matches!(operation, Operation::Simulate)
            {
                if amount_observation {
                    return Err(Failure::Input(USAGE));
                }
                amount_observation = true;
                index += 1;
            } else if args[index] == "--trace-output" && matches!(operation, Operation::Simulate) {
                if trace_output.is_some() || index + 1 >= args.len() {
                    return Err(Failure::Input(USAGE));
                }
                trace_output = Some(PathBuf::from(&args[index + 1]));
                index += 2;
            } else {
                return Err(Failure::Input(USAGE));
            }
        }
        Ok(Self {
            operation,
            root: (&args[1]).into(),
            selection: (&args[2]).into(),
            request: needs_request.then(|| PathBuf::from(&args[3])),
            trace_output,
            amount_observation,
            runtime_safety,
        })
    }
}

enum Success {
    Document(AuthoringQueryResult),
    Session {
        session: TrialSession,
        amount_documents: Vec<AuthoringQueryResult>,
    },
}
enum Failure {
    Input(&'static str),
    RuntimeSafety(game_core::runtime_safety::RuntimeSafetyError),
    Authoring(AuthoringError),
    Simulation(SimulationFailure),
    AmountQuery {
        session: TrialSession,
        error: AuthoringError,
    },
}
impl From<AuthoringError> for Failure {
    fn from(value: AuthoringError) -> Self {
        Self::Authoring(value)
    }
}

fn read_bounded(path: &Path, limit: usize) -> Result<Vec<u8>, Failure> {
    let metadata =
        fs::metadata(path).map_err(|_| Failure::Input("request file metadata cannot be read"))?;
    if !metadata.is_file() || metadata.len() > limit as u64 {
        return Err(Failure::Input(
            "request must be a regular file within its byte ceiling",
        ));
    }
    let file = File::open(path).map_err(|_| Failure::Input("request file cannot be opened"))?;
    if !file
        .metadata()
        .map_err(|_| Failure::Input("opened request metadata cannot be read"))?
        .is_file()
    {
        return Err(Failure::Input("opened request is not a regular file"));
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(limit + 1)
        .map_err(|_| Failure::Input("request buffer allocation refused"))?;
    file.take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| Failure::Input("request file cannot be read"))?;
    if bytes.len() > limit {
        return Err(Failure::Input("request file exceeds its byte ceiling"));
    }
    Ok(bytes)
}
fn read_small<T: DeserializeOwned>(path: &Path) -> Result<T, Failure> {
    serde_json::from_slice(&read_bounded(path, SMALL_INPUT)?)
        .map_err(|_| Failure::Input("selection or explanation JSON does not match its schema"))
}

fn execute(args: &Arguments) -> Result<Success, Failure> {
    let settings = if let Some(path) = &args.runtime_safety {
        let bytes = read_bounded(path, game_core::runtime_safety::MAX_SETTINGS_BYTES).map_err(
            |failure| {
                let message = match failure {
                    Failure::Input(message) => message,
                    _ => "settings file could not be read",
                };
                Failure::RuntimeSafety(game_core::runtime_safety::RuntimeSafetyError {
                    source: path.display().to_string(),
                    field: "$".into(),
                    message: message.into(),
                })
            },
        )?;
        std::sync::Arc::new(
            game_core::runtime_safety::RuntimeSafetySettings::from_ron(
                &path.display().to_string(),
                &bytes,
            )
            .map_err(Failure::RuntimeSafety)?,
        )
    } else {
        game_core::runtime_safety::RuntimeSafetySettings::embedded_shared()
    };
    let selection: ContentSelection = read_small(&args.selection)?;
    // Parse the entire finite request before spending work validating content.
    let explanation: Option<ExplainQuery> = if matches!(args.operation, Operation::Explain) {
        Some(read_small(
            args.request.as_deref().expect("parsed request"),
        )?)
    } else {
        None
    };
    let scenario = if matches!(args.operation, Operation::Simulate) {
        let limits = ScenarioLimits::default();
        Some(TrialScenario::from_json_slice(
            &read_bounded(
                args.request.as_deref().expect("parsed request"),
                limits.max_input_bytes,
            )?,
            &limits,
        )?)
    } else {
        None
    };
    let cancellation = CancellationToken::default();
    let limits = authoring::AuthoringLimits::with_runtime_safety(&settings);
    let content = authoring::validate_with_runtime_safety(
        &args.root,
        &selection,
        &limits.input,
        &cancellation,
        settings.clone(),
    )?;
    let document = match args.operation {
        Operation::Validate => {
            authoring::validation_receipt(&content, &limits.execution, &cancellation)?
        }
        Operation::Describe => authoring::describe(&content, &limits.execution, &cancellation)?,
        Operation::Explain => authoring::explain(
            &content,
            explanation.as_ref().expect("parsed explanation"),
            &limits.execution,
            &cancellation,
        )?,
        Operation::Simulate => {
            let mut trial_limits = TrialLimits::with_runtime_safety(&settings);
            if args.trace_output.is_some() {
                trial_limits.trace = TraceOptions {
                    limits: Some(TraceLimits::default()),
                };
            }
            trial_limits.amount_observation = args.amount_observation;
            let session = authoring::simulate(
                content,
                scenario.expect("parsed scenario"),
                trial_limits,
                &cancellation,
            )
            .map_err(Failure::Simulation)?;
            let mut amount_documents = Vec::new();
            if args.amount_observation {
                let query_limits = authoring::ExecutionLimits::default();
                let queried = (|| -> Result<Vec<AuthoringQueryResult>, AuthoringError> {
                    let mut documents = Vec::new();
                    for battle in [AmountBattle::Active, AmountBattle::LastTerminal] {
                        let query = AmountQuery::Latest { battle };
                        documents.push(session.preview_amount(
                            query,
                            &query_limits,
                            &cancellation,
                        )?);
                        documents.push(session.explain_amount(
                            query,
                            &query_limits,
                            &cancellation,
                        )?);
                    }
                    Ok(documents)
                })();
                amount_documents = match queried {
                    Ok(documents) => documents,
                    Err(error) => return Err(Failure::AmountQuery { session, error }),
                };
            }
            return Ok(Success::Session {
                session,
                amount_documents,
            });
        }
    };
    Ok(Success::Document(document))
}

fn line(output: &mut impl Write, document: &str) -> io::Result<()> {
    output.write_all(document.as_bytes())?;
    output.write_all(b"\n")
}

fn accepted_session(result: &Result<Success, Failure>) -> Option<&TrialSession> {
    match result {
        Ok(Success::Session { session, .. }) => Some(session),
        Err(Failure::Simulation(failure)) => failure.accepted(),
        Err(Failure::AmountQuery { session, .. }) => Some(session),
        _ => None,
    }
}

// All header fields are Copy/fixed values; no source string is formatted here.
#[derive(Serialize)]
struct TraceHeader {
    kind: &'static str,
    window: &'static str,
    key: BattleKey,
    enabled: bool,
    retained_rows: u64,
    last_retained_ordinal: Option<u64>,
    gap: Option<TraceGap>,
}
struct HeaderBuffer {
    bytes: [u8; 1024],
    len: usize,
}
impl Write for HeaderBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let next = self
            .len
            .checked_add(bytes.len())
            .ok_or(io::ErrorKind::InvalidData)?;
        if next > self.bytes.len() {
            return Err(io::ErrorKind::InvalidData.into());
        }
        self.bytes[self.len..next].copy_from_slice(bytes);
        self.len = next;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn trace_window(
    output: &mut impl Write,
    window: &'static str,
    view: BattleTraceView<'_>,
) -> io::Result<()> {
    let header = TraceHeader {
        kind: "trace_window",
        window,
        key: view.key(),
        enabled: view.enabled(),
        retained_rows: view.retained_rows(),
        last_retained_ordinal: view.last_retained_ordinal(),
        gap: view.gap(),
    };
    let mut buffer = HeaderBuffer {
        bytes: [0; 1024],
        len: 0,
    };
    serde_json::to_writer(&mut buffer, &header).map_err(io::Error::other)?;
    output.write_all(&buffer.bytes[..buffer.len])?;
    output.write_all(b"\n")?;
    output.write_all(view.ndjson())
}
fn export_trace(path: &Path, session: &TrialSession) -> io::Result<()> {
    let view = session.trace().map_err(|_| io::ErrorKind::InvalidData)?;
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(b"{\"kind\":\"trace_windows\",\"schema_version\":1,\"retention\":\"active_and_last_terminal\"}\n")?;
    if let Some(terminal) = view.last_terminal() {
        trace_window(&mut file, "last_terminal", terminal)?;
    }
    if let Some(active) = view.active() {
        trace_window(&mut file, "active", active)?;
    }
    file.flush()
}

fn finish(result: Result<Success, Failure>, trace_path: Option<&Path>) -> u8 {
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    let mut transport_failed = false;
    if let Ok(Success::Document(document)) = &result {
        transport_failed |= line(&mut stdout, document.document()).is_err();
    }
    if let Some(session) = accepted_session(&result) {
        for receipt in session.receipt_documents() {
            if line(&mut stdout, receipt).is_err() {
                transport_failed = true;
                break;
            }
        }
        if let Ok(Success::Session {
            amount_documents, ..
        }) = &result
        {
            for document in amount_documents {
                if line(&mut stdout, document.document()).is_err() {
                    transport_failed = true;
                    break;
                }
            }
        }
        if let Some(path) = trace_path {
            transport_failed |= export_trace(path, session).is_err();
        }
    }
    let code = match &result {
        Ok(_) => 0,
        Err(failure) => {
            let limits = DiagnosticLimits::default();
            let cancellation = CancellationToken::default();
            let rendered = match failure {
                Failure::Authoring(error) => authoring::render_error(error, &limits, &cancellation),
                Failure::Simulation(error) => {
                    authoring::render_trial_failure(error.failure(), &limits, &cancellation)
                }
                Failure::AmountQuery { error, .. } => {
                    authoring::render_error(error, &limits, &cancellation)
                }
                Failure::RuntimeSafety(error) => authoring::render_error(
                    &AuthoringError::Request {
                        diagnostic: RequestFailure {
                            code: "runtime_safety_settings_invalid",
                            phase: ExecutionPhase::Acquisition,
                            field_path: "runtime_safety",
                            reason: error.to_string(),
                        },
                    },
                    &limits,
                    &cancellation,
                ),
                Failure::Input(reason) => authoring::render_error(
                    &AuthoringError::Request {
                        diagnostic: RequestFailure {
                            code: "authoring_cli_input",
                            phase: ExecutionPhase::Inspection,
                            field_path: "cli",
                            reason: (*reason).into(),
                        },
                    },
                    &limits,
                    &cancellation,
                ),
            };
            match rendered {
                Ok(document) => transport_failed |= line(&mut stderr, document.document()).is_err(),
                Err(_) => {
                    transport_failed = true;
                    let _ = stderr.write_all(b"author_content: diagnostic rendering failed\n");
                }
            }
            if matches!(failure, Failure::Input(_) | Failure::RuntimeSafety(_)) {
                2
            } else {
                1
            }
        }
    };
    transport_failed |= stdout.flush().is_err();
    if transport_failed {
        let _ = stderr.write_all(b"author_content: transport output failed\n");
    }
    transport_failed |= stderr.flush().is_err();
    if transport_failed {
        3
    } else {
        code
    }
}

fn main() -> ExitCode {
    match Arguments::parse() {
        Ok(args) => ExitCode::from(finish(execute(&args), args.trace_output.as_deref())),
        Err(error) => ExitCode::from(finish(Err(error), None)),
    }
}
