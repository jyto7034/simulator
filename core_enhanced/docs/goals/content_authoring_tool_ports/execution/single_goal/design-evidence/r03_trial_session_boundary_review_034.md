# R03 public TrialSession boundary review 034

No concrete defect was established in the inspected INPUT080 public boundary. This is a bounded source review, not a completion audit. No source/docs/Git mutation or test/build execution occurred. Focused116 outcomes were neither read nor inferred.

Read capture began **2026-09-10 13:09:14.187982 UTC**. Exact SHA-256 values, inspected spans, INPUT080 comparison and closing drift are recorded in [the companion JSON](/tmp/r03_trial_session_boundary_review_034.json).

## Boundary findings

**Position and diagnostic preservation.** `prepare_positioned` keeps the Copy position outside the scope closure (`trial.rs:404–440`). Original-limit validation and trace-scope entry map to Initial. The control entry poll also occurs while Initial remains set. Each prefix iteration sets its global Step index before calling prepare_step; PreparationReady is set before readiness validation and final receipt. Both outer execution failure and inner AuthoringError are wrapped with the final cursor value. The control boundary runs its final acceptance poll before returning the value/usage/ledger (`control.rs:215–253`), so late cancellation is still returned with PreparationReady. No failure path updates committed usage or provenance. TrialFailure stores the original AuthoringError and provides borrowed/consuming access; operation_phase does not overwrite a diagnostic’s more precise phase (`trial.rs:97–147`).

Public `advance` always labels its operation TrialExecution (`trial.rs:234–248`). That is consistent with the only public constructor: prepare finishes every preparation record plus the actual ready receipt before publishing. The unprepared initializer remains cfg-test/private (`trial.rs:212–219,404–440,461–534`). Public APIs provide no path to remove preparation records or change the scenario. The private advance retains its existing per-index phase calculation and commits only after outer/inner success (621–653).

**Partial simulate and replacement.** `run_to_end` tests the next finite scenario slot and calls exactly one advance each iteration (251–263). It stops on first failure; success means explicit input exhaustion, without extra gameplay or a terminal/victory loop. `simulate` returns accepted=None for preparation failure, and moves the last accepted session into the failure after execution failure (341–389). No prefix collection, owner clone, postcommit encoding or additional gameplay scope was introduced. `into_parts` preserves both owned session and typed failure. `replace` constructs B completely before its single assignment (222–231). The current candidate step append/return and native-storage/usage assignment keep prior A unchanged on failure (621–653,799–837).

**Views and chronology.** ReceiptView borrows an encoded String; its total_bytes is explicitly cumulative. StepView borrows AcceptedStep and returns Option<&Value>; `step(index)` separately indicates whether a record exists (`trial.rs:150–197,264–285`). Getters do not encode, parse, clone or allocate. An absent record, an accepted None result and JSON null are not collapsed by the API.

The receipt iterator is initial → preparation records → ready → execution records (287–300). Its split_at and the ready `expect` rely on the same proved public-construction invariant: successful preparation contains at least the full prefix and a ready receipt; later public advances only append. No safe public reconstruction/mutation bypass was found. Finite indexing uses checked scenario membership rather than an unchecked sum (`scenario.rs:256–263`); checked parsing caps total step count (437–447).

**Authority and settings privacy.** TrialSession owns one private TrialState and has only a small manual Debug implementation. The inspected declarations/re-exports expose no Clone, Deserialize, Serialize, Deref, inner CoreApplication reference, mutable settings or native provenance handle (`trial.rs:77–89,199–339`; `mod.rs:3–24`). Receipt/step views expose immutable borrows; cloning those views copies references, not authority. `limits()` returns &TrialLimits. TraceLimits/TraceOptions became public Copy configuration, while validity checks remain mandatory before setup/advance and TraceAttempt entry (`trial.rs:18–68`; `trace.rs:10–79`). Public visibility did not bypass finite ceiling validation.

**Describe/Explain.** Session methods delegate to the existing content query functions with &self and explicit independent query limits/cancellation (324–337). Query results retain their own bounded document/usage; run_query opens an Inspection scope and never updates TrialState counters (`query.rs:17–33,57–87,106–115,258–268`). The narrow wrappers do not add state mutation or another first-JSON path.

## Oracle limits

The two new test bodies (`trial.rs:996–1164`) meaningfully exercise real prepared public wrappers: paused/assertion None slots versus an absent step, all receipt positions, finite end error and idempotent run_to_end, replay receipt equality, pre-entry cancellation, prefix Step 1 decode error, final readiness rejection, partial simulate failing at global Step 10, repeated failure/failed replacement preservation, and valid seed-changing replacement.

They do not currently force a public-wrapper **receipt-phase** limit failure or **final acceptance** cancellation and assert the preserved cursor plus exact native phase/site. Initial cancellation and no-entry readiness rejection are different boundaries. Minimal follow-up is a reached prefix/ready/final-poll fault with exact position and original diagnostic assertions.

The partial simulation fixture accepts a Run snapshot before an unknown-command decode failure. Preservation assertions compare receipt strings, accepted work and input hash; they are not an independent oracle for gameplay/RNG/continuation/native ledger/copy quota/trace preservation after a late gameplay failure. Extending an existing real gameplay failure fixture through the public wrapper would address this without repeating all private runtime tests.

These cfg-test internal consumers use embedded fixtures. They do not prove standalone default-release privacy/borrow rejection, external-pack A/B generation lifetime or the session Describe/Explain delegates. Their eventual external consumer checks should require the expected forbidden API/borrow diagnostics, not any compilation failure. No test outcome is asserted here.

Known staged public observation/live-inspection/preview, failure renderer and CLI work is excluded and is not filed as a current boundary defect. Whole-goal source/trace/native-bound completeness was not reviewed.

Closing capture: **2026-09-10T13:12:45.591219+00:00**. All 6 reviewed source files match the INPUT080 read baseline. 2 read-to-close changes. Successors not reviewed: `core_enhanced/src/authoring/trial.rs`, `core_enhanced/src/authoring/mod.rs`. The manifest was rehashed as well; exact details remain in JSON.
