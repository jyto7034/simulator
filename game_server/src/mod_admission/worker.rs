use game_core::mod_validation::{
    execute_worker, BuildIdentity, LabPhaseSample, ModValidationError, ModValidationErrorKind,
    WorkerFailure, WorkerMeasurement, WorkerMode, WorkerRequest, WorkerResponse,
};
use std::{io::Read, path::Path};

/// Shared by the normal local launcher and server executable. Worker requests
/// contain captured content and metadata, never issuer secrets or capabilities.
pub fn entry(args: &[String]) -> Result<bool, ModValidationError> {
    let Some(command) = args.first() else {
        return Ok(false);
    };
    if command != "--cmva-build-identity" && command != "--cmva-lab-worker" {
        return Ok(false);
    }
    let binary = std::env::current_exe()?;
    let build = BuildIdentity::current(&binary)?;
    if command == "--cmva-build-identity" {
        if args.len() != 1 {
            return Err(invalid(
                "worker.arguments",
                "build identity takes no arguments",
            ));
        }
        println!(
            "{}",
            serde_json::to_string(&build).map_err(|e| invalid("worker.build", e.to_string()))?
        );
        return Ok(true);
    }
    if args.len() != 3 {
        return Err(invalid(
            "worker.arguments",
            "request path and mode required",
        ));
    }
    let mode = match args[2].as_str() {
        "plain" => WorkerMode::Plain,
        "instrumented" => WorkerMode::Instrumented,
        _ => return Err(invalid("worker.mode", "unknown mode")),
    };
    if (mode == WorkerMode::Instrumented)
        != cfg!(any(
            feature = "runtime-measurement",
            feature = "cmva-lab-measurement"
        ))
    {
        return Err(invalid(
            "worker.mode",
            "compiled instrumentation must match actual mode",
        ));
    }
    let path = Path::new(&args[1]);
    let file = std::fs::File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(invalid("worker.request", "regular request required"));
    }
    let mut bytes = Vec::new();
    file.take(16 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err(invalid("worker.request", "finite request byte ceiling"));
    }
    let request: WorkerRequest =
        serde_json::from_slice(&bytes).map_err(|e| invalid("worker.request", e.to_string()))?;
    // Input bytes, parsed request and eventual result stay live outside measured
    // drop. Their heap is included in each absolute requested-byte baseline.
    let mut collector = Collector::default();
    let failure_binding = (request.nonce, request.candidate_identity.clone());
    let report = match execute_worker(request, mode, build.clone(), &mut collector) {
        Ok(report) => WorkerResponse::Complete(report),
        Err(error) => WorkerResponse::Failed(WorkerFailure {
            schema_version: 1,
            nonce: failure_binding.0,
            mode,
            build,
            candidate_identity: failure_binding.1,
            error,
        }),
    };
    println!(
        "{}",
        serde_json::to_string(&report).map_err(|e| invalid("worker.report", e.to_string()))?
    );
    Ok(true)
}
fn invalid(field: &str, reason: impl Into<String>) -> ModValidationError {
    ModValidationError::new(ModValidationErrorKind::Incomplete, field, reason)
}
#[derive(Default)]
struct Collector {
    #[cfg(any(feature = "runtime-measurement", feature = "cmva-lab-measurement"))]
    heap: Option<crate::runtime_measurement::allocator::HeapWindowGuard<'static>>,
    #[cfg(any(feature = "runtime-measurement", feature = "cmva-lab-measurement"))]
    native: Option<game_core::runtime_observation::LabObservationWindow>,
}
impl WorkerMeasurement for Collector {
    fn begin(&mut self, phase: &'static str) -> Result<(), ModValidationError> {
        #[cfg(any(feature = "runtime-measurement", feature = "cmva-lab-measurement"))]
        {
            self.heap = Some(
                crate::runtime_measurement::begin_process_window()
                    .map_err(|e| invalid(phase, e))?,
            );
            self.native = Some(
                game_core::runtime_observation::begin_lab_window()
                    .ok_or_else(|| invalid(phase, "native window unavailable or nested"))?,
            );
        }
        let _ = phase;
        Ok(())
    }
    fn end(
        &mut self,
        phase: &'static str,
        elapsed_ns: u64,
        authoring_work: u64,
    ) -> Result<LabPhaseSample, ModValidationError> {
        let mut sample = LabPhaseSample {
            phase: phase.into(),
            repeat_index: 0,
            elapsed_ns,
            requested_heap_live_bytes: None,
            requested_heap_peak_bytes: None,
            rss_bytes: None,
            rss_lifetime_peak_bytes: None,
            native_work: 0,
            native_accounting: None,
            authoring_work,
            sample_count: 1,
            dropped_samples: 0,
            missing_samples: 0,
            counter_overflowed: false,
            window_uncertain: false,
        };
        #[cfg(any(feature = "runtime-measurement", feature = "cmva-lab-measurement"))]
        {
            let native = self
                .native
                .take()
                .and_then(|w| w.finish())
                .ok_or_else(|| invalid(phase, "native sample unavailable"))?;
            sample.native_work = native.work;
            sample.native_accounting =
                Some(serde_json::to_value(native).map_err(|e| invalid(phase, e.to_string()))?);
            let heap = self
                .heap
                .take()
                .ok_or_else(|| invalid(phase, "heap window missing"))?
                .finish();
            sample.requested_heap_live_bytes = heap.end.live_requested_bytes;
            sample.requested_heap_peak_bytes = heap.peak_requested_bytes;
            sample.counter_overflowed =
                heap.baseline.counter_overflowed || heap.end.counter_overflowed;
            sample.window_uncertain =
                heap.boundary_updates_unavailable || heap.retiring_hooks_at_end != 0;
            match crate::runtime_measurement::process_rss() {
                crate::runtime_measurement::RssSample::Available {
                    resident_bytes,
                    lifetime_peak_bytes,
                } => {
                    sample.rss_bytes = Some(resident_bytes);
                    sample.rss_lifetime_peak_bytes = Some(lifetime_peak_bytes);
                }
                _ => {}
            }
            if sample.requested_heap_live_bytes.is_none()
                || sample.requested_heap_peak_bytes.is_none()
                || sample.rss_bytes.is_none()
                || sample.rss_lifetime_peak_bytes.is_none()
            {
                sample.missing_samples = 1;
            }
        }
        Ok(sample)
    }
}
