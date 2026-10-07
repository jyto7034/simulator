# Phase 40 CRA-P07-C04 Ghost Movement Tick State Cleanup Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — the user's 2026-08-11 instruction approves all remaining registered cleanup
Parent Master: `core_enhanced_refactoring_rebaselined_backlog_implementation_master`
Phase: 40 of 44
Candidate / Work Type: CRA-P07-C04 / `cleanup`
Direct Linear And Technical Dependency: Phase 39 CRA-P07-C03 `Complete / High`
Direct Policy: CRA-POL-006
Repository / Start Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Initialization Date: 2026-08-11 (`Asia/Seoul`)

## Objective

Reduce the surviving single-backend movement tick model to the values and transitions that current production
actually consumes. Remove only fields, stored body state, clone/write churn and never-produced outputs proven ghost
by a complete current reader/writer/constructor/serde/public-consumer inventory. Preserve production Rapier/default
movement, Core-owned body facts, active movement segments and every world/range/event/Boss observable.

## User/Contract-Visible Problem

Historical evidence reports that movement input, per-unit body plumbing and output enums carry placeholder lifecycle
state with no production reader or producer. Such fields present a second-looking goal/segment/physics lifecycle even
though current authority is Core body position/radius/speed, the current planner goal, terrain/locks/lifecycle and
active movement segments.

Historical names are locating hypotheses, not deletion authority. Phase 39 changed the movement backend surface, so
this Phase must inventory the current retained model from scratch and prove every proposed removal against actual
world position, range, event, route, blocked/stopped and external Rust/serde consumers.

## Audit Finding ID

CRA-P07-C04. The rebaselined catalog classifies it `still_valid`, `cleanup`, Tier D, priority axes
`2/0/2/2/2/1/unknown = 9`. Exact linear item 40 requires P07-C03 completion, CRA-POL-006 consumer evidence and
field-use plus world position/range/event parity for each removal.

## Historical Evidence And Current-Proof Gate

The original audit prospectively named `MovementTickInput.now_ms`, per-unit owner/current-target data, body mode and
physics-handle state, stored goal/previous position/velocity, stopped payload semantics and never-produced
`GoalAssigned`/`SegmentStarted` outputs. The post-Boss reaudit found private goal/segment/physics scaffolding still
valid for review while Boss directives remained ordinary movement-leaf consumers.

None of those historical members is assumed to exist or remain ghost now. After sole activation, current code must
establish exact definitions, derives/serde, constructors, reads, writes, match arms, tests, docs and accessible
external consumers before baseline, test transfer or deletion.

## Source Of Truth And Ownership

### Prospective current authority

- Phase 39 proves one Core-private Rapier world owns physical translation.
- `BattleCore`/`RuntimeUnit` prospectively own body facts used by spatial/range/world consumers.
- Current planner goals, terrain/locks/lifecycle and active movement segments prospectively own intent/publication.
- Exact surviving fields, output transitions and consumer graph remain `discovery pending`.

### Target authority

One minimal internal input view feeds the private Rapier/default movement leaf. It carries only values actually read
for physical resolution and current goal interpretation. Outputs contain only transitions `BattleCore` applies.
Runtime/Core retain every authoritative body and active-segment fact needed for spatial, range, facing, events and
Boss movement. No compatibility wrapper or placeholder survives unless current evidence proves it is required.

## Invariants

- Phase 39's single private Rapier backend, deterministic input/output order and repeated default-run log/final-state
  equality remain exact.
- World coordinates, body radii/speed, board clamp, ground/air obstacle policy and intentional overlap remain exact.
- Current planner goals, route progress, blocked/stopped behavior, target reach and Boss Move/Rejoin remain exact.
- Opponent facing, body endpoint, move epoch, action/lifecycle locks and relocation clearing remain exact.
- Active movement-segment fields, quantization, immutable event order/timestamps and all world/range/event consumers
  remain exact.
- Approved event-log stop reasons and any actual public DTO/serde contract remain exact unless a member is proven
  unproduced/unconsumed and covered by the user's cleanup approval.
- No Phase-41+ field, event-log I/O, range/delivery or package cleanup is pulled forward.
- Existing dirty work and the protected Phase-27 marker remain exact.

## Refactor Candidate And Behavior-Preservation Boundary

- Candidate: CRA-P07-C04; `cleanup`; `still_valid`; Tier D; known score `9/18`.
- Prospective removal: only current fields/variants/stored writes/clones proven to have no semantic reader, producer,
  serialization consumer or retained test authority.
- Retained scope: private Rapier world, actual movement inputs/outputs, Core body facts, planner/terrain/lock/lifecycle,
  active segments, facing, quantization and downstream world/range/event/Boss behavior.
- First safety net: exact field/variant disposition matrix plus before/after default moving, stopped and blocked logs,
  final positions and downstream family controls.
- Boundary: any movement policy, output ordering, serialized shape with a surviving consumer, route/Boss behavior or
  later-Phase concern stops cleanup and is not disguised as ghost-state removal.

## Approved Policy And Evidence

- Original P07-C04: remove unread fields and never-produced variants only after C03, retaining actual Core body,
  planner, terrain/lock/lifecycle, segment/facing/event/DTO authority.
- Boss execution registry and post-Boss reaudit: Boss adds ordinary-leaf consumers; it does not own or close broad
  ghost-state cleanup.
- Final test catalog: retain and target removal using compile/field-use inventory and world position/range/event
  publication parity after C03.
- Exact linear item 40: every removed member must have field-use inventory and downstream parity evidence.
- Phase 39 audit: one private Rapier world is final, all movement assertions are on retained production behavior, and
  Phase 40 is ready.
- User approval on 2026-08-11 authorizes each remaining registered public/serde cleanup after its own inventory and
  behavior guards, with no shim/deprecation pause. It does not authorize gameplay or live contract changes.

## Open Policy Decisions

None at initialization. Public/serde removal is authorized only for members the current inventory proves ghost. If a
member has an unknown surviving serialized/deployed consumer or its removal changes meaning rather than false shape,
stop that member and record the exact unresolved contract.

## In Scope

- exact current movement input/unit/body/output definitions, visibility, derives and module reachability;
- all constructor, read, write, clone, match, serde, fixture/mock/test, docs/tool and external-consumer use;
- field/variant-by-field disposition: retain, transfer then remove, proven shape-only removal or out of scope;
- default moving/stopped/blocked/target-reached/facing/segment logs and final state before and after cleanup;
- Boss ordinary-leaf, route, world position, tactical, range and event consumers;
- public Rust/serde/package/deployment classification under CRA-POL-006;
- focused/broad debug/release/static validation and separate completion audit.

## Out Of Scope

- changing movement/collision/target/range/facing/stopped or blocked gameplay policy;
- changing Rapier selection or adding another movement abstraction/backend;
- removing `ActiveMovementSegment`, quantization, action locks, move epoch or approved event stop reasons merely
  because historical evidence mentioned adjacent ghost state;
- Phase 41 `ScenarioEvent.once`, Phase 42 battle-log file I/O, Phase 43 range/delivery surfaces or Phase 44 package;
- live RON/content, DTO/wire, save/checkpoint, server/Unity or canonical semantic migration;
- unrelated lint, formatting or dirty-worktree cleanup.

## Implementation Scope

1. initialize and solely activate this Child before current production inspection;
2. capture prospective filename/status/hash boundaries for movement and downstream consumers;
3. enumerate every current candidate member and all producers/consumers/serialization/test uses;
4. establish nonzero default moving/stopped/blocked/target/Boss/world/range/event baselines;
5. add or strengthen retained production-path controls before removing assertion carriers;
6. apply only member-level cleanup authorized by the disposition matrix and user approval;
7. repeat exact searches, focused/broad profiles, workspace/static checks and dirty/marker verification;
8. perform a separate completion audit; make Phase 41 ready only on `Complete / High`.

## DTO, Data, Save And Migration

- DTO/wire migration: none planned.
- Live RON/data migration: none planned.
- Save/checkpoint migration: none; active battle movement is not durable save state per historical evidence, to be
  reverified against current code.
- Runtime event migration: none planned; actual event-log shape/order must remain exact.
- Public Rust/serde migration: immediate removal is approved only for members proven ghost after current inventory.
- Unity/server migration: none planned.

## Deployment And Compatibility

Core, `game_server`, in-workspace tools/tests/docs and accessible repository consumers are one validation unit.
Public derives/module reachability, package metadata and accessible deployment evidence must be inventoried. Unknown
external existence does not itself prove a member used, but the user's approval applies only after affirmative proof
that the member has no surviving contract. No dual shape, alias, fallback or deprecation wrapper is authorized.

## Focused Validation

- exact member/field/variant producer-consumer/serde/public disposition matrix;
- pre/post default-Core moving, stopped, blocked and target-reached outputs, final body state and event-log equality;
- Rapier obstacle/clamp/overlap, planner route/block, immutable segment, facing and relocation clearing controls;
- Boss Move/Rejoin and authored route; world position/range/event publication parity;
- exact public/legacy/placeholder search and test-list delta.

Every accepted command records selected/pass/fail/ignored counts. Zero-selected filters or compile-only success do not
prove behavior.

## Broad Validation

- complete movement, Boss, tactical, range, event and world families in debug; representative release;
- `cargo test -p game_core --all-targets` in debug and release with exact distribution;
- workspace all-target checks in debug and release;
- normal Core Clippy, `cargo fmt --all -- --check` and whole-worktree `git diff --check`;
- current live RON integration if discovery proves relevant;
- final test-list/member reconstruction, target/non-target hashes and protected-marker verification.

## Omitted Validation

- game_server/Unity runtime probes are omitted only if no DTO/wire/mapping shape changes and current searches prove no
  transport coupling; otherwise they become mandatory.
- save migration probes are omitted only if current inventory proves movement tick state is not persisted.
- canonical edits are omitted only if removal is private/false shape with no gameplay or serialized contract change.
- repository-wide Stage-D integration remains a Master task after Phase 44.

## Completion Conditions

- Every current candidate field/variant has a named producer/reader/serde/test/public disposition.
- Each removed member is proven to have no surviving runtime, data, DTO/save, external or assertion contract.
- Retained production-path tests pass before obsolete carrier removal.
- One minimal internal movement tick contract remains with no false lifecycle, compatibility shim or placeholder.
- All movement/Boss/world/range/event/facing/segment invariants remain exact.
- No Phase-41+ concern or unrelated dirty work is changed.
- Focused/broad debug/release/static checks are green with exact counts and failure classification.
- Non-target hashes and protected marker are exact.
- A separate completion audit returns `Complete / High`.

## Completion Audit Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P40-C01 | current member/public graph is exact | definitions, derives, constructors, producers, readers, matches and consumers | pass |
| P40-C02 | production authority graph is exact | planner→input→Rapier→Core apply→world/event path | pass |
| P40-C03 | dirty work is preserved | activation/current hashes, scoped diffs and marker | pass |
| P40-C04 | every candidate member is classified | complete named disposition matrix | pass |
| P40-C05 | current behavior is characterized | nonzero moving/stopped/blocked/target/Boss/world/range/event baselines | pass |
| P40-C06 | useful assertions move first | retained production controls green before cleanup | pass |
| P40-C07 | public/serde removal gate is exact | consumer/package/deployment inventory and approval mapping | pass |
| P40-C08 | only proven ghost state is removed | exact member/source/test delta; no replacement placeholder | pass |
| P40-C09 | movement/Boss/downstream behavior is exact | pre/post logs, final state and focused families | pass |
| P40-C10 | DTO/data/save/server/Unity boundary is exact | hashes/searches/affected package checks | pass |
| P40-C11 | broad/static validation is green | both profiles, workspace, format/diff and Clippy | pass |
| P40-C12 | independent audit/next boundary is current | `Complete / High`; Phase 41 absent until exit | pass |

## Stop Conditions

Stop and do not activate Phase 41 if any member has an unclassified reader/producer/serialized consumer; a removed
field carries actual behavior or assertion authority; movement/Boss/world/range/event meaning changes; a DTO/save/
wire/data/canonical migration is required beyond approval; dirty overlap cannot be isolated; a Phase-41+ change is
required; mandatory tests fail or select zero; the protected marker changes; or the audit is not `Complete / High`.

## Initialization Dirty Baseline

- Expanded status before this Child: 578 rows at canonical C-sort SHA-256
  `4665ba9b1fd62439bf4f6136c9e3098b988ed353f67cd6178a5d8c2512fd7dbd`.
- Phase 40 path was absent and active implementation Child count was zero.
- Phase 39 completion audit is 232 lines at SHA-256
  `456b37e104e49298403d9ff9a4c37ef570bddac069bd388424882bf841cc993d` with verdict `Complete / High`.
- Protected marker is 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`.
- These three new Child documents are the only expected status-set additions before activation evidence.
