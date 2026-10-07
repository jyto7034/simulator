# P08 Performance and Final Integration Experiments

## 2026-08-23 — GEX-00 Intake

- Run mode: full Goal Set `goal-run`.
- Selected member: `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION)`.
- Internal order is mandatory: `WP-CRAR-C2-09` before `WP-CRAR-C2-10`.
- P01 through P07 completion/currentness evidence was read before selection; all prerequisites are `Complete / High`.
- Existing exact-identity P08 execution directory/tool/receipt: none. Canonical execution directory
  `execution/p08_performance_final_integration` selected.
- Production/source mutation during intake: none.

### Baseline and Final-Boundary Trace

- Resolved the accepted P01 baseline to root `evidence/P01_PERFORMANCE_BASELINE.json` (schema/workload `2` /
  `crar-preservation-baseline/2`). Its receipt and tool hashes match the manifest and P01 audit. The stopped historical
  schema-1 receipt was explicitly excluded.
- Recorded all ten P01 raw samples and exact deterministic identities. Baseline nearest-rank five-sample p95 is
  23.407 ms event-heavy and 14.117 ms for 64-session/8-way; these are observations, not P08 passes.
- Confirmed the P01 support workload measures the direct battle kernel through the existing `test-fixtures` capability.
  The current default-release public root is instead `CoreApplication`; final evidence therefore needs both exact raw
  workload comparison and additional actual-façade serialization/checkpoint/staging measurements.
- Confirmed TD-018 history constants are currently `8,192`, `4 * 1024 * 1024`, and `180_000 ms`, with indexed cursor
  validation/partition query and typed old-cursor resync semantics.
- Confirmed current application transactions stage a complete cloned root. This is an observation and candidate
  profiler target, not yet a defect verdict; GEX-10 must measure retained-history scaling before changing it.
- No benchmark, test, profiler or optimization was run in GEX-00. No external source, schema, fixture or consumer was
  inspected.

### GEX-00 Result

- Result: `passed`.
- No stop condition, upstream ambiguity or unsafe dirty overlap blocks a dedicated standalone harness and exact Core
  hotspot hunks.
- Next authorized work: separated GEX-10, beginning only with `WP-CRAR-C2-09` current measurement and retained raw
  receipt. `WP-CRAR-C2-10` remains ineligible until every performance gate passes.

## 2026-08-23 — GEX-10 / WP-CRAR-C2-09 Performance Stabilization

### Entry Measurement and Hotspot Isolation

- Built the standalone `tools/runtime_architecture_final_validation` workspace. `p01_exact_replay` includes the
  immutable P01 source; `final_performance` independently binds receipt/source/input identities, release profile,
  affinity, 21 latency samples, five allocation samples, nearest-rank p95 and exact one-sided comparison.
- Preserved the unoptimized raw entry receipt. Event-heavy `25.735434 ms` and concurrent `13.486429 ms` failed; exact
  output parity passed. Diagnostic A/B receipts isolated blocker work as the event-heavy hotspot and redundant movement
  ordering as the concurrent hotspot.
- Added the actual public boundary guard using a live embedded-data `CoreApplication`: start, briefing, starter
  selection, SavePoint, checkpoint, selectable Combat entry and rejected in-battle map command. Initial V5 export
  `123.394 µs` failed; after caching, pre-COW 8,192-entry staging `176.113 µs` still failed scaling.

### Production Corrections

- Event log: allocation-free JSON length counter, deferred conservative encoded accounting with exact reconciliation,
  exact reserve/cap cache, and COW entry storage. Added clone-sharing/mutation-isolation and exact serde tests.
- Result path: scenario-aware employee accumulator omission where no Employee can exist; no result meaning changed.
- Kernel hot paths: eliminated redundant movement order materialization, preallocated deterministic maps/outputs,
  bypassed empty static-obstacle queries, skipped unchanged Rapier body writes, streamed deterministic target minima,
  short-circuited structurally impossible blocker scans and retained the full zero-base usefulness preview path.
- Checkpoint: cached canonical V5 bytes in private serde-skipped checkpoint state after validation/save/continue; export
  reuses exact bytes with the same miss encoder.
- Public verifier fixture: feature-gated history installation only; dedicated default-release E0599 probe proves absence.

### Frozen WP-09 Evidence

| Receipt | Direct result | Verdict |
| --- | --- | --- |
| `P08_FINAL_PERFORMANCE_LATENCY.json` | 10.818511 ms / 8.343780 ms p95; exact outputs; p=1.0 | pass |
| `P08_FINAL_PERFORMANCE_ALLOCATION.json` | all operation/peak/retained maxima no worse than P01 | pass |
| `P08_FINAL_P01_EXACT_RESULT.json` | immutable source replay; both hashes exact | pass |
| `P08_PUBLIC_PERFORMANCE_GUARDS.json` | snapshot/checkpoint/query/staging time+size+rollback | pass |
| `P08_PUBLIC_ALLOCATION_GUARDS.json` | low/high staging allocation observations exactly equal | pass |

`WP-CRAR-C2-09` result: `passed`. `GAP-CRAR-P2G20-002` is closed. Only after these five receipts were frozen did
`WP-CRAR-C2-10` become eligible.

## 2026-08-23 — GEX-10 / WP-CRAR-C2-10 Final Reconciliation

- Reconciled `CRAR-LEGACY-008` from the absent `pub entries: Vec<BattleEventLogEntry>` locator to the current bounded
  `BattleEventEntries` replacement. Manifest verifier: 57 features, 57 headings, 14 bundles, 10 dimensions, 10 legacy
  targets, 44 locator contents, composite `5ff7356c…620`, errors 0.
- Fresh release focused selectors: event log 8/8, result statistics 7/7, Rapier backend 17/17, checkpoint validation
  15/15 and application/authority 9/9.
- Fresh full `cargo test -p game_core --all-targets`: library 1,108 plus integrations 51 = 1,159 passed; failed/ignored 0.
- Formatting/diff check, release library check and release rustdoc each exit 0 with no warnings.
- Five default-release valid API callers compile. Five established invalid callers fail with the expected privacy/error
  families. The additional P08 fixture-method invalid caller fails E0599; release rustdoc fixture count is zero.
- Static audit: public raw root/kernel/behavior/result/rollback authority 0; unbounded event-log field 0; V4 serializer
  0; runtime test/benchmark switches 0; competing application commit owner 0; compatibility fallback 0. The explicit
  P06-certified V4 importer remains and is not counted as a writer/fallback.
- Refreshed direct P01–P07 successor-currentness records `_008/_007/_006/_005/_004/_002/_001`, each
  `Complete / High`. No external source, schema, runner or consumer was inspected or used.

`WP-CRAR-C2-10` result: `passed`. Ten policy acceptances and ten validation dimensions are each 10/10; failed,
missing and unknown counts are zero. The next action is to freeze `ATR-CRAR-C2-P08-GEX10-001`, stop mutation and run a
fresh read-only GEX-20 audit.
