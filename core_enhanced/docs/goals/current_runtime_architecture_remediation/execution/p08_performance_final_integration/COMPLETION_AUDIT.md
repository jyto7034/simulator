# P08 Performance and Final Integration Completion Audit

Current verdict: `Complete / High — successor GEX-20 audit 001`

## Audit Identity and Separation

- Handoff / Goal Set: `HA-CRAR-P2G60-002` / `GS-CRAR-P2G50-002`.
- Audited member: `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION)`; Phase / nodes
  `PHASE-CRAR-C2-08-PERFORMANCE-FINAL-INTEGRATION` / `WP-CRAR-C2-09`, `WP-CRAR-C2-10`; Goal Quality Contract `2`.
- Policy / decision: `PER-CRAR-008-R6-CODE-REVIEWED`, `DR-CRAR-002-R1`; realizations `TD-CRAR-018/019`.
- Frozen GEX-10 target: `ATR-CRAR-C2-P08-GEX10-001` / `evidence/P08_AUDIT_TARGET.json`, SHA-256
  `a3486eca50fc9f0f9685ece4e31609fa904eb972afb81bbe5610dfa6d43bed9e`.
- GEX-20 began only after the 95-member target was published. It changed no frozen member; temporary receipts, build
  output and this explicitly excluded completion document are outside the target.

## Target, Scope and Predecessor Currentness

- Repository revision/branch remains `f318414fb0156a67e04c5479000c2a2683fc0c09` /
  `cleanup/simulator-dirty-worktree-20260803`.
- Target JSON parses; declared member count is exact; all 95/95 hashes match before and after fresh GEX-20 validation.
- Core-only unrelated dirty baseline remains exact after cumulative task-owned exclusions: 92 modified, 23 deleted,
  675 untracked, total 790, fingerprint `d9e815c7…d622`. Existing user work was neither normalized nor discarded.
- P01 `_008`, P02 `_007`, P03 `_006`, P04 `_005`, P05 `_004`, P06 `_002` and P07 `_001` are direct
  `Complete / High` successor-currentness records.
- `game_server`, Unity, sibling resources, external consumers, deployment and product SLA evidence are excluded and
  were not inspected.

## Performance Completion Audit

| Condition | Frozen direct evidence | Result |
| --- | --- | --- |
| immutable baseline binding | receipt `86ccd5c7…5848`, source `64debfd6…c631`, schema/workload 2 exact | pass |
| event-heavy absolute/relative | 21 standard samples; 10.818511 ms p95 <25 ms; p=1.0; no supported regression | pass |
| concurrent absolute/relative | 21 standard samples; 8.343780 ms p95 <10 ms; p=1.0; no supported regression | pass |
| deterministic identity | 186/65,720/`2ada19d4…333a`; 11,840/3,881,088/`68da2cf9…ca9` | pass |
| direct heap evidence | five observer samples/workload; allocation, reallocation, peak and retained all ≤ P01 | pass |
| public snapshot | live façade 226.508 µs p95, 48,493 bytes, exact repeatable | pass |
| public V5 checkpoint | live façade 8.344 µs p95, 17,956 bytes, exact repeatable | pass |
| bounded query | newest ten at 1,024/8,192: 0.202/0.212 µs p95; exact three-cap identity | pass |
| staging time | rejected command at 128/8,192: 23.399/22.284 µs; rollback exact | pass |
| staging allocation | both sizes: 675 alloc, 0 realloc, 84,409 peak, 0 retained in every sample | pass |

The standard build is the only latency authority. Allocator-observer timing is non-applicable and cannot create a false
latency failure or PASS. Conversely, the standard receipt has no direct-allocation claim.

The retained entry evidence proves correction necessity rather than hiding it: initial event/concurrent p95
25.735434/13.486429 ms failed; initial checkpoint 123.394 µs failed; pre-COW high-history staging 176.113 µs failed.
Exact-output A/B receipts isolated the blocker and redundant movement-order hotspots before production correction.

## Final Integration Audit

| Acceptance | Direct evidence | Result |
| --- | --- | --- |
| policy acceptance | PE-001 through PE-010 evidence index | 10/10 pass |
| validation dimensions | success through legacy/fallback absence | 10/10 pass |
| feature inventory | Rulebook/manifest mapping | 57/57 pass |
| focused release evidence | event/result/Rapier/checkpoint/application selectors | 56/56 pass |
| full Core | library 1,108 + integrations 51 | 1,159/1,159 pass |
| build/docs | fmt, diff, release check, release rustdoc | pass, warnings 0 |
| public API probes | five valid + five expected-invalid + fixture expected-invalid | 11/11 expected |
| structural closure | writer/privacy/history/V4/fallback/test/benchmark categories | unresolved 0 |
| predecessor currentness | direct P01–P07 successor audits | 7/7 Complete / High |
| external claim absence | external evidence/consumer/rollout/SLA rows | 0 claimed |

The manifest verifier reports 57 features, 57 headings, 14 bundles, 10 dimensions, 10 legacy targets, 44 current
locator contents, composite `5ff7356c…620` and zero errors. The old unbounded event-log Vec locator is an asserted
absence with the bounded COW replacement.

## Fresh GEX-20 Validation

- Fresh five-tool performance/public run to temporary receipts: event-heavy 9.883428 ms, concurrent 8.301524 ms;
  exact hashes; both allocation guards pass; snapshot 281.041 µs; checkpoint 7.967 µs; query 0.184/0.211 µs;
  staging 22.535/21.151 µs; low/high allocation observations remain exactly equal.
- Fresh formatting and diff checks: exit 0.
- Fresh release library check and rustdoc: exit 0, warnings 0; fixture/raw public names absent.
- Fresh manifest verifier: 57/57, ten dimensions, 44 contents, zero errors.
- Fresh five valid probes: all exit 0. Fresh five established invalid probes and the fixture invalid probe: all expected
  exit 101.
- Fresh `cargo test -p game_core --all-targets`: 1,159 passed; failed/ignored 0.
- Post-validation target comparison: 95/95 exact. Post-validation unrelated dirty baseline: exact.

## Structural Classification and Limitations

- `LiveBattleEventDeltaDto.events: Vec<_>` is a bounded response value after cursor validation, not authoritative
  retained history. Internal `ActiveRunTransactionRollback` is scoped rollback state, not a second application writer.
- The one explicit `ActiveRunCorePayloadV4` path is a deserializer/importer required by P06's certified removal gate.
  V4 serializer/writer count is zero; it is not a broad fallback.
- `test-fixtures` compiles the cap-scale installer only for the standalone verifier. Default release returns E0599 and
  rustdoc count zero. Runtime test/benchmark-dependent production branches are zero.
- System `perf` sampling was unavailable; exact-output source-instrumented A/B receipts are retained. This does not
  weaken the final repeated standard/allocator measurements.
- Whole-library Clippy retains the previously recorded unrelated denied `never_loop`; required release check and
  rustdoc are warning-free.

## Verdict

`GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION` is `Complete / High` on
`ATR-CRAR-C2-P08-GEX10-001`, currentness `current / unchanged`.

Both internal nodes pass in strict order, `GAP-CRAR-P2G20-002` is closed, and no completion stop condition remains.
This verdict completes the eighth and final Core-only phase member; it does not claim external consumer compatibility,
deployment readiness or a product SLA.

## Successor GEX-20 Audit 001 — `Complete / High`

### Trigger, Target and Separation

- Master GEX-30 Attempt 2 found the runtime `cfg!(test …)` projection switch omitted by the original P08 source audit.
  A separated P07 correction GEX-10 removed that runtime switch without changing configuration-visible values, and
  fresh P07 GEX-20 completed on replacement target `_002`.
- P08 re-audit target: `ATR-CRAR-C2-P08-GEX20-001` / `evidence/P08_AUDIT_TARGET_002.json`, SHA-256
  `1a0b7c628b581ed5c476b3e6aa26399394575013d5aa30140c6984d9b4d42101`.
- It composes the unchanged original 95-member P08 target with 16 current P07 delta/upstream/currentness members.
  Initial and audit-exit comparisons are 95/95 and 16/16 exact; predecessor Phase currentness is 7/7
  `Complete / High`.
- This was a fresh read-only P08 GEX-20. No implementation/source/data/test/tool/manifest/target member changed.

### Fresh Performance and Public Evidence

| Guard | Fresh current result | Result |
| --- | --- | --- |
| event-heavy | 21 samples; 9.495603 ms p95 <25 ms; p=1.0; exact output | pass |
| 64-session concurrent | 21 samples; 7.183969 ms p95 <10 ms; p=1.0; exact output | pass |
| direct allocations | event 14,295/25,896/260,550/260,550; concurrent 170,740/140,036/8,008,277/4,196,928 maxima | pass |
| P01 exact replay | five repeatable samples; both immutable output identities exact | pass |
| public snapshot | 223.580 µs p95; 48,493 bytes; exact | pass |
| V5 checkpoint | 9.032 µs p95; 17,956 bytes; exact | pass |
| bounded query | 0.184/0.193 µs at 1,024/8,192; exact/caps | pass |
| transaction staging | 21.106/20.282 µs; rollback/scaling exact | pass |
| staging allocation | low/high both 675 alloc, 0 realloc, 84,409 peak, 0 retained | pass |

All five verifier commands exited 0 against unchanged P01/tool/input identities. The P07 correction does not enter
the deterministic battle workload and preserves the `test-fixtures` public-guard configuration value.

### Fresh Final Integration Evidence

- Formatting/diff, release check and release rustdoc pass; warnings zero. Default rustdoc exposes neither the fixture
  method nor raw `GameCore`/behavior/result/rollback names.
- Manifest verifier passes 57/57 features, 57 headings, 14 bundles, ten dimensions, ten legacy targets, 44 contents,
  current composite `07f3b56b…fb8a7`, errors zero.
- Five valid API callers exit 0; five established invalid callers and the default fixture caller exit expected 101.
- Fresh full Core passes library 1,108 + integrations 51 = 1,159/1,159; failed/ignored zero.
- Direct structural counts: one application writer; runtime test/benchmark switches 0; unbounded event-log Vec fields
  0; V4 serializers 0; required typed V4 importer 1; default fixture/raw public index names 0.
- The original ten policy acceptances and ten validation dimensions remain 10/10 with current direct evidence. The
  changed source specifically repairs the `producer_consumer`/`legacy_fallback_absence` guardrail evidence and adds no
  public meaning, authority, compatibility or external dependency.

### Current Verdict

`GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION` is `Complete / High` on
`ATR-CRAR-C2-P08-GEX20-001`, currentness `current / P07 correction incorporated and final integration revalidated`.

All eight Phase Goals are again current. A new Master Audit Target and fresh GEX-30 are required; the earlier Master
Attempt 2 remains `Not Complete` historical evidence.
