# Phase 39 CRA-P07-C03 Rapier-Only Production Movement Backend Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — the user's 2026-08-09 instruction to execute the Master campaign
Parent Master: `core_enhanced_refactoring_rebaselined_backlog_implementation_master`
Phase: 39 of 44
Candidate / Work Type: CRA-P07-C03 / `cleanup`
Direct Linear Dependency: Phase 38 CRA-P08-C06 `Complete / High`
Direct Technical Dependencies: Phase 08 CRA-P07-C01 and Phase 09 CRA-P07-C02 `Complete / High`; completed Boss
Runtime v1 P03 movement-leaf seam and Master audit
Direct Policy: CRA-POL-006
Repository / Start Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Initialization Date: 2026-08-11 (`Asia/Seoul`)

## Objective

Make the production-default Rapier movement path the single private continuous-movement backend only after every
unique assertion carried by the historical Direct implementation, backend selector and parity/test-only surfaces is
enumerated and transferred to the retained Rapier/default `BattleCore` path. Preserve the ordinary movement leaf
consumed by Boss directives, current tactical/world publication and exact collision/clamp/determinism behavior.
Remove public or externally reachable backend surface only when CRA-POL-006 consumer evidence or an explicit
candidate-scoped breaking/migration decision authorizes it.

## User/Contract-Visible Problem

Historical evidence reports that production always selected Rapier while a public Direct/Rapier backend model,
test-only selection, a second swept-AABB/depenetration implementation and parallel parity assertions remained.
Some authored-route proof even bypassed the production default, while Rapier carried test-only board-wall state that
production did not use. This made tests and public inspection surfaces present two movement authorities although the
runtime had one production choice.

Deleting Direct first would be unsafe: its tests may contain the only direct proof for deterministic ordering,
collision, board clamp, static obstacles, overlap, locks, endpoints or authored-route progress. The assertion value
must move to actual Rapier/default behavior before its carrier disappears, and public Rust narrowing requires more
than a negative workspace search.

## Audit Finding ID

CRA-P07-C03. The rebaselined catalog classifies it `still_valid`, `cleanup`, Tier D, priority axes
`3/1/1/3/3/2/unknown = 13`, gated by assertion transfer and public/external inventory. Exact linear item 39 requires
all 17 Direct-dependent assertions to be inventoried and unique collision/clamp/obstacle/determinism cases to pass
on the retained backend before cleanup.

## Historical Evidence And Current-Proof Gate

Original and post-Boss evidence prospectively locates a `ContinuousMovementBackend` with Direct/Rapier variants,
test-only switching, Direct-only/parity/Core tests, Rapier board-wall fixtures and public inspection methods. It
records that production defaults to Rapier, world/Boss runtime consumes the ordinary movement leaf, and Boss target/
route ownership does not supersede the cleanup.

These are locating hypotheses, not current removal authority. After sole activation this Phase must refresh exact
files, visibility, constructors, selectors, callers, all 17 Direct-dependent tests, retained Rapier/default tests,
Boss/tactical/world consumers, package/public metadata and accessible external consumers against the current dirty
tree. No present source detail is claimed before prospective fingerprints.

## Source Of Truth And Ownership

### Current prospective authority

- Production `BattleCore` prospectively selects the Rapier continuous-movement backend.
- Core planner/body/terrain/lock/lifecycle inputs and active movement segments own movement intent and publication.
- Boss runtime owns target/route/directive state but delegates physical translation to the ordinary movement leaf.
- Historical Direct code, selector, parity tests and test-only Rapier wall/introspection surfaces form a parallel
  implementation/assertion surface whose exact current status remains `discovery pending`.

### Target authority

One private Rapier-backed `BattleCore` movement implementation owns continuous translation. A neutral pure board
clamp and the actual static-obstacle policy remain only where production behavior consumes them. Tests enter through
the default production Core path or retained Rapier behavior, and no alternate backend, runtime selector, parity-as-
authority suite or test-only physics world survives unless a confirmed compatibility contract requires it.

## Invariants

- Unit processing and emitted movement results remain deterministic under insertion/order variation and repeated
  default runs.
- Ground units stop at actual static obstacles without unapproved sliding or tunneling; airborne units preserve the
  established obstacle-bypass policy.
- Manual board clamp, body radius/endpoints and current out-of-bounds handling remain exact.
- Intentional unit overlap and current no-dynamic-unit-collision policy remain exact.
- Block locks, lifecycle/action locks, goal selection, movement epochs and same-time event ordering remain exact.
- Body position, facing, velocity/segment endpoints and immutable movement-segment publication remain exact for all
  current world/range/event consumers.
- Authored routes, ordinary target movement and Boss `MoveToTarget`/rejoin directives keep current progress,
  suppression, fallback and route-cursor behavior.
- Phase 08 final tactical aggregation and Phase 09 staged tactical publication remain unchanged.
- Public Rust/API surface is not narrowed from negative workspace evidence alone.
- No replacement backend abstraction, compatibility shim, fallback, dynamic selector, placeholder or ignored test is
  introduced.
- No RON/content, DTO/wire, V3 save/checkpoint, server/Unity or durable canonical meaning changes.
- Phase 40 ghost-state cleanup is not pulled forward.
- Existing user/campaign dirty work and the protected Phase-27 marker remain exact.

## Refactor Candidate And Behavior-Preservation Boundary

- Candidate ID: CRA-P07-C03.
- Work type: `cleanup`; `still_valid`; Tier D; assertion transfer and external inventory first.
- Priority axes: ownership `3`, correctness/atomicity `1`, lifecycle `1`, change amplification `3`, test fragility
  `3`, failure blast radius `2`, change frequency `unknown`; known subtotal `13/18`.
- Prospective removal: Direct backend implementation, backend selection, Direct-only/parity tests, test-only Rapier
  board walls and unused public instrumentation after exact inventory, transfer and consumer authorization.
- Retained scope: production Rapier/default Core movement, neutral clamp/static-obstacle policy, planner/body/
  segments, tactical/world projections and Boss movement leaf.
- First safety net: classify all 17 Direct-dependent assertions and make every unique behavioral assertion pass
  through retained Rapier/default production behavior before any carrier removal.
- Behavior-preservation boundary: any collision, clamp, overlap, lock, timing/order, published segment, authored
  route, Boss directive, data/DTO/save/wire or public compatibility change beyond explicitly authorized cleanup stops
  implementation.

## Approved Policy And Evidence

- Original P07-C03: production defaults to Rapier; transfer unique Direct behavior and authored-route proof to the
  default path before removing alternate physics, selectors and test-only walls/introspection.
- Post-Boss P07: candidate remains `still_valid`; Boss directives consume the ordinary leaf and add consumers rather
  than a second backend authority.
- Final test catalog: `transfer then remove Direct-only tests`; inventory 17 Direct-dependent tests and port unique
  collision/clamp/static-obstacle/determinism assertions.
- Exact linear item 39 and Wave-4 gate require public/external inventory under CRA-POL-006.
- Phase 08/09 preserve tactical final-state effects and atomic world command publication around the movement seam.
- Boss Runtime P03/Master audits preserve existing movement as the leaf authority and leave P07-C03 as a separate
  cleanup candidate.
- Phase 38 is the independent direct linear predecessor at `Complete / High` and introduces no movement placeholder.
- On 2026-08-11 the user explicitly approved the planned cleanup for all remaining Phases 39~44. For Phase 39 this
  is candidate-scoped breaking authority to remove the public Direct/backend-selection/Rapier implementation and
  test-only wall/introspection surfaces without a compatibility shim or deprecation period after the already-green
  assertion transfer. It does not authorize movement/gameplay or later-Phase mutation inside Phase 39.

## Open Policy Decisions

None for Phase 39. The user approved the exact candidate-scoped breaking cleanup on 2026-08-11. The approval resolves
unknown deployed/standalone compatibility without requiring a shim or deprecation period, but does not waive the
recorded consumer inventory, behavior-preservation checks, dirty-worktree boundary or independent completion audit.

## Current Runtime And Live Data Evidence

Current source now has exactly one stored `RapierMovementWorld` in `BattleCore`; production `sim.rs` executes that
ordinary movement leaf and clears the same world on reset. `rapier_backend` is a private module, its Core-only world
and `clear` method cannot escape `battle::core`, and the engine trait is module-private. The backend enum, Direct
implementation, both selectors, test-only board-wall world and public Rapier inspection/sync surface are absent.

Before removal the retained transfer controls passed Rapier 19/19, engine 19/19, default authored route 1/1 and Boss
movement 25/25 while every old carrier remained. After removal Rapier is 17/17, engine/default Core is 6/6,
movement is 61/61, Boss is 25/25 and the authored route is 1/1. The default-Core suite additionally proves equal
two-tick results, final positions and serialized movement logs under reversed unit insertion order.

The public/external inventory found no repository or accessible `/mnt/f/work` Rust consumer outside the removed
implementation/tests; `game_server` is the only path dependency and did not consume the surface. The package lacks
`publish = false`, so the user's candidate-scoped breaking approval is the recorded compatibility disposition. No
live RON/data, DTO/wire, save, server/Unity or canonical carrier is involved or changed.

## In Scope

- exact current movement file/module/type/visibility/constructor/selector/caller graph;
- exact production default and every Direct/Rapier/backend-selection path;
- all 17 Direct-dependent tests and assertion-by-assertion retain/transfer/remove classification;
- retained Rapier/default collision, clamp, obstacle, overlap, lock, endpoint, determinism and authored-route proof;
- Boss directive, tactical/world, range/event and package/tool consumers needed to prove the boundary;
- public Rust/package/tool/deployment consumer classification under CRA-POL-006;
- test-first assertion transfer and exact consumer-authorized false-authority cleanup;
- focused/broad debug/release/static validation, dirty preservation and a separate completion audit.

## Out Of Scope

- changing movement/collision/gameplay policy or replacing Rapier;
- adding dynamic backend selection, runtime physics configuration or a new abstraction layer;
- Phase-40 removal of unread movement tick/body/output fields or variants;
- refactoring Boss target/route/task authority, tactical commands, projectiles or range queries;
- changing live RON/schema/content, DTO/wire, save/checkpoint, server/Unity or public Boss lifecycle;
- adjacent event-log, range or package cleanup from Phases 41~44;
- unrelated lint, format or dirty-worktree cleanup.

## Implementation Scope

After sole activation and prospective fingerprints:

1. refresh exact module/type/visibility/backend/selector/introspection/test and consumer graphs;
2. prove which backend production and each Boss/tactical/world/authored route actually execute;
3. establish nonzero Direct, Rapier/default, Core movement, Boss movement and authored-route baselines;
4. inventory all 17 Direct-dependent tests and classify every assertion as unique transfer, stronger survivor,
   false parity/shape or required compatibility;
5. add or strengthen retained Rapier/default production controls before deleting any assertion carrier;
6. run transferred assertions against old/current production and repeat deterministic default runs;
7. classify every public symbol/module under CRA-POL-006 and resolve any required approval/migration gate;
8. apply only authorized alternate-backend/selector/wall/introspection/test cleanup with no behavior change;
9. rerun exact discovery, focused/broad debug/release/static checks and dirty/marker verification;
10. perform an independent completion audit and transition linearly only on `Complete / High`.

## DTO, Data, Save And Migration

- DTO/wire migration: none.
- Live RON/data migration: none.
- Save/checkpoint migration: none.
- Runtime event serialization migration: none.
- Public Rust migration: the user explicitly approved immediate candidate-scoped removal with no shim or deprecation
  lifetime after the consumer inventory and retained assertion transfer completed.
- Unity/external migration: none.

## Deployment And Compatibility

Core, game_server, in-workspace tools and accessible repository consumers are one validation unit. Package metadata,
public docs/examples/fixtures/tools and accessible deployment evidence must be inventoried affirmatively. Workspace
absence does not prove standalone/deployed absence. No coordinated external rollout or other-repository edit is
authorized. If required public removal remains unknown, stop Phase 39 after policy-independent assertion transfer
rather than keeping dual backends or silently weakening the target.

## Focused Validation

- exact current backend/module/public-symbol/selector/caller and 17-test inventories;
- nonzero Direct and retained Rapier/default baselines before cleanup;
- test-first transferred controls for deterministic order, ground obstacle stop/no-slide, airborne bypass, manual
  clamp, intentional overlap, locks, endpoints/static sync and authored-route progress;
- actual default-Core Boss Move/Rejoin and ordinary movement controls;
- repeated default-run movement log/final-state equality;
- exact pre/post file/symbol/test delta and zero alternate/shim/placeholder/ignored replacement search.

Every command records selected/pass/fail/ignored counts. Zero-selected filters, compile-only success and testing a
replacement private helper do not count as behavioral proof.

## Broad Validation

At minimum after implementation:

- complete movement, Boss movement, tactical/world position/range/event families in debug and representative release;
- `cargo test -p game_core --all-targets` in debug and release with exact per-target counts;
- workspace all-target checks in debug/release;
- `cargo fmt --all -- --check`, whole-worktree `git diff --check` and normal Core Clippy diagnostics;
- live authored-route/current RON integration if current discovery proves it consumes production data;
- final test-list reconstruction, target/non-target hashes, public/legacy searches and marker verification.

## Omitted Validation

- game_server/Unity runtime probes are provisionally omitted because no DTO/wire or event shape change is approved;
  reconsider if current consumer discovery finds runtime or fixture coupling.
- Save/checkpoint migration probes are omitted because no persistent carrier change is approved.
- Canonical edits are omitted only if movement/gameplay/public serialized meaning remains exact and the diff is
  internal cleanup plus tests. Any semantic or public migration requires policy first.
- Repository-wide Stage-D integration remains a Master requirement after Phase 44.

## Completion Conditions

- Current Direct/Rapier/default/selector/introspection/test/public/consumer graphs are exact.
- All 17 Direct-dependent assertions are enumerated and each has an evidence-backed disposition.
- Every unique collision/clamp/obstacle/determinism/route invariant passes on retained Rapier/default production
  behavior before old carriers disappear.
- One production movement backend remains without selector, fallback, parity authority or test-only physics world,
  subject to explicit CRA-POL-006 authorization for any public narrowing.
- Boss ordinary-leaf movement, tactical/world/range/event publication and all preserved invariants remain exact.
- No Phase-40 field/output cleanup or adjacent later-Phase work is included.
- No RON/data, DTO/wire, save/checkpoint, server/Unity or canonical contract change occurs unless separately approved.
- Focused/broad debug/release/static validation is green with actual nonzero counts and exact test-list delta.
- Dirty work and the protected marker are preserved.
- A separate completion audit returns `Complete / High` before Phase 40 becomes ready.

## Completion Audit Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P39-C01 | current backend/public graph is exact | files, modules, types, visibility, constructors, selectors and callers | pass — one Core-private Rapier world |
| P39-C02 | production/Boss/tactical consumers are exact | default path and ordinary-leaf call graph | pass — ordinary leaf retained |
| P39-C03 | dirty targets and concurrent work are preserved | pre/post status, scoped diffs, hashes and marker | pass — attributable hashes and exact marker |
| P39-C04 | all 17 Direct-dependent assertions are classified | named test/assertion disposition matrix | pass — 17/17 |
| P39-C05 | current Direct and retained behavior are characterized | nonzero old/default/Rapier/Core/Boss/route baselines | pass — all nonzero/green |
| P39-C06 | unique assertions move first | transferred named production tests passing before cleanup | pass — retained controls green; old carriers present |
| P39-C07 | public consumer/removal gate is exact | package/tool/deployment inventory and policy disposition | pass — candidate-scoped breaking cleanup approved |
| P39-C08 | only authorized alternate authority is removed | exact file/module/symbol/test delta and no replacement | pass — 16 carriers removed, one default control added |
| P39-C09 | movement/Boss/world behavior remains exact | focused deterministic/collision/clamp/route evidence | pass — focused families green |
| P39-C10 | DTO/RON/save/server/Unity compatibility is exact | hashes, searches and affected package checks | pass — non-target hashes exact |
| P39-C11 | focused/broad/static validation is green | actual test counts, profiles and failure classification | pass — debug/release 1,056/1,056 |
| P39-C12 | independent audit and next boundary are current | `Complete / High`; Phase 40 absent until exit | pass — `Complete / High` audit |

## Activation Protocol

Before Phase-39-directed production content, caller, test or consumer inspection:

1. Master records Phase 38 `complete — Complete / High`, Phase 39 as `ready` and Phases 40~44 pending.
2. Change all three Child headers from `initialized` to `active`; `Implementation Complete` remains `false`, and
   change Master Phase 39 from `ready` to `active`.
3. Fully reread this PLAN, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, the Phase-38 completion audit, Phase-08/09
   technical dependency audits, Boss P03/Master audits, historical P07-C03 evidence and final catalog/order/test/
   gate/policy records.
4. Capture HEAD, expanded status count/hash and filename-only/path-known prospective status/hashes for movement,
   Boss/tactical/world consumers, manifests, data/canonicals and dependency inputs without opening current content.
5. Record activation evidence before current-tree source, caller, test or external-consumer investigation.

Historical audit observations do not waive this gate.

## Stop Conditions

Stop and do not activate Phase 40 if any of the 17 assertions is missing or only tests a replacement helper; retained
Rapier/default behavior cannot express a unique invariant; movement/Boss/tactical/world/data/DTO/save meaning
changes; public removal requires unrecorded approval; dirty overlap cannot be separated; a Phase-40+, server, Unity,
RON, canonical or package change is required; any mandatory test fails or selects zero; the protected marker changes;
or the independent audit is not `Complete / High`.

## Initialization Dirty Baseline

- Expanded porcelain status before this Child: 573 rows.
- Canonical C-sort SHA-256:
  `a086d6ad75aae503d2da3649bc84bf8d38605e21b0466c9b5f15ed1c6251b149`.
- Phase 39 and Phase 40 paths were absent; active implementation Child count 0.
- Protected marker: 57 bytes, SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`.
- These three new Child files are the only expected status-set additions before activation evidence.
- All existing rows, including completed Phase-38 source/test/doc changes, are user/campaign-owned and preserved.
