# Completion Audit

## Conclusions

- Goal Status: `complete`.
- Runtime Assessment: `FAIL`.
- reason for separation: the approved read-only audit and evidence contract are complete, while one S1, six S2 and one S3
  structural findings remain intentionally unremediated.
- final criterion totals: 1 PASS, 1 PARTIAL, 12 FAIL, 0 UNKNOWN, 0 N/A.

## Baseline and contamination control

- baseline ID: B-001
- start: `2026-08-21T23:03:28+09:00 KST`
- HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- exact content scope: `../Cargo.lock`, `Cargo.toml`, `src/lib.rs`, `src/game`, `tests`,
  `../game_resources/data`
- files: 210
- aggregate SHA-256: `e30995cec4249a92c1d3981d4e7b5e64c153dac4b6e5df90d4f7d9fbb510fb61`
- toolchain: rustc/cargo 1.91.1, x86_64 WSL2 Linux
- final check: E-022 reproduced the same HEAD, file count and aggregate. All ADJ-PERSIST-001/ADJ-TRANSPORT-001 hashes
  also matched their first read.

The worktree was already extensively dirty. The audit did not normalize, revert, format or edit production/runtime/test/live
data/server files. Goal documents were already represented by the untracked Goal directory at baseline; the only executable
artifact added by the audit is the E-021 source/binary under `/tmp`.

## First and only broad test baseline

- command: `cargo test --all-targets --all-features`
- evidence: E-002
- outcome: exit 0; 45.50s compile; 10 test binaries
- counts: 1,131 passed, 0 failed, 0 ignored
- distribution: lib 1,080; four fixture-generator bins 0 each; live item 4; live skill 4; RON loading 25;
  skill runtime 14; skill test suite 4
- repeated: no. The production baseline never changed and the plan prohibited no-value repeat runs after all-pass.

Additional gates:

- E-004 `cargo fmt --all -- --check`: PASS.
- E-005 `cargo clippy --all-targets --all-features -- -D warnings`: FAIL, exit 101; lib 82 diagnostics and lib-test
  133 before abort. No broad warning suppression was found.
- E-021 `/tmp` focused scaling probe: PASS as a measurement, not a correctness test. It was pre-noted and did not modify
  the crate.

## C00-C13 final matrix

| ID | verdict | decisive evidence | final rationale |
| --- | --- | --- | --- |
| C00 | FAIL | B-001, E-002/E-004/E-005/E-019/E-020/E-022 | reproducible baseline, broad tests and fmt pass, but strict clippy gate fails |
| C01 | FAIL | A-001~A-005, E-006/E-008/E-020, findings 001/003 | root ownership is mostly clear; public currency and BattleCore authority bypasses remain |
| C02 | FAIL | T-001~T-003, E-006/E-008/E-009/E-010, findings 001/003/004/005/006 | exhaustive gates and safe staged families coexist with constructible invalid state and failed-command mutation |
| C03 | FAIL | E-007/E-009/E-010/E-011/E-019, findings 005/006 | six live commands and invalid-data start violate failed-command equality |
| C04 | FAIL | E-008/E-017, determinism inventory, findings 003/004 | required A/B and insertion probes pass, but public non-finite event construction violates Eq/Ord precondition |
| C05 | FAIL | E-003/E-015/E-016/E-020, findings 008/009 | compiler-enforced enum routing is positive; leaf→behavior coupling and mixed protocol/history owners amplify change |
| C06 | FAIL | E-006/E-009/E-015/E-016, findings 001/005/008 | central adapter/error codes exist; payloads remain unversioned/domain-coupled and projection can follow mutation |
| C07 | FAIL | E-010/E-013/E-014/E-019, data publication map, findings 006/007 | official 24-file path is strict; public alternate publication panics and shares the runtime type |
| C08 | PARTIAL | E-011/E-012, persistence map, GAP-003 | strict V4 core + atomic adjacent store evidence; mandatory broad continuation equivalence and adjacent execution incomplete |
| C09 | PASS | E-003/E-006/E-015/E-016/E-019/E-020, S-003/S-005 | declared player/host/admin actions and 47 result shapes are exhaustively routed; no production NotImplemented path |
| C10 | FAIL | E-007/E-009/E-010/E-018/E-019, findings 005/007/009 | typed errors/non-authoritative trace are positive; partial-error, panic publication and unbounded history remain |
| C11 | FAIL | E-002/E-009/E-010/E-017/E-021, test traceability, GAP-001~004 | broad useful suite, but risk-proportional S1 atomicity/start/continuation/performance guards are missing |
| C12 | FAIL | E-008/E-018/E-021, finding 009, GAP-004 | no history/work cap; fixed two-entry tail cost scales about 10x per 10x retained N |
| C13 | FAIL | E-005/E-007/E-015/E-018/E-020/E-021, RC-01~RC-03 | fixture/production conflation, per-handler transaction adoption and multi-purpose protocol/history owners persist |

The detailed source/range and residual column is authoritative in `EVIDENCE_MATRIX.md`.

## Finding catalog

Active: 8. Candidate/Hypothesis: 0. Rejected: 1. Superseded: 0.

| finding | severity | confidence | concise impact |
| --- | --- | --- | --- |
| RUNTIME-AUDIT-001 | S3 Low | Strong inference | test-only currency setter is production-public and bypasses typed command/admin authority |
| RUNTIME-AUDIT-003 | S2 Medium | Strong inference | standalone BattleCore/RuntimeUnit authority and event injection are publicly mutable |
| RUNTIME-AUDIT-004 | S2 Medium | Strong inference | public NaN-bearing events violate the manual Eq/Ord precondition |
| RUNTIME-AUDIT-005 | S1 High | Strong inference | six battle commands commit authority before future-cursor projection can fail |
| RUNTIME-AUDIT-006 | S2 Medium | Strong inference | StartNewGame mutates roots before fallible RunStart trigger activation |
| RUNTIME-AUDIT-007 | S2 Medium | Confirmed | public alternate GameData construction panics on invalid authoring input |
| RUNTIME-AUDIT-008 | S2 Medium | Strong inference | unversioned result payloads reuse internal/domain serde representation |
| RUNTIME-AUDIT-009 | S2 Medium | Confirmed | each recent battle delta scans total append-only history; retention has no cap |

Rejected history:

- RUNTIME-AUDIT-002: unchecked shop-sale addition looked unsafe, but an immediately preceding checked evaluation uses the
  same immutable amount/quote and prevents the overflow condition. It remains recorded rather than deleted.

## Root-cause consolidation

- RC-01 — fixture/debug construction is not separated by production type/visibility: findings 001/003/004/006/007.
- RC-02 — transaction/preflight/projection ordering is selected per handler rather than enforced at root: findings
  005/006, with 008 as a boundary contributor.
- RC-03 — transport projection and retained history reuse domain representations for multiple responsibilities: findings
  008/009, with 005 sharing projection placement.

No finding was superseded merely because it shares a cause; each retained ID has a distinct trigger and impact.

## Test trust and known gaps

`TEST_TRACEABILITY.md` maps each risk to an executed test/probe and its limit. Unresolved gaps are:

- GAP-001: future-cursor before/after equality for pause/resume/speed/targeting/activation.
- GAP-002: full failed-state equality for missing RunStart trigger.
- GAP-003: canonical original-vs-continued multi-domain command/result/event/final-state equivalence.
- GAP-004: permanent real-scale event-history budget/scaling guard.

These gaps cause C08 PARTIAL and C11/C12 FAIL; they are not hidden as unclassified UNKNOWNs.

## UNKNOWN, N/A and inaccessible scope

- criterion UNKNOWN: none.
- criterion N/A: none.
- Unity consumer/compatibility: explicit exclusion; not accessed.
- server-wide auth/network/config/store behavior and full test suite: not audited. Only the necessary direct persistence,
  behavior adapter and tick/cursor seams were read under separately recorded adjacent snapshots.
- cross-process/cross-platform Rapier and float portability: not executed.
- gameplay policy/rulebook alignment, balance and content quality: explicit exclusion.
- past Goal/audit evidence: deliberately not used.

These exclusions do not block the core verdict. Where they limit a criterion, the matrix uses PARTIAL (C08) or states the
residual alongside independent FAIL evidence.

## Skipped or additional verification

Skipped:

- no repeat of the all-pass broad suite or focused cargo tests on unchanged production bytes;
- no adjacent server test execution, hence no server-wide or C08 PASS claim;
- no generic continuation equivalence beyond existing representative tests (GAP-003);
- no separate-process/platform physics run;
- no lint fixes/recheck, production remediation or policy/data changes.

Additional:

- E-021 was the sole new runtime probe. C12 required representative measured scale, so it compiled a `/tmp` harness against
  the exact all-feature B-001 rlib. With a fixed two-entry tail, 1K/10K/100K histories cost 4.99/47.87/489.66 microseconds
  per call; one 100K full clone copied at least 24MB inline and took 13.76ms in that synthetic environment.

## Out-of-scope follow-up candidates

Priority order based on current severity and root cause, without implementation in this Goal:

1. RUNTIME-AUDIT-005 + GAP-001: unify cursor preflight/staged transaction for all six live command families.
2. RUNTIME-AUDIT-009 + GAP-004: indexed recent deltas, explicit retention/work/byte budgets and a permanent scale guard.
3. RUNTIME-AUDIT-006/007 + GAP-002: validated GameData type-state/typed builder and StartNewGame failure transaction.
4. RUNTIME-AUDIT-003/004/001: private battle authority, finite event input and cfg/test-only setter.
5. RUNTIME-AUDIT-008: versioned explicit result/snapshot DTOs and compatibility fixtures.
6. GAP-003 plus adjacent persistence test execution: broad continuation behavior equivalence.

No new Goal, Master Goal, production fix or test change was created automatically.

## User-visible completion conditions

| condition | evidence/status |
| --- | --- |
| reproducible dirty baseline | B-001 and exact E-022 final match |
| exact first full-suite command/result | E-002, 1,131/0/0 |
| C00-C13 all classified | final matrix above; no UNKNOWN/N/A |
| production entries and owners inventoried | S-001~S-006 and A-001~A-005 |
| major mutation/error order inspected | T-001~T-003, E-007/E-009/E-010/E-011/E-019 |
| determinism inputs/order/identity/time/physics inspected | E-008/E-017 and determinism inventory |
| data/persistence/external DTO inspected | E-010~E-016 and three boundary maps |
| tests mapped to proof/gaps | `TEST_TRACEABILITY.md`, GAP-001~004 |
| every FAIL finding satisfies evidence contract | all 8 Active entries in `FINDINGS.md` contain required fields |
| candidates/rejections/history retained | 0 open candidates; rejected RUNTIME-AUDIT-002 retained |
| immediate-record traceability | E-001~E-022 and chronological notes, including one sequencing deviation disclosed |
| symptoms consolidated by root cause | RC-01~RC-03 |
| remediation kept out of scope | follow-up list only; E-022 proves production bytes unchanged |
| completion vs runtime acceptance separated | Goal complete / Runtime FAIL, stated at top |

## Final handoff

- user decision required for audit completion: none.
- future decision: choose whether to authorize a remediation Goal, preferably starting with RUNTIME-AUDIT-005.
- completion blocker: none. Final documentation integrity and PLAN status checks succeeded.
