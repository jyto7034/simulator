# R03 TrialSession surface design 033

Use one public opaque wrapper around the existing private TrialState. Preserve its candidate/receipt/provenance commit boundary; add position-bearing failures and borrowed views. The proposal below implements the current Goal GS-CATP-P2G50-001 / GOAL-CATP-01 without adding gameplay or approval policy. It is a bounded design review, not implementation or completion evidence.

Read capture began **2026-09-10 12:52:19.328006 UTC**. Exact source hashes, spans and closing drift are in [the companion JSON](/tmp/r03_trial_session_surface_design_033.json). No source/docs/Git changes or tests/builds were run. Root’s concurrent trace work is outside this design review.

## Minimal public surface

Keep these definitions in existing `authoring/trial.rs`, re-export only public types from `authoring/mod.rs`. TrialState and every native owner remain private. A separate `authoring/view.rs` is optional if the borrowed observation adapters make trial.rs unwieldy.

```rust
pub struct TrialSession { state: TrialState } // no Clone/Serialize/Deref

pub struct TrialFailure {
    // Private fields; borrowed/Copy accessors.
    position: TrialPosition,
    operation_phase: ExecutionPhase,
    error: AuthoringError,
}
pub enum TrialPosition {
    Initial,
    Step { index: usize }, // global index, including preparation steps
    PreparationReady,
    End,
}

pub struct SimulationFailure {
    accepted: Option<TrialSession>,
    failure: TrialFailure,
}

impl TrialSession {
    pub fn prepare(content: AuthoringContent, scenario: TrialScenario,
        limits: TrialLimits, cancel: &CancellationToken)
        -> Result<Self, TrialFailure>;

    pub fn advance(&mut self, cancel: &CancellationToken)
        -> Result<StepView<'_>, TrialFailure>;

    pub fn run_to_end(&mut self, cancel: &CancellationToken)
        -> Result<(), TrialFailure>;

    pub fn replace(&mut self, content: AuthoringContent,
        scenario: TrialScenario, limits: TrialLimits,
        cancel: &CancellationToken) -> Result<ReceiptView<'_>, TrialFailure>;

    pub fn initial_receipt(&self) -> ReceiptView<'_>;
    pub fn ready_receipt(&self) -> ReceiptView<'_>;
    pub fn step(&self, index: usize) -> Option<StepView<'_>>;
    pub fn steps(&self) -> impl ExactSizeIterator<Item = StepView<'_>>;
    pub fn receipt_documents(&self) -> impl Iterator<Item = &str>;
    pub fn next_step_index(&self) -> usize;
    pub fn is_complete(&self) -> bool;
    // Borrowed/Copy identity, scenario initial/hash, accepted usage counters.
}
pub fn simulate(content: AuthoringContent, scenario: TrialScenario,
    limits: TrialLimits, cancel: &CancellationToken)
    -> Result<TrialSession, SimulationFailure>;
```

These are proposed signatures, not current APIs. Promote the existing trusted TrialLimits and trace configuration/finite gap DTOs to public visibility; retain their present validation and keep limits outside scenario JSON. No new limits object is needed for gameplay. Do not expose TrialState, PreparedReceipt’s mutable fields, CoreApplication, native-storage provenance, mutable scenario/limits access, or whole-owner serialization. A small manual Debug showing counts/identity is sufficient if Error implementations require it.

`ReceiptView` holds `&str` and Copy byte counts. `StepView` holds the global index, a private borrowed AcceptedStep, and methods `receipt()` and `public_result() -> Option<&serde_json::Value>`. Neither getter encodes, parses, clones, charges, or allocates. Outer `step(index) == None` means no accepted step; an existing step with `public_result() == None` means a real accepted slot with no public result (assertion or paused HostTick). `Some(Value::Null)` stays distinct. That distinction is required by current resolver behavior (`resolve.rs:78–120,129–140`).

Keep receipt order chronological: **initial → preparation step records → ready → execution step records**. Existing records contains both step sections, while ready is separate (`trial.rs:76–88,182–202,307–339`). A naïve initial/ready/all-records chain changes the recorded chronology. Iterate borrowed slices and chain the existing ready handle; never collect or re-encode the prefix. Ready is infallibly available only because public construction excludes cfg-test `initialize`.

## Concrete owner and failure changes

| Public operation | Existing producer and required change |
| --- | --- |
| prepare | Delegate to `TrialState::prepare_owned` (`trial.rs:97–127`): one scope over initial + complete explicit preparation prefix + actual ready validation + final mandatory receipt. Return a session only after both outer and inner success. |
| replace | Reuse `replace_prepared` (130–142), or call the same prepare wrapper and move the resulting state once. Candidate B starts with fresh provenance, seed, input and limits; failed B is dropped and A is untouched. |
| advance | Reuse 307–339 and prepare_step 342–525. Capture old `records.len()` before the attempt, wrap the original error, and return a borrowed view only after the current all-owner swap. |
| run_to_end | While `next_step_index < preparation_steps.len() + steps.len()`, call advance once. Stop immediately on its first error. No extra scope around this loop, and no additional ready gate after every execution step. |
| simulate | Prepare once, then run_to_end. On failure after successful preparation, move that same last accepted session into SimulationFailure; on preparation failure use accepted=None. |

“Complete” means every remaining **explicit finite scenario step** was accepted. It does not mean “advance until battle terminates.” Existing checked scenario length and clock/work/receipt limits remain authoritative (`scenario.rs:14–94,178–265`; `trial.rs:18–68`). Running-at-end and terminal-at-end are honest distinguishable observations. The input must include the actual clocks/commands needed by a terminal experiment. Do not infer targets, deployment, gameplay commands, resume actions or clock deltas.

Preparation currently returns AuthoringError and loses the failing prefix index. The smallest fix is a fixed Copy `position` local **outside** `run_with_provenance`, updated before initialization, each prefix step and final ready work. Attach it to both the outer execution error and inner domain error; the final acceptance poll may fail after the closure returned (`control.rs:215–253`). Validation/trace-attempt failures before the closure use Initial. Advance already knows its global candidate index; preserve that on quota, execution and output failure. End can retain current `authoring_trial_finished` behavior; run_to_end tests completion before calling advance.

Keep `operation_phase` (preparation/execution) distinct from the **original diagnostic’s phase**, which can be Receipt. Do not replace native code, source, related inputs, site or expected/actual fields with a generic trial error. TrialFailure should provide `error() -> &AuthoringError` and a consuming accessor. Current AuthoringError already owns the structured native variants (`mod.rs:47–86`).

SimulationFailure should expose `accepted() -> Option<&TrialSession>`, `failure() -> &TrialFailure`, and `into_parts(self) -> (Option<TrialSession>, TrialFailure)`. It needs no prefix Vec/Arc clone or new heap owner. A preparation prefix was provisional within a single preparation attempt; do not relabel its discarded records as accepted on failure. During simulation, earlier accepted steps are returned intact and remain retryable at the failed index. Existing accepted_work/copy/receipt/provenance stay those of the last successful step. No new aggregate “completed simulation” receipt is necessary: the final accepted step receipt already exists, and a new mandatory encoding after all commits would introduce a new late-failure boundary.

## Mandatory observation and optional trace views

Current private owners are `TrialObservationView` and `TrialTraceView` (`world/authoring_observation.rs:50–118`), exposed only through crate-private facade methods (`application.rs:852–874`). Add public **borrowed wrappers or borrowed scalar DTOs**, not public native owners:

- Lifecycle/observation key: Copy ordinal, UUID, actual elapsed time/status/verdict; terminal outcome ID as `&str`.
- Mandatory active/terminal observation: Copy counters plus optional borrowed latest witness and latest nonzero sourced HP witness. Use explicit borrowed source strings/IDs and Copy runtime occurrence fields. Reuse the existing source projection mapper so JSON and typed views cannot quietly diverge.
- Optional trace: active and latest-terminal keys plus `enabled`, already encoded NDJSON `&[u8]`, retained row count, last retained ordinal and Copy gap metadata. Existing `TraceView` already has this shape (`trace.rs:160–189`). No Arc or Vec mutation surface.

Expose these via `observation(&self)` / `trace(&self)` returning the public view adapters; retain Result if the internal invariant-checked getter fails. Getter construction must not build a JSON Value, format source strings or serialize a trace. Current terminal’s trace is serde-skipped in the mandatory capsule (`world/authoring_observation.rs:21–38`), and the mandatory ready/step payloads contain only observation. Preserve this separation. Do not place optional trace rows/usage/gaps in binding slots, gameplay fuel accounting or a mandatory receipt.

The wrappers borrow the session; Rust prevents an advance/replace while such a borrow is subsequently used. If a caller needs a durable copy, the caller can explicitly copy returned bytes outside the session operation. Do not introduce public Clone of session/native owners to make views survive replacement. A/B drop behavior then follows existing Arc ownership; externally retained validated tokens remain governed by the existing token contract.

## Describe, Explain, live inspection and transport

Add session convenience `describe` and `explain` methods delegating to the current functions with its borrowed AuthoringContent (`query.rs:57–87,106–111,258–268`). This is needed after prepare consumes that content. Their separate Inspection scope returns the existing owned, already bounded AuthoringQueryResult; `document()` borrows it (`query.rs:17–33`). Inspection usage must not increment session accepted_work or copy/receipt quotas, so repeating queries cannot change the next gameplay acceptance decision.

For a live snapshot/reconnect query, the current facade already offers `snapshot(&self, ApplicationSnapshotKind)` and `active_battle_reconnect(&self, after_seq)` (`application.rs:976–1018`). A session adapter must install an explicitly bounded Inspection scope **before** these first JSON-producing calls and before final document encoding. This is a new query operation with its own limits/cancellation, not a getter that silently materializes a snapshot. Never call the no-scope facade snapshot and then try to bound only its second encoding.

An actual Preview seam is implementable without adding a new evaluator. The current public command kinds include `RequestDeploymentRangePreview` and `PreviewSkillTarget`; their real world handlers borrow &self (`world/combat.rs:2078–2128,2666–2717`). The smallest boundary-preserving adapter accepts an **owned already parsed ApplicationCommand plus explicit ApplicationExecutionContext**, rejects all other kinds, preadmits request serialization, installs a separate Inspection scope seeded with the committed provenance copy, forks through `fork_for_authoring`, executes the ordinary public command with the scenario player ID, prepares a bounded query result, then drops the fork and returned ledger. Do not auto-start tactical focus or repair authority. This preserves current public authority checks and avoids raw world access. A later direct borrowed facade seam is smaller in runtime cost only if it shares the same public context/authority checks; calling the private handlers directly would bypass that boundary. Query output/error does not become a scenario record.

Transport should write existing receipt/document slices verbatim; trace writes existing complete NDJSON rows separately. Iteration plus `Write::write_all` avoids a combined Value, duplicated JSON strings or first serialization in a getter. Newline framing is bounded by finite receipt count and is a transport concern. I/O errors identify the stream/record being delivered and leave the accepted session untouched; they are not gameplay rejection or rollback.

Failure diagnostics are already typed, but are not pre-encoded documents. Provide a **separate explicit bounded diagnostic renderer** borrowing TrialFailure, with independent transport byte/work limits and a result that can report encoding/I/O failure without replacing the original TrialFailure. Run it after the failed gameplay scope has exited, not inside its exhausted/cancelled scope. Apply byte admission before append and never recurse through AuthoringError construction when rendering fails. A renderer failure is transport failure; the caller still owns the typed error and any accepted session. This is the narrow remaining first-diagnostic transport seam, not permission to emit an empty successful trial result.

## Public consumer checks to add

Use the existing standalone release/offline positive and negative probe pattern (`tools/public_api_external_probe.py:22–58`), with explicit external inputs and actual forbidden-API compiler diagnostics.

Positive: owned validate/parse/prepare; actual one-step and finite simulate; exact global slot indexes including None/null; initial/preparation/ready/execution receipt chronology; partial-simulate failure returning the prior session; successful B swap and failed B preservation; readonly query/Preview then identical next command/clock; mandatory bytes unchanged across trace settings; borrowed terminal/source/trace data available without re-encoding. The existing public prototype cannot yet prove these tests pass.

Negative: session Clone/struct literal/private state/CoreApplication borrow, Deref/AsRef bypass, mutable result/receipt/observation access, unprepared initialization, and use of a borrowed view after mutating/replacing/dropping its session. Require the expected privacy/borrow diagnostic rather than accepting unrelated Cargo failure. No new Serialize/Deserialize implementation should reconstruct a TrialSession or a native provenance owner.


Closing source capture: **2026-09-10T12:59:09.274575+00:00**; 2 of 16 inspected source/document hashes changed. Changed after the captured read: \u0060core_enhanced/src/authoring/trial.rs\u0060, \u0060core_enhanced/src/authoring/trace.rs\u0060. Successor content was not reviewed; apply the design against the exact read hashes and reconcile those deltas.
