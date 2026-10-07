# Phase 39 CRA-P07-C03 Completion Audit

Audit Mode: `phase_exit`
Audit Target: `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_39_p07_c03`
Audit Date: 2026-08-11 (`Asia/Seoul`)
Current Status: `active`
Implementation Complete Claim: `false`
Candidate / Work Type: CRA-P07-C03 / cleanup
Direct Policy: CRA-POL-006
Approval Source: user's 2026-08-11 approval to resume Phase 39 and continue all remaining registered cleanup phases
Finding Sources: original/post-Boss P07-C03, final test catalog and rebaselined exact item 39
Goal Documents: current Child `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`; parent Master `Plan.md`
Repository / Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Audit Scope And Independence

This is the separate Phase-exit review required by the Child and Master contracts. It treats implementation notes as
pointers, independently checks the current movement authority and visibility graph, reconciles every historical
Direct-dependent assertion with a retained production-path control, verifies the exact carrier/test delta, and
reviews Boss/world/range/event, public compatibility, data/transport, dirty-worktree and validation boundaries.

After the implementation record was written, the audit reran the complete Core debug all-target suite, repeated
current legacy/public searches, verified that Phase 39 is the sole active Child and Phase 40 is absent, and refreshed
the expanded status, marker and diff checks. Release all-target, workspace debug/release checks and Clippy remain
current because no production, test, manifest, data or dependency source changed after those successful commands;
only Goal evidence documents changed.

## Approved Condition Restatement

CRA-P07-C03 must leave one private Rapier-backed continuous-movement authority. It may remove the historical Direct
implementation, backend enum/selectors, false parity suite and test-only Rapier board-wall/introspection surface only
after every useful collision, clamp, obstacle, overlap, target, determinism and authored-route assertion passes on
retained Rapier or unselected default `BattleCore` behavior.

The local consumer search could not by itself prove absence of deployed standalone Rust users because the Core
package is not marked non-publishable. The user's candidate-scoped approval authorizes this exact public break with
no shim or deprecation lifetime after inventory and transfer. It does not authorize movement gameplay changes,
Phase-40 ghost-state cleanup, or changes to live RON, DTO/wire, save/checkpoint, server/Unity or canonicals.

## Completion Evidence Matrix

| ID | Completion condition | Current evidence | Validation evidence | Verdict |
| --- | --- | --- | --- | --- |
| P39-C01 | backend/public graph is exact | one stored Core-private Rapier world; private module and engine trait; no enum/Direct/selectors/walls | zero legacy source/test search; explicit visibility inventory | pass |
| P39-C02 | production/Boss/tactical consumers are exact | Core calls the ordinary movement leaf; reset clears the same world; Boss remains a caller, not authority | movement, Boss, tactical, authored-route families | pass |
| P39-C03 | dirty work is preserved | activation hashes bound pre-dirty carriers; only formerly-clean movement `mod.rs` adds a source status row | 576 activation rows → 577 pre-audit rows; exact marker | pass |
| P39-C04 | all 17 assertions are classified | Child Notes contains a named 17/17 disposition matrix | retained tests passed before deletion | pass |
| P39-C05 | old and retained behavior are characterized | old engine/Rapier/default/Boss/route baselines were all nonzero | 19/19, 15/15, 1/1 and 25/25 accepted baselines | pass |
| P39-C06 | assertion transfer precedes deletion | Rapier 19/19, engine 19/19, route 1/1 and Boss 25/25 passed with old carriers still present | E04 source hashes prove pre-cleanup state | pass |
| P39-C07 | public removal gate is exact | local/package/deployment inventory recorded; user approved exact break | workspace Core/server compilation | pass |
| P39-C08 | only false authority is removed | enum, Direct, selectors, walls and inspection surface absent; no replacement selector/fallback | `-16 +1` exact test accounting and zero legacy search | pass |
| P39-C09 | movement/Boss/world behavior remains exact | neutral clamp retains arithmetic; collision/terrain/log/route policies unchanged | movement 61, Boss 25, tactical 34, range 52, event 76, world 293 | pass |
| P39-C10 | DTO/RON/save/server/Unity boundary is exact | no serialized/data/transport carrier changed; activation hashes retained | RON integrations in both all-target profiles; workspace checks | pass |
| P39-C11 | focused/broad/static validation is green | tested production hashes equal current source | debug/release 1,056 each; checks, format, diff and Clippy exit 0 | pass |
| P39-C12 | independent audit and next boundary are current | Phase 39 sole active; Phase 40 pending and absent during review | this audit returns `Complete / High` | pass |

## Current Runtime And Visibility Evidence

- `BattleCore` stores `RapierMovementWorld` directly and constructs it with `default()`. There is no runtime or test
  backend selector, enum variant, fallback or feature-gated alternate authority.
- Movement reset invokes the same world's restricted `clear()` method. No parallel physics state survives reset.
- `movement::rapier_backend` is private; movement exposes its world only with `pub(super)`, while the definition and
  `clear` are restricted with `pub(in crate::game::battle::core)`. The engine trait is `pub(super)`.
- Public movement DTO/value types used by actual consumers remain public. This cleanup did not indiscriminately
  privatize `MovementTickInput`, `MovementOutput`, terrain policy or `BattleCore` movement methods; Phase 40 owns
  any verified ghost fields/variants inside those retained contracts.
- The test-only Rapier board-wall state and wall insert/sync/remove path are absent. Actual production behavior still
  uses the unchanged neutral manual board clamp and retained static-obstacle correction.

Current source fingerprints are:

- `movement/engine.rs`: 819 lines, `222836cdc0200962c7f9a4cdf7cbe0ba0cd3182b6c6d2b45d46ef19adddab344`;
- `movement/rapier_backend.rs`: 921 lines, `b70188da0adc8fbd205ffa877d33aee95a29810b4c000c46af14719d570f342f`;
- `movement/mod.rs`: 27 lines, `aaac13d908f686946df64367119b554b01cf4c41024b206555e6ab17565a9565`;
- `core/mod.rs`: 8,523 lines, `9c9f01d0880b9376328f30adff691a08d24e6cdecda9697f178575e03299aa09`;
- `core/sim.rs`: 3,359 lines, `af5130fc76e5d80226767f8e43c3f6d6783e5eccacfd104df6abba768c19fae8`;
- `events/combat.rs`: 1,970 lines, `5c232c3841f6003d10ebe935022810b050e3e6b4ca3150678a829a9a34662597`.

## Assertion Transfer And Test-List Accounting

The Child's 17-row matrix enumerates four backend-parity carriers, nine Direct-only carriers, three Core tests that
selected Direct and one authored-route test that selected Direct. Before cleanup, retained Rapier/default controls
passed while all old carriers remained. The two segment tests and authored route survive with their selector removed;
the redundant Direct Core twin and 13 Direct/parity engine tests were then removed.

Two additional test-only board-wall tests were removed because production ignored their collider world. The complete
test-list delta is therefore:

- removed four `movement_backends_*` parity tests;
- removed nine `direct_engine_*` tests;
- removed `battle_core_can_run_one_direct_continuous_tick_with_explicit_goal`;
- removed `sync_board_bounds_creates_four_static_wall_colliders` and
  `rapier_ground_correction_ignores_board_wall_colliders`;
- added `battle_core_default_movement_is_repeatable_across_unit_insertion_order`.

The pre-cleanup 1,071 all-target tests minus 16 obsolete carriers plus one retained default-path control equals the
current 1,056. The current `: test` list hash is
`35a110371b12841df4d86281f99c6d55dae6195860a5501e8334eae5fd7c3ecf`.

The new repeatability test constructs equivalent default cores with opposite unit insertion order, runs two equal
ticks, and compares complete tick results, both final positions and serialized event logs. This closes the explicit
repeated-default-run invariant without introducing a test-only selection seam.

## Behavior-Preservation Evidence

- Retained Rapier tests cover canonical input/output order, exact movement velocity, exact ground/air board clamps,
  static-obstacle blocking without tunneling or unapproved slide, airborne obstacle bypass, intentional moving and
  boundary overlap, locked-unit collider bypass and `TargetReached` identity.
- Default Core tests cover explicit-goal movement, immutable adjacent movement segments, no event without body
  motion, reverse-insertion repeatability and battlefield/void static-obstacle projection.
- The movement family includes planner controls that distinguish tile-range eligibility from ghost `range_units`.
  Phase 39 did not change either. The user's accepted `Nearest` behavior and all tile-range functions remain intact.
- Boss runtime 25/25 includes ordinary-leaf Move/Rejoin and movement-lock timing. The exact authored defense route
  still progresses past its spawn boundary and emits movement through the production default.
- Tactical, range, event and world families demonstrate that final positions, movement segments and current public
  projections remain usable by downstream consumers.

## Consumer And Compatibility Evidence

- Before cleanup, complete repository and accessible `/mnt/f/work` Rust searches found no consumer of the backend
  selection/inspection surface outside its implementation and tests. `game_server` is the only workspace path
  dependency and did not consume it.
- The Core crate is not marked `publish = false`, so the audit does not misstate negative local evidence as proof of
  external absence. The user's exact breaking approval is the compatibility authority.
- Current Core/server source and tests have zero matches for `ContinuousMovementBackend`,
  `DirectContinuousMovement`, either backend selector, `RapierBoardBounds`, or its wall methods.
- No alias, shim, deprecated copy, feature flag, ignored test, dynamic selector or placeholder replaces the removed
  surface. Historical Goal references remain evidence, not live API.

## Data, DTO, Transport And Persistence Evidence

- Root workspace manifest, lockfile, Core/server manifests and Core lib root retain activation SHA-256 values
  `98c9cf4d...`, `4223318b...`, `40d8a9ec...`, `19b41cd...` and `3fbfdca2...`.
- Live Boss runtime RON remains `f696c9d054ed1e4397809f80c2707e06614f4324a1829bb7efa43ccb666c223a`.
- Gameplay, runtime and data canonicals remain respectively `6c53ecd5...`, `ab7d2f98...` and `30ed5fd5...`.
- No event serde carrier, DTO, server message, Unity protocol, save/checkpoint, loader schema, RON value or canonical
  meaning changed. No coordinated runtime rollout is required beyond the explicitly approved Rust API removal.
- Both all-target profiles include 25 live RON loading tests, four live catalog audits, 14 skill runtime contracts
  and four skill scenarios.

## Scoped Diff And Dirty Preservation

- HEAD remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Sole activation baseline was 576 expanded status rows at canonical C-sort SHA-256
  `93625e86d7ea3bce8539875054846dc9d48b1aea3e82526d6c060bfc1326376e`.
- Before this audit artifact the tree is 577 rows at
  `7bb66e69dc1fa7b821f6108744d8db7b139ca5e0553f4c96fa75bcb92e69e3e0`. The exact additional source status row
  is the formerly-clean movement `mod.rs`, whose private-module/re-export delta is required by this candidate.
- `engine.rs`, `rapier_backend.rs`, Core `mod.rs`, `sim.rs` and `combat.rs` were already dirty; activation/transfer
  hashes and current fingerprints bound the attributable Phase-39 hunks. No unrelated path was cleaned or restored.
- Whole-worktree `git diff --check` and workspace format check pass.
- The protected marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`; mtime and ctime remain exactly
  `2026-08-10 22:55:07.178423600 +0900`.

## Validation Results

### Baseline And Transfer Proof

- Old engine family: 19 passed; Rapier baseline: 15 passed; exact authored route: 1 passed; Boss: 25 passed.
- With new retained controls and old carriers still present: Rapier 19/19, engine 19/19, authored route 1/1 and Boss
  25/25. This is accepted transfer-before-removal proof.

### Post-Cleanup Focused Validation

- Engine/default Core: 6/0/0. Rapier: 17/0/0. Complete movement filter: 61/0/0.
- Boss: 25/0/0. Tactical: 34/0/0. Range: 52/0/0. Event: 76/0/0. World: 293/0/0.
- Exact authored route: 1/0/0. Every accepted filter selected at least one test.

### Post-Cleanup Broad And Static Validation

- `cargo test -p game_core --all-targets`: 1,056 passed, 0 failed, 0 ignored.
- `cargo test -p game_core --all-targets --release`: 1,056 passed, 0 failed, 0 ignored.
- Each distribution is library 1,005 plus integrations 4 + 4 + 25 + 14 + 4. Six executable/fixture targets select
  zero tests and are not counted as behavior evidence.
- `cargo check --workspace --all-targets` and release equivalent both compile `game_core` and `game_server`, exit 0.
- `cargo fmt --all -- --check` and whole-worktree `git diff --check`: exit 0.
- `cargo clippy -p game_core --all-targets`: exit 0 with inherited warnings; engine/Rapier target-path diagnostics 0.

### Independent Audit Refresh

- `cargo test -q -p game_core --all-targets`: current 1,056/0/0 with the same distribution.
- Legacy search selects zero live source/test matches. Phase 39 is the sole active implementation Child; the Phase-40
  directory is absent and Master still marks it pending during review.
- The pre-audit 577-row status/hash, protected marker and diff syntax were independently reverified.
- Release, workspace and Clippy commands were not repeated during refresh because all audited source fingerprints
  equal the successful implementation-validation state; only Goal evidence documents changed afterward.

## Failures And Diagnostics

The first post-cleanup Rapier command failed compilation and selected no tests because a `pub(super)` type was
re-exported to the broader Core scope (E0365/E0603); it also exposed one unnecessary `mut`. That attempt is not used
as behavior evidence. Restricting the world explicitly to `crate::game::battle::core` and removing `mut` corrected
the visibility defect; the rerun passed 17/17 and every subsequent focused/broad/static command passed.

The earlier shortened authored-route exact filter selected zero tests because it omitted the full module path. It was
also rejected as evidence and corrected with the exact full path, which passed 1/1. No behavioral test failed.

## Cross-Component And Long-Term Direction Review

| Candidate | Current → target authority | Preserved invariants | Cleanup result | Validation basis | Verdict |
| --- | --- | --- | --- | --- | --- |
| CRA-P07-C03 | public enum plus Direct/Rapier test authorities → one Core-private Rapier world | deterministic order/logs, collision, clamp, overlap, terrain, Boss/route/event behavior | alternate implementation, selectors, walls and false parity removed without replacement | 17-row transfer, focused families, both all-target profiles and workspace/static checks | pass |

- Fit: `High`.
- Improvement class: `none`.
- Reason: the false movement authority disappears without a new abstraction or gameplay change. Tests now protect
  production-default behavior directly, downstream and compatibility boundaries remain exact, and Phase 40 remains
  a separate linear cleanup.
- Material cross-component findings: none.

## Primary Verdict

- Primary verdict: `Complete`.
- Long-term fit: `High`.
- Combined verdict: `Complete / High`.
- Confidence: High. Candidate-scoped approval, assertion transfer before deletion, exact current visibility and
  test accounting, both profiles, workspace/static checks and dirty/marker evidence converge on the same result.

## Status Transition

- Child: `active` / `Implementation Complete: false` → `complete` / `Implementation Complete: true`.
- Master Phase 39: `active` → `complete — Complete / High`; approved public break applied.
- Master Phase 40: `pending` → `ready` only after Child/Master exit records are updated.
- Phase 40 may then be initialized and activated under its own complete-document reread, prospective fingerprint,
  inventory and assertion-transfer gates. This audit did not create or inspect a Phase-40 Child.

## Remaining Risk And Required Action

The Rust API removal is intentionally breaking and explicitly approved. No Phase-39 correction remains. Apply the
record transitions above, preserve the marker and dirty work, initialize Phase 40 only after the transitions, and
leave repository-wide Stage-D validation/final audit for after Phase 44.
