# Phase 42 CRA-P06-C07 Battle-Log I/O Debug Boundary Cleanup Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — the user's 2026-08-11 instruction approves every remaining registered
cleanup after candidate-specific inventory and retained behavior guards
Parent Master: `core_enhanced_refactoring_rebaselined_backlog_implementation_master`
Phase: 42 of 44
Candidate / Work Type: CRA-P06-C07 / `cleanup`
Direct Linear Dependency: Phase 41 CRA-P06-C06 `Complete / High`
Direct Technical Dependency: none; current battle-log producer/trace/compression graph must be rediscovered
Direct Policy: CRA-POL-006
Repository / Start Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Initialization Date: 2026-08-11 (`Asia/Seoul`)

## Objective

Keep battle-log serialization on the data type only where currently consumed, and keep file path/lifetime/error
policy at explicit debug/trace boundaries. After a complete current consumer and behavior inventory, remove or
depublish only unused general public file read/plain-write and redundant pretty-string surfaces, while retaining or
consolidating the real narrowly named debug writer required by current trace/test output.

## User/Contract-Visible Problem

Historical evidence found public `BattleEventLog::{write_json, read_json}` helpers with no workspace callers even
though the type has no runtime file lifecycle. Standard JSON serialization and server compression use an in-memory
JSON string, while the authoritative world transition owns explicit best-effort debug export. Leaving file ownership
on the general data type advertises a durable battle-history API that the runtime does not implement.

That report is locating evidence, not deletion authority. This Phase must reconstruct the current methods,
visibility, errors, serializers, all debug/test/server/tool callers, file paths and external consumers after sole
activation. Any current readback, durable history, deployed standalone consumer or distinct pretty-output contract
stops that member's removal and is not silently reinterpreted.

## Audit Finding ID

CRA-P06-C07. The final catalog classifies it `still_valid`, `cleanup`, Tier D, priority axes
`1/0/0/1/1/0/unknown = 3`. Exact linear item 42 requires CRA-POL-006 public/external inventory while standard serde,
compression and retained debug-writer behavior remain exact.

## Historical Evidence And Current-Proof Gate

- Original P06 audit: public `write_json`/`read_json` had no workspace callers; tests/debug used pretty output;
  server compression used `to_json_string`; world trace owned explicit best-effort export.
- Post-Boss reaudit: the public file helpers remained unused and no Boss-specific file lifecycle appeared.
- Final order/test records: inventory public/standalone consumers; keep standard serde round trip, compression and
  debug trace failure behavior; Phase 44 waits on the stabilized retained tool/API surface.
- Phase 41 audit is 224 lines at SHA-256 `b800f55a...`, verdict `Complete / High`.

Historical facts do not substitute for current proof. Sole activation and prospective fingerprints precede current
production discovery, baselines or mutation.

## Source Of Truth And Ownership

### Prospective current authority

- `BattleEventLog` prospectively owns in-memory entries and serde-compatible JSON content.
- World combat/result tracing prospectively owns whether, where and how a best-effort debug file is written.
- Server transport prospectively owns any compression/envelope lifecycle over the retained JSON content.
- Exact current methods, helpers, consumers and file/error behavior remain `discovery pending`.

### Target authority

The event-log data type exposes only currently consumed in-memory/serialization operations. Explicitly named debug
or trace helpers own file path, creation, pretty formatting and failure policy. No durable player battle-history
reader, implicit write side effect, compatibility wrapper or second serializer is introduced.

## Invariants

- Event-log serde field/tag/nesting/version, entry order, causes and source-command correlation remain exact.
- Standard serde round trip and retained in-memory JSON output remain exact.
- Server compression/decompression and published payload content remain exact.
- Current debug/trace output remains human-readable at the same explicit boundary and path policy.
- Debug export failure remains best-effort and cannot block authoritative combat result/world transition.
- No durable per-player battle history, restore/replay authority or production readback is introduced.
- No fallback, alias, dual serializer, ignored test or compatibility shim is added.
- No Phase-43 targeting/range or Phase-44 package/dependency cleanup is pulled forward.
- Existing dirty work and the protected Phase-27 marker remain exact.

## Refactor Candidate And Behavior-Preservation Boundary

- Candidate: CRA-P06-C07; `cleanup`; `still_valid`; Tier D; known score `3/18`.
- Prospective removal: unused public file reader/plain writer and redundant pretty-string API proven to have no
  current consumer or distinct contract.
- Prospective retention/consolidation: `to_json_string`, serde shape, server compression and the one real explicit
  debug/trace writer path after current inventory.
- First safety net: exact symbol/caller/error/path matrix plus nonzero standard serde round trip, compression and
  debug-writer success/failure behavior before removal.
- Boundary: any real runtime readback, durable storage, standalone tool or external public consumer stops that member
  and is not replaced by a permissive compatibility path.

## Approved Policy And Evidence

- CRA-POL-006 normally requires consumer inventory plus deprecation/versioned migration or explicit breaking
  approval before narrowing a public Rust API.
- The user's 2026-08-11 instruction authorizes every remaining verified public/package cleanup without a shim after
  its own inventory and behavior guards.
- That approval does not authorize a serde/wire change, durable battle history, error-policy change or deletion of a
  real debug/server consumer.

## Open Policy Decisions

None at initialization. If discovery finds a real durable/deployed read/write contract or requires changing JSON,
compression, trace failure or retention semantics, stop the affected work as `policy_decision_required` rather than
infer a migration.

## In Scope

- exact current event-log types, serde derives/version fields, public serialization and file-I/O methods;
- every workspace/server/test/tool/debug caller, helper wrapper, file path and error policy;
- accessible external Rust/C#/script/config consumer and package publishability inventory;
- standard serde/string/pretty behavior and server compression/decompression;
- world authoritative trace/export success and failure behavior;
- assertion transfer or helper consolidation needed to retain real debug behavior before public cleanup;
- focused/broad debug/release/static validation and separate completion audit.

## Out Of Scope

- durable player battle history, replay/playback, restore/import or retention policy;
- event-log field/tag/version/order/cause/source-command changes;
- combat result transaction, reward, checkpoint, wire envelope or server message redesign;
- changing trace enablement, destination policy or best-effort failure semantics;
- Phase 43 range/delivery surfaces and Phase 44 package/target/dependency cleanup;
- unrelated formatting, lint or dirty-tree cleanup.

## Implementation Scope

1. initialize and solely activate this Child before current production inspection;
2. capture prospective filename/status/hash boundaries for event log, world trace, server compression, tests,
   manifests and consumers;
3. enumerate every serialization/file helper, visibility, caller, path, error and external consumer;
4. establish standard serde round-trip, compression and explicit trace success/failure baselines;
5. transfer or strengthen retained debug-boundary assertions if needed before removal;
6. remove/depublish only verified unused general file/redundant pretty surfaces and consolidate the real debug helper
   only when exact output/error parity is proven;
7. repeat exact legacy/caller searches, focused/broad profiles, workspace/static and dirty/marker verification;
8. perform a separate completion audit; make Phase 43 ready only on `Complete / High`.

## DTO, Data, Save And Migration

- DTO/wire migration: none; event-log JSON content and any server envelope/compression remain exact.
- Live RON/data migration: none; battle logs are runtime output, not production authored input.
- Save/checkpoint migration: none; no durable battle-log history is approved.
- Public Rust migration: immediate removal is approved only for current unused helpers after consumer inventory.
- Filesystem migration: none; retained explicit debug writer keeps current path/output/failure contract unless current
  evidence shows a narrower exact consolidation.
- Unity/server migration: none planned; server compression behavior is a required preserved consumer.

## Deployment And Compatibility

Core, game_server, in-workspace tools/tests and accessible repository consumers form one validation unit. The Core
crate's publishability and unknown standalone consumers remain explicit until inventoried. The user's approval
allows only verified removal; no alias, deprecated forwarding method, dual output or fallback is authorized.

## Focused Validation

- exact method/visibility/caller/path/error/consumer disposition matrix;
- standard serde round trip and retained compact/pretty JSON parity;
- server compression/decompression content parity through the actual producer path;
- explicit debug/trace writer success and failure behavior, including failure non-interference;
- exact removed-symbol/placeholder/public search and current test-list delta.

Every accepted command records selected/pass/fail/ignored counts. Zero-selected filters and compile-only commands do
not prove behavior.

## Broad Validation

- complete event-log, battle result/world combat and server compression/transport families in debug; representative
  release;
- Core and server affected package suites plus Core all-target debug/release;
- workspace all-target checks in debug/release;
- normal affected-package Clippy, `cargo fmt --all -- --check` and whole-worktree `git diff --check`;
- final test-list/member reconstruction, non-target hashes and protected marker verification.

## Omitted Validation

- Unity runtime is omitted only if JSON/envelope/compression shape remains byte-equivalent and no Unity consumer
  reads these internal helpers; otherwise it becomes mandatory.
- save migration probes are omitted only if current inventory confirms no persisted battle-log reader/writer.
- live RON tests are omitted only if no authored-data coupling exists; package broad validation may still include
  them as a regression guard.
- repository-wide Stage-D integration remains a Master task after Phase 44.

## Completion Conditions

- Every current serialization/file/debug helper, caller, path, error and consumer is named and classified.
- Standard serde/string content, server compression and explicit debug trace behavior are protected before cleanup.
- Only unused/redundant public file or pretty surfaces are removed/depublished; real debug ownership is explicit.
- No event-log JSON/version/order, wire, save or runtime-result behavior changes.
- No durable history, compatibility shim, fallback or Phase-43+ change is introduced.
- Focused/broad debug/release/static checks are green with exact counts.
- Non-target hashes, unrelated dirty work and protected marker are exact.
- A separate completion audit returns `Complete / High`.

## Completion Audit Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P42-C01 | current public/helper graph is exact | definitions, visibility, errors, callers and consumers | pending |
| P42-C02 | serialization/file authorities are exact | data serde → server compression; world → explicit debug trace | pending |
| P42-C03 | dirty work is preserved | activation/current hashes, scoped diffs and marker | pending |
| P42-C04 | every candidate member is classified | read/write/pretty/string/debug helper disposition matrix | pending |
| P42-C05 | current behavior is characterized | serde, compression and trace success/failure baselines | pending |
| P42-C06 | useful assertions precede removal | retained production/debug controls green with old API | pending |
| P42-C07 | public compatibility gate is exact | consumer/package/deployment inventory and approval mapping | pending |
| P42-C08 | only false I/O surface is removed | exact symbol/API delta; retained writer; no shim | pending |
| P42-C09 | event-log behavior is exact | content/order/version/compression/trace parity | pending |
| P42-C10 | DTO/data/save/server/Unity boundary is exact | hashes/searches/affected integration checks | pending |
| P42-C11 | broad/static validation is green | both profiles, workspace, format/diff and Clippy | pending |
| P42-C12 | independent audit/next boundary is current | `Complete / High`; Phase 43 absent until exit | pending |

## Stop Conditions

Stop and do not activate Phase 43 if a helper has a real durable/runtime/external consumer, a JSON/version/order or
server compression shape changes, debug failure can block result completion, a migration beyond approval is needed,
dirty overlap cannot be isolated, Phase-43+ work is required, a mandatory test selects zero, the protected marker
changes, or the independent audit is below `Complete / High`.

## Initialization Dirty Baseline

- Expanded status before this Child: 590 rows at canonical C-sort SHA-256
  `21f63c04e1ef6b957449f412d6aa05b5619118602b500cb99372ea0b9ed49b45`.
- Phase 42 path was absent and active implementation Child count was zero.
- Phase 41 completion audit is 224 lines at SHA-256
  `b800f55ac439eb33c8c8dc3b005f5e10324d7b0d41be613ac5c5f4f275d25497`, verdict `Complete / High`.
- Protected marker is 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`.
- These three new Child documents are the only expected status-set additions before activation evidence.
