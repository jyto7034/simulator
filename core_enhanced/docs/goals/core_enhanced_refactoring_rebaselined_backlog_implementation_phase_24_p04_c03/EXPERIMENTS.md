# Phase 24 CRA-P04-C03 Experiments

Status: complete
Implementation Complete: true
Candidate: CRA-P04-C03
Master Phase: 24 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Experiment Rules

- No production source, test implementation, live data or canonical content is inspected before sole activation and
  the complete activation protocol.
- Every command records its selected test count; zero selected cases are not evidence.
- Add paired-state/no-run characterization before production refactoring.
- Compare complete Event/Shop session values and relevant outer state, not only enum tags or selected fields.
- Preserve public/admin fixture behavior and exact checkpoint/result/error order.
- Use only the smallest private Event/Shop commit boundary justified by the caller graph.
- Preserve unrelated dirty work; no reset, restore, clean, stash or broad formatting.
- Phase 25/26 work is forbidden.

## Historical Hypothesis To Verify

Event's current commit helper requires a run and writes both the run-persisted session and active Event mirror.
Shop's helper writes the persisted cache only when a run exists but always updates the active Shop mirror. Initial
Event/Shop map entry and HQ-to-Shop historically repeat equivalent direct writes.

The refactor hypothesis is that explicit domain commit paths can own those paired writes without changing session
values, lifecycle or consumers. Current source is authoritative and may disprove or refine the hypothesis after
activation.

## Required Experiment Sequence

1. Activation/status/prospective-fingerprint capture.
2. Complete Event/Shop/HQ paired-session caller and consumer inventory.
3. Baseline nonzero focused test discovery and runs.
4. Test-first complete equality and explicit no-run matrix on the old implementation.
5. Smallest private boundary and caller-by-caller migration.
6. Static removal/fallback/public/persistence/canonical comparison.
7. Focused then broad Core/server debug/release validation.
8. Independent 567-line-guide phase-exit audit.

## 2026-08-10 — E01 Initialization

- Phase 23's final 264-line audit was fully read and Master records it `complete — Complete / High`.
- Original, re-audited and final P04-C03 candidate/order/test/dependency evidence was read before this Child was
  written. The candidate remains policy-neutral and requires paired-state characterization first.
- Pre-initialization expanded status is 493 entries at normalized SHA-256
  `d63d5b1640d6f85fa645b3b3ded7588388d061a573852927bf86341764911137`.
- This initialization creates only the three required Phase 24 Goal documents. Production inspection/mutation is
  still forbidden until Master sole activation and the full Child reread.

## 2026-08-10 — E02 Activation Baseline And Prospective Fingerprints

### Activation Gate

- Master registers Phase 24 as the sole active Child and Phases 25–44 remain pending.
- The active 191-line PLAN, 51-line experiments, 60-line notes and final 264-line Phase 23 audit were read completely
  after activation.
- HEAD is `f318414fb0156a67e04c5479000c2a2683fc0c09`. Expanded activation status is 496 entries at normalized SHA-256
  `e843ff04328b134ad762d18ee6dd0563b146cb7ebdad837b71eaa5d186c04e85`; the three-row increase from readiness is
  exactly the three Phase 24 Goal artifacts.

### Prospective Source/Test Fingerprints

- Production: Event `b107be74...`, Shop `dfeb6030...`, Headquarters `bf19e22e...`, node flow `a311f02b...`,
  snapshot `6e512c4d...`, state `1e5f9177...`, world module `c533e0ff...`.
- Tests: node sessions `c30a3965...`, map flow `c823a5c9...`, facility `3e13a4c0...`, snapshots/start
  `2144e86e...`.
- Canonicals: runtime `6e20d6a6...`, rulebook `6c53ecd5...`, server bridge `4d734af0...`, external Unity
  `a3224e24...`.
- Every listed workspace target except clean `headquarters.rs` is already modified in the shared dirty tree. These
  hashes are prospective ownership boundaries, not attribution to Phase 24.

### Next Experiment

Production-detail inspection is now authorized. Inventory exact persisted/active types, every caller and consumer,
then run nonzero baseline filters before adding characterization.

## 2026-08-10 — E03 Current Owner Graph And Test-First Characterization

### Current Graph

- `RunState.event_sessions` and `RunState.shop_sessions` are serialized durable caches keyed by node ID. Active
  `EventSessionState`/`ShopSessionState` values are separate `ActiveNodeContent` variants used by admission,
  capabilities and snapshot projection.
- Event paired writes occur in the existing private helper and one normal `enter_event_node` branch. The re-entry
  branch for a previously started Event combat writes only the persisted session before handing active authority to
  Combat; this is intentional and is not paired-write duplication.
- Shop paired writes occur in the existing helper, map Shop entry and HQ-to-Shop. Purchase/refresh use the helper;
  map resolution reads persisted Shop history. Sale changes no Shop session.
- Admin fixtures deliberately construct/remove active or cached sessions directly and remain compatibility setup,
  not production paired-commit ownership. Completion removal, checkpoint mapping and snapshot readers remain outside.

### Baseline And Test-First Results

- Old-tree baseline: node sessions 19, map flow 41, Shop unit 7, Event unit 5, snapshots/start 23 and facility 11;
  total 106, all passed with zero failed/ignored.
- Added complete-value comparison helpers: Event uses its exact equality; Shop compares the full serialized value
  because its public type intentionally lacks `PartialEq`.
- Characterization now covers map Shop entry, HQ handoff, purchase, refresh, Event entry, Event check/grant,
  checkpoint/re-entry, authored scene updates and duplicate reuse. No-run map Shop/Event entry reject before active
  mutation, while no-run Shop purchase/refresh retain active-only behavior.
- The first no-run run expected `InvalidAction`; current `run_state[_mut]` authority actually returns exact
  `MissingResource("RunState")`. Tests were corrected to pin that existing variant rather than accept any error.
- Corrected test-first counts pass: node sessions 21, map flow 42 and Shop unit 7; total 70, zero failed/ignored.

### Implementation Decision

- Rename the Event paired helper to make required-run semantics explicit and route normal Event entry through it.
- Give the single Shop helper an explicit typed mode: required run for map/HQ entry, preserved active-only allowance
  for in-session purchase/refresh fixture behavior.
- Leave started-Combat Event persistence, session construction, HQ transaction, completion/removal, checkpoint and
  public/admin surfaces at their current owners.

## 2026-08-10 — E04 Implementation And Focused Parity

### Implemented Boundary

- Normal Event entry and every existing Event scene/check mutation now use
  `commit_event_session_required_run`. The helper preserves the exact run-cache insert, active assignment and
  `MissingResource("RunState")` order.
- One `pub(super)` Shop helper accepts the typed `ShopSessionCommitMode`. Map and HQ entry select `RequireRun`;
  purchase/refresh select `AllowActiveOnlyWithoutRun`, preserving current fixture behavior.
- Map Shop and HQ no longer contain direct cache-insert plus active-assignment pairs. HQ recruitment assignment,
  transition and `with_headquarters_transaction` remain in place.
- Started-Combat Event persistence remains a deliberate persisted-only handoff to Combat. Admin fixtures,
  completion removal, checkpoint mapping, snapshot/action readers and Shop sale remain unchanged.

### Focused And Static Results

- `cargo check --all-targets` passes.
- Focused implementation total is 109/109: node sessions 21, map flow 42, Shop unit 7, Event unit 5,
  snapshots/start 23 and facility 11; zero failed/ignored.
- Direct production paired writes now live only in the Event helper and Shop helper. Searches retain only the
  classified Event combat handoff, state/checkpoint tests and admin/test fixture operations.
- Formatter and repository-wide diff check pass. No Phase 25 HQ rollback/context replacement or Phase 26 VN
  assembly/placement work was introduced.

### Next Experiment

Run complete Core and server debug/release suites/checks, compatibility hashes/searches and diagnostic Clippy, then
hand P24-C01–P24-C09 to the independent phase-exit audit.

## 2026-08-10 — E05 Broad, Release And Compatibility Validation

### Broad Results

- Complete Core debug and release each pass 1006 unit tests plus 51 official integration tests, with zero failures
  or ignored tests.
- Complete server debug and release each pass 59 tests, with zero failures or ignored tests.
- Core and server `cargo check --all-targets` pass in debug and release profiles. `cargo fmt --all -- --check` and
  repository-wide `git diff --check` pass.
- Non-strict all-target Clippy completes with existing debt: Core library 82/library-test 135, server library
  3/library-test 7. No diagnostic points at the Event/Shop commit helpers, modes or caller-transfer lines. Strict
  global `-D warnings` is not claimed.

### Compatibility, Static And Dirty-Scope Results

- Session types, `RunState` serialized fields, checkpoint mapping, snapshot/action readers, public behavior/results,
  live data, server and Unity source were not changed. Full Core/server tests exercise the current consumers.
- Canonical hashes remain activation-identical: runtime `6e20d6a6...`, rulebook `6c53ecd5...`, server bridge
  `4d734af0...`, external Unity `a3224e24...`. The refactor is private representation and warrants no canonical edit.
- Final production hashes are Event `947ceb5f...`, Shop `aa53d11a...`, map content `6d21eccc...`, Headquarters
  `1bc7d090...`; unchanged supporting node flow/snapshot/state hashes remain `a311f02b...`, `6e512c4d...`,
  `1e5f9177...`.
- Final test hashes are helper `f6c405d5...`, node sessions `dddcb61b...`, map flow `bfbf36a2...`; untouched
  facility/snapshot tests remain `3e13a4c0...` and `2144e86e...`.
- Expanded pre-audit status is 498 entries at normalized SHA-256
  `dceeff2a4457953500422ea840dec67ef81d2e454238fabe23f05c2b253baa17`. The 496→498 increase is the two
  production paths newly brought into dirty scope for this Phase; unrelated work was preserved.

### Review Handoff

P24-C01–P24-C09 claim complete. P24-C10 remains `review_pending` until the complete 567-line review guide, current
Child, source/tests/consumers/canonicals and cross-Phase boundary are independently reread and the phase-exit audit
returns `Complete / High`.

## 2026-08-10 — E06 Independent Phase-Exit Audit And Completion

- The current 874-line authoring/execution guide and complete 567-line completion-review guide were read. The current
  190-line PLAN, 176-line experiments and 107-line notes were also reread before verdict.
- The audit separately reread runtime/session/checkpoint/projection/fixture, server/external consumer, canonical,
  dirty-scope and Phase 25/26 boundary evidence.
- Focused audit rerun: node sessions 21, map flow 42, Shop 7, Event 5, snapshots/start 23 and facility 11; 109/109
  passed, zero failed/ignored.
- Broad audit rerun: `cargo test --tests` and `cargo test --release --tests` each pass Core 1006 unit plus 51
  integrations; server equivalents each pass 59. Four all-target checks, formatter and diff gate pass.
- `COMPLETION_AUDIT.md` records all ten evidence rows as pass, primary verdict `Complete`, fit `High`, improvement
  class `none`, and no immediate correction.
- Audit-inclusive expanded status is 499 rows at normalized SHA-256
  `1f576cd8f6dcb7108d3ba54a3b4e14cf3118f852249107bf8309788c86a4a331`.
- P24-C10 is complete. Child Status transitions to `complete`, Implementation Complete to `true`; Phase 25 may become
  ready only through the Master registry.
