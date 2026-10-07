# P02 Validated Data Completion Audit

## GEX-20 Attempt 1 — `Not Complete`

### Audit Identity and Currentness

- Exact Goal: `GOAL-CRAR-C2-P02-VALIDATED-DATA` / `WP-CRAR-C2-04`.
- Audit target: `ATR-CRAR-C2-P02-GEX10-001` / `evidence/P02_AUDIT_TARGET.json`.
- Predecessor: P01 `Complete / High`, current on fresh successor record
  `ATR-CRAR-C2-P01-GEX20-002`.
- Separation: GEX-10 implementation mutation stopped before this read-only audit. This audit attempt wrote only this
  audit document.
- Target currentness: 34/34 member hashes exact, unrelated dirty fingerprint
  `e348f4fd…2caa` exact, revision/branch unchanged; `current / unchanged`.

### Fresh Evidence That Passed

- `cargo check -p game_core --release`: exit 0, warnings 0.
- Valid default-release probe executed actual live publication and `GameCore::new`: exit 0.
- Invalid default-release builder/parts probe failed as expected with `E0432`/`E0603`.
- Fresh semantic parity, invalid rollback and retry/idempotency selectors: 3/3 passed.
- Frozen broad evidence remains current: Core suite 1,132 passed, failed/ignored 0; `ron_loading` 25/25.
- Canonical document and the two N/A dispositions remain aligned with current actual.

### Completion-Blocking Finding

Source-level producer/consumer review found one public bare construction bypass not covered by the first invalid probe:

`CombatExecutor::build_battle_with_combat_preview` is public, accepts `Arc<GameDataBase>`, and returns a newly
constructed `BattleCore` through the crate-private constructor. Therefore production bare construction path count is
not zero even though `GameDataBuilder`, parts/new and the direct `BattleCore` constructor are isolated.

This violates the P02 objective, structural completion condition, `producer_consumer` and
`legacy_fallback_absence` rows. Passing tests and the narrower compile-fail probe cannot replace this source-level
condition.

### Verdict and Return

Verdict on `ATR-CRAR-C2-P02-GEX10-001`: `Not Complete`.

The defect is an in-scope technical correction with unchanged policy/Goal/Phase meaning. The active goal-run returns
to a correction GEX-10 to change the public helper to consume `ValidatedGameData`, extend the negative API probe to
cover bare-Arc conversion/construction, rerun proportional validation, and freeze a new Audit Target. P03 and GEX-30
remain ineligible until a fresh P02 GEX-20 is `Complete / High`.

## GEX-20 Attempt 2 — `Complete / High`

### Replacement Target and Separation

- Replacement target: `ATR-CRAR-C2-P02-GEX10-002` / `evidence/P02_AUDIT_TARGET_002.json`.
- It derives from immutable target 001: 28/34 base members remain exact, six corrected members have new hashes, and
  one `battlefield_plan.rs` member was added. All 35 current identities rehashed exact.
- Unrelated dirty fingerprint `92a03ef…80b9`, counts 141 modified / 24 deleted / 695 untracked, exact at audit entry.
- P01 remains `Complete / High` on `ATR-CRAR-C2-P01-GEX20-002`, currentness
  `current / proven non-impacting delta`: the correction is test-only visibility gating, the 1,132-test current suite
  passes, and the P01 verifier retains the exact 42-content `ecaee174…eab` composite/current receipt.
- Attempt 2 was a fresh read-only pass. It performed only target comparison, source review and validation; this appended
  audit section is excluded audit evidence and no implementation member changed.

### Completion Condition Audit

| Condition | Current direct evidence | Result |
| --- | --- | --- |
| one opaque valid publication | `ValidatedGameData::load_live_embedded`; parsed/published every-catalog semantic parity 1/1 | pass |
| actual producer→consumer | standalone default-release probe loads actual embedded data and constructs `GameCore`; exit 0 | pass |
| invalid pre-authority failure | malformed/reference/range live matrix plus focused invalid equipment candidate; recoverable `LiveRonError`, no token/Core | pass |
| rollback / partial publication 0 | invalid equipment test preserves installed official publication; later official load remains exact | pass |
| retry/idempotency | two full valid publications compare every catalog; identical invalid attempts return equal errors; post-failure publication equal | pass |
| production bare construction 0 | release source scan: public Core constructors accept `impl Into<ValidatedGameData>` only; bare loader/new/held Arc seams private | pass |
| fixture capability isolated | four fixture binaries require non-default `test-fixtures`; fixture builders/conversion absent from default release | pass |
| four-way negative API audit | expected release failure: builder `E0432`, parts `E0603`, bare Arc conversion `E0277`, removed combat helper `E0599` | pass |
| typed operating mutation | no data setter/hot reload introduced; existing typed Core `AdminCommand` remains the operating seam | pass |
| valid gameplay parity | current Core suite 1,132 passed; `ron_loading` 25/25; combat setup module 14/14 | pass |
| canonical documentation | `docs/data_loading_contract.md` names opaque Core entrypoint, 24 sources, fail-closed publication and fixture gates | pass |
| legacy/fallback absence | old loader reference 0; public bare combat helper 0; fallback/default/dual publication path 0 | pass |

The Attempt 1 finding is directly closed: `build_battle_with_combat_preview`, its bare-Arc handoff and its two
preview-only assembly helpers are private `cfg(test)` code and do not exist in release.

### Validation Dimension Audit

| Dimension | Audited evidence / N/A basis | Result |
| --- | --- | --- |
| `success` | actual release publication→`GameCore`, catalog parity, 1,132-test suite | pass |
| `invalid_input` | malformed, missing, reference, range and unsupported manifest/equipment source rejection | pass |
| `failure_rollback` | failed candidate cannot yield token/Core and does not alter before/after official publication | pass |
| `retry_idempotency` | explicit repeated valid/invalid publication test with full semantic equality/equal error | pass |
| `ordering_concurrency` | startup API has no reload/publisher owner, shared mutable generation or ordering contract | N/A confirmed |
| `boundary_empty_max` | current live suite covers empty source/catalog restrictions, missing required fields and numeric maxima/overflow | pass |
| `disconnect_restart_persistence` | P02 changes no session, checkpoint, save schema or lifetime | N/A confirmed |
| `live_data` | actual embedded 24-source bundle runs in unit/integration/standalone release paths | pass |
| `producer_consumer` | actual token consumed by public `GameCore`/`BattleCore`; mock/builder cannot satisfy default release | pass |
| `legacy_fallback_absence` | source/API/feature audit and four negative compiler errors; old/public bypass count 0 | pass |

No required row is replaced by a happy-path-only test or unsupported N/A.

### Implementation Quality and Structural Review

- Production behavior contains no test-name, fixture identity, mock/probe or special-value branch.
- Invalid authoring is not normalized or defaulted. The loader builds no externally reachable partial candidate and
  returns source-aware errors.
- Validation/data authority is not duplicated in a consumer; the opaque token is created only in the data layer.
- No future registry, shim, compatibility alias, fallback bundle, hot reload or dual construction authority was added.
- Test capability code is explicitly gated; default release compiler errors are direct evidence, not a textual claim.
- Gameplay algorithms and authored data meaning are unchanged; the full current oracle passes.
- Root release builds warning-free. Whole-package Clippy still stops on a pre-existing denied `never_loop` in unrelated
  `boss_runtime_data.rs` plus broad dirty warnings; it is recorded rather than changed or suppressed. The dedicated
  P01 support workspace passes strict Clippy. This baseline does not omit a required P02 condition.

### Scope, Dirty Work and Limitations

- Every tracked implementation path was already dirty; only exact validated-publication, visibility, test call-site
  and documentation hunks are attributed to P02. Unrelated modifications/deletions/untracked work remain preserved.
- External consumers, server/Unity/admin compatibility, deployment and release claims were neither inspected nor used.
- Transaction/lifecycle, battle finite-kernel, checkpoint, protocol and final performance work remain later-Phase
  obligations. P02 makes no performance PASS claim.
- Completion-blocking omission, inaccessible evidence, unchecked production dependency, policy question or unsafe
  overlap: none.

### Final P02 Verdict and Next Boundary

`GOAL-CRAR-C2-P02-VALIDATED-DATA` is `Complete / High` on
`ATR-CRAR-C2-P02-GEX10-002`, currentness `current / unchanged` at audit exit.

The strict sequence now makes P03 `GOAL-CRAR-C2-P03-APPLICATION-AUTHORITY` eligible for its own GEX-00. Attempt 1's
`Not Complete` verdict remains immutable historical evidence and is not overwritten by this replacement-target
verdict. P02 completion does not complete or pre-authorize P03–P08.
