# Core Enhanced Rebaselined Backlog Implementation Master Goal

Status: complete
Implementation Complete: true
Goal Type: master
Execution Mode: orchestration_only
Production Mutation Authorized: yes — the user's 2026-08-09 request to read and perform this PLAN activates the
campaign; mutation remains limited to the single active Child Phase
Repository Root: `/mnt/f/work/simulator`
Initialization Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
Initialization Date: 2026-08-09 (`Asia/Seoul`)

## Overall Objective And Completion Conditions

Implement and independently verify all 44 `still_valid` identities from the post-Boss Runtime refactoring reaudit,
one candidate-scoped Phase at a time and in the exact dependency order defined by the Final consolidation. The
Master is complete only when every Phase is `Complete / High`, all six approved policy IDs are connected to current
production evidence, tests and canonical documents, no unauthorized compatibility/migration remains, and an
independent Master final audit passes.

This Master coordinates scope, order, policy coverage, cross-Phase invariants and final validation. It never edits
production directly; the single active Phase owns its investigation, baseline, production implementation,
verification, canonical update and completion audit.

## User/Contract-Visible Problem

The current tree has 44 confirmed open findings: 17 correctness/test corrections, 17 behavior-preserving authority
refactors and 10 cleanup items. They include fail-open publication, partial mutation, invalid checkpoint admission,
incorrect HP/death and targeting behavior, duplicate runtime/publication/projection authorities, false test models
and public/serialized cleanup gates. Leaving them as one unordered backlog risks relying on false-positive tests,
refactoring unstable behavior or deleting a surface before its real consumers and assertions are known.

The required result is not merely “all tests green.” Each finding must move from its audited current authority to its
named target authority, preserve its explicit invariants, add the missing behavior-focused safety net and close its
legacy/removal scope without changing unrelated contracts.

## Audit Finding IDs And Authoritative Inputs

- Finding IDs: the exact 44 open CRA identities in
  [REBASELINED_CANDIDATE_CATALOG.md](../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/REBASELINED_CANDIDATE_CATALOG.md).
- Strict execution order and per-candidate first proof:
  [EXACT_LINEAR_IMPLEMENTATION_ORDER.md](../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md).
- Test disposition and safety nets:
  [TEST_REBASELINE_CATALOG.md](../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md).
- Dependency and consumer gates:
  [IMPLEMENTATION_WAVES_AND_GATES.md](../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/IMPLEMENTATION_WAVES_AND_GATES.md).
- Approved target policies:
  [APPROVED_POLICY_DECISIONS.md](../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/APPROVED_POLICY_DECISIONS.md).
- Detailed current evidence: the matching P01~P10 `CANDIDATE_REAUDIT.md`, `TEST_REAUDIT.md` and
  `BOUNDARY_AUDIT.md` documents in the same reaudit campaign.

Historical audit prose is evidence input, not current runtime authority. Every Phase must recheck its exact current
source/data/test/canonical paths before the first production change.

## Current Runtime And Live Data Evidence

- The 2026-08-09 reaudit mapped 53 source candidates into 51 final identities: 7 fully implemented and 44 still
  valid. No distinct new Boss Runtime candidate was confirmed.
- P01~P10 selected 970 physical test executions with zero selected failures/ignores, but assertion review proved the
  44 named gaps remain; those historical results are not a future Phase baseline.
- Current production uses strict embedded live RON publication, V3 checkpoint carriage, private non-serde Boss
  scheduling, ordinary movement/effect/damage/death leaves and the existing Core/server/Unity-facing battle shape.
- CRA-POL-001~005 now define the five formerly open behavior choices. CRA-POL-006 freezes current V3/RON/wire/public
  API compatibility by default.
- The repository is intentionally dirty and contains user-owned production, test, data, canonical and Goal changes.
  Each Phase must treat the current dirty tree—not `HEAD` alone—as its baseline.

Any material drift after this Master initialization is `discovery pending` until the active Phase rereads the
affected path. A Phase may not rely on the reaudit date to overwrite newer user work.

## Approved Policy Package And IDs

The approved package is CRA-POL-001~006 in Final `APPROVED_POLICY_DECISIONS.md`. It is a target decision record for
implementation; durable canonical documents remain current-state contracts until the owning Phase completes.

## Policy Coverage Matrix

| Policy ID   | Approved contract                                                                             | Phase coverage                                                                      |
| ----------- | --------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| CRA-POL-001 | lethal MaxHealth clamping enters normal death finalization with causal provenance/kill credit | 12 P08-C02; 28 P08-C05                                                              |
| CRA-POL-002 | OnHit Counterpart BonusDamage is one non-recursive retaliation against the attacker           | 11 P08-C01; 13 P08-C04; 28 P08-C05                                                  |
| CRA-POL-003 | invalid projectile/basic definitions fail strict atomic publication                           | 15 P09-C01; 29 P09-C05                                                              |
| CRA-POL-004 | cast usefulness is any-step; execution usefulness is current-step over actual delivery sets   | 17 P09-C03                                                                          |
| CRA-POL-005 | public effect preview is the deterministic union of step footprints with the existing DTO     | 18 P09-C04; 34 P10-C03                                                              |
| CRA-POL-006 | current V3/RON/wire/public API compatibility is preserved unless separately migrated          | all Phases; direct save/RON/wire/public cleanup gates are called out in child PLANs |

Policy prose is referenced, not copied into each child. Every Phase PLAN and completion audit must cite all directly
applicable IDs. Incidental refactoring cannot supersede a policy.

## Open Policy Decisions

None at Master initialization. The five discovered gameplay/publication choices and the compatibility posture are
approved as CRA-POL-001~006.

If implementation requires a new gameplay meaning, DTO/RON/V3 shape, compatibility interval, deployment migration,
fallback, public Boss wire, admin lifecycle or external breaking change not covered by those IDs, the active Phase
must stop as `policy_decision_required`; the Master must not infer an answer.

## Common Source Of Truth And Invariants

Evidence priority follows `core_enhanced/docs/README.md`: current runtime code, production live data, actual
Unity-facing producer/consumer contract, durable canonical and only then Goal/history.

The following invariants apply to every Phase:

1. One candidate identity and one contract boundary per Phase; no adjacent backlog item is implemented early.
2. Test or characterization proof precedes production behavior/refactor/cleanup changes.
3. Failure-atomic, deterministic and exactly-once state transitions remain explicit where the candidate touches
   state, event order, checkpoint, delivery or projection.
4. Current live RON continues to load; invalid inputs fail at the approved authority with exact provenance and no
   silent normalization or partial publication.
5. V3 checkpoint shape and current private active-battle/Boss exclusion remain exact; no repair/rebind/old-version
   compatibility is introduced.
6. Current command/result tags, payloads, nesting, direct-map RunComplete, setup/update and resync topology remain
   exact unless a separately approved migration replaces CRA-POL-006.
7. Boss runtime remains private/non-serde and continues to use ordinary movement/projectile/effect/damage/death
   leaves; no speculative first public Boss wire is added.
8. Public/serialized symbols are not removed because workspace search is negative. Required consumer inventory,
   deprecation/versioning or explicit breaking approval must exist first.
9. No dual-read/write, fallback, alias, ignored test, placeholder state/schema or test-only production branch.
10. Initial and concurrent user-owned dirty changes are preserved; no reset, restore, clean, stash or unrelated
    formatting/cleanup.
11. Canonical documents are updated only after a Phase implementation is real; Goal documents never replace them.
12. A Phase is complete only after the completion review guide yields `Complete / High`.

## Refactor Campaign Composition

| Work type                                             |  Count | Completion interpretation                                                                                 |
| ----------------------------------------------------- | -----: | --------------------------------------------------------------------------------------------------------- |
| correctness correction, including one test correction |     17 | approved contract is directly enforced with a defect-reproducing test and no partial/silent behavior      |
| behavior-preserving refactor                          |     17 | target authority is singular while observable behavior, serialization, ordering and failures remain exact |
| cleanup                                               |     10 | unique assertions are transferred and every relevant consumer is inventoried before removal/depublication |
| **Total**                                             | **44** | exact catalog set, no duplicate or implicit merged implementation Phase                                   |

Candidate-specific priority axes, current/target authority, producer/consumer, preserved invariants and
removal/consolidation scope remain owned by the Final/Phase reaudit inputs and must be restated with current code
paths in the activated child PLAN. Scores are ordering evidence, never completion verdicts.

## Master In Scope / Out Of Scope

### In scope

- coordination of the exact 44 Phase Goals in the registry below;
- current-tree investigation and candidate-scoped production/test/canonical changes owned by the active Phase;
- Core, game_server and production data surfaces directly required by a candidate's existing contract boundary;
- accessible external consumer/canonical read and exact fixture comparison where the candidate requires it, except
  the explicitly excluded, independently stale Unity client recorded on 2026-08-12;
- focused baseline, test-first implementation, affected integration/broad validation and per-Phase completion audit;
- cross-Phase policy, dependency, dirty-preservation and final authority/legacy audit.

### Out of scope

- the seven already closed identities or reopening Boss Runtime work without new current evidence;
- new gameplay/content/balance/UX, hidden Boss content, admin durability/publication or first public Boss wire;
- unrelated game_server, game_resources, Unity, tooling or documentation refactors;
- Unity source/fixture synchronization and Unity acceptance are explicitly excluded from this Core campaign by the
  user's 2026-08-12 scope decision; discovered Unity drift is follow-up evidence, not a Master completion gate;
- V3/RON/DTO schema migration, compatibility reader/writer, fallback or breaking public API change without a new
  explicit user policy;
- repository history changes, commit/branch/stash operations, broad formatter runs and worktree cleanup;
- deleting or rewriting historical audit/Goal evidence as though it were current canonical;
- absorbing newly discovered independent problems into the active Phase.

Out-of-scope findings go to the active Phase `EXPERIMENT_NOTES.md` with an owner/evidence pointer only.

## Implementation Scope

Master production implementation scope is `none`: it owns orchestration documents, Phase activation/status,
cross-Phase validation and final audit. Each activated child owns exactly one candidate's current-tree investigation,
production/test/canonical changes and verification under `implement_and_verify`. No future child implementation is
implicitly authorized or performed by writing this Master.

## Production Authorization And Phase Activation

This Master authoring request does not authorize production mutation. No child directory is initialized and no
runtime/data/test/canonical file is changed until the user explicitly activates the implementation campaign or
Phase 01.

After authorization:

1. Master changes Phase 01 from `deferred_by_user` to `active` and initializes its required documents.
2. That child records `Goal Type: phase`, `Execution Mode: implement_and_verify` and the exact production mutation
   authorization source.
3. Only that Phase may investigate/mutate production.
4. After its independent `Complete / High` audit, Master records evidence, marks the next Phase `ready`, then
   activates only that Phase.
5. No Phase is skipped, merged or run in parallel without an explicit Master update and, if outcome/scope changes,
   user approval.

Default active Phase count is exactly one. Parallel execution is prohibited because the order includes shared
source/test/canonical seams and the user requested a linear correction campaign.

## Phase Dependency And Status Table

CRA-POL-006 applies globally. “Dependency” below is the activation gate; technical edges and per-candidate first
proofs are in `EXACT_LINEAR_IMPLEMENTATION_ORDER.md`. Each child name is reserved here but its directory is created
only when activated.

| No. | Candidate   | Work type                     | Child Goal name                                                                 | Dependency                           | Direct policy            | Coordination status                                                                                         |
| --: | ----------- | ----------------------------- | ------------------------------------------------------------------------------- | ------------------------------------ | ------------------------ | ----------------------------------------------------------------------------------------------------------- |
|  01 | CRA-P09-C08 | correctness correction (test) | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_01_p09_c08` | Master initialized + user activation | —                        | complete — `Complete / High`                                                                                |
|  02 | CRA-F-C01   | correctness correction        | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_02_f_c01`   | Phase 01 Complete / High             | CRA-POL-006              | complete — `Complete / High`                                                                                |
|  03 | CRA-P02-C01 | correctness correction        | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_03_p02_c01` | Phase 02 Complete / High             | —                        | complete — `Complete / High`                                                                                |
|  04 | CRA-P03-C02 | correctness correction        | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_04_p03_c02` | Phase 03 Complete / High             | —                        | complete — `Complete / High`                                                                                |
|  05 | CRA-P04-C01 | correctness correction        | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_05_p04_c01` | Phase 04 Complete / High             | CRA-POL-006              | complete — `Complete / High`                                                                                |
|  06 | CRA-P05-C01 | correctness correction        | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_06_p05_c01` | Phase 05 Complete / High             | CRA-POL-006              | complete — `Complete / High`                                                                                |
|  07 | CRA-P06-C05 | correctness correction        | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_07_p06_c05` | Phase 06 Complete / High             | —                        | complete — `Complete / High`                                                                                |
|  08 | CRA-P07-C01 | correctness correction        | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_08_p07_c01` | Phase 07 Complete / High             | —                        | complete — `Complete / High`                                                                                |
|  09 | CRA-P07-C02 | correctness correction        | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_09_p07_c02` | Phase 08 Complete / High             | —                        | complete — `Complete / High`                                                                                |
|  10 | CRA-P08-C03 | correctness correction        | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_10_p08_c03` | Phase 09 Complete / High             | —                        | complete — `Complete / High`                                                                                |
|  11 | CRA-P08-C01 | correctness correction        | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_11_p08_c01` | Phase 10 Complete / High             | CRA-POL-002              | complete — `Complete / High`                                                                                |
|  12 | CRA-P08-C02 | correctness correction        | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_12_p08_c02` | Phase 11 Complete / High             | CRA-POL-001              | complete — `Complete / High`                                                                                |
|  13 | CRA-P08-C04 | correctness correction        | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_13_p08_c04` | Phase 12 Complete / High             | CRA-POL-002              | complete — `Complete / High`                                                                                |
|  14 | CRA-P09-C02 | correctness correction        | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_14_p09_c02` | Phase 13 Complete / High             | —                        | complete — Complete / High                                                                                  |
|  15 | CRA-P09-C01 | correctness correction        | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_15_p09_c01` | Phase 14 Complete / High             | CRA-POL-003              | complete — `Complete / High`                                                                                |
|  16 | CRA-P09-C06 | behavior-preserving refactor  | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_16_p09_c06` | Phase 15 Complete / High             | —                        | complete — `Complete / High`                                                                                |
|  17 | CRA-P09-C03 | correctness correction        | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_17_p09_c03` | Phase 16 Complete / High             | CRA-POL-004              | complete — `Complete / High`                                                                                |
|  18 | CRA-P09-C04 | correctness correction        | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_18_p09_c04` | Phase 17 Complete / High             | CRA-POL-005              | complete — `Complete / High`                                                                                |
|  19 | CRA-P01-C03 | behavior-preserving refactor  | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_19_p01_c03` | Phase 18 Complete / High             | CRA-POL-006              | complete — `Complete / High`                                                                                |
|  20 | CRA-P02-C03 | behavior-preserving refactor  | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_20_p02_c03` | Phase 19 Complete / High             | CRA-POL-006              | complete — `Complete / High`                                                                                |
|  21 | CRA-P02-C04 | behavior-preserving refactor  | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_21_p02_c04` | Phase 20 Complete / High             | CRA-POL-006              | complete — `Complete / High`                                                                                |
|  22 | CRA-P03-C03 | behavior-preserving refactor  | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_22_p03_c03` | Phase 21 Complete / High             | CRA-POL-006              | complete — `Complete / High`                                                                                |
|  23 | CRA-P03-C04 | behavior-preserving refactor  | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_23_p03_c04` | Phase 22 Complete / High             | —                        | complete — `Complete / High`                                                                                |
|  24 | CRA-P04-C03 | behavior-preserving refactor  | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_24_p04_c03` | Phase 23 Complete / High             | —                        | complete — `Complete / High`                                                                                |
|  25 | CRA-F-C02   | behavior-preserving refactor  | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_25_f_c02`   | Phase 24 Complete / High             | —                        | complete — `Complete / High`                                                                                |
|  26 | CRA-P04-C04 | behavior-preserving refactor  | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_26_p04_c04` | Phase 25 Complete / High             | —                        | complete — `Complete / High`                                                                                |
|  27 | CRA-P05-C03 | behavior-preserving refactor  | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_27_p05_c03` | Phase 26 Complete / High             | CRA-POL-006              | complete — `Complete / High`                                                                                |
|  28 | CRA-P08-C05 | behavior-preserving refactor  | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_28_p08_c05` | Phase 27 Complete / High             | CRA-POL-001, CRA-POL-002 | complete — `Complete / High`                                                                                |
|  29 | CRA-P09-C05 | behavior-preserving refactor  | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_29_p09_c05` | Phase 28 Complete / High             | CRA-POL-003              | complete — Complete / High                                                                                  |
|  30 | CRA-P10-C01 | behavior-preserving refactor  | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_30_p10_c01` | Phase 29 Complete / High             | CRA-POL-006              | complete — `Complete / High`                                                                                |
|  31 | CRA-P10-C04 | behavior-preserving refactor  | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_31_p10_c04` | Phase 30 Complete / High             | CRA-POL-006              | complete — `Complete / High`                                                                                |
|  32 | CRA-P10-C05 | behavior-preserving refactor  | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_32_p10_c05` | Phase 31 Complete / High             | CRA-POL-006              | complete — `Complete / High`                                                                                |
|  33 | CRA-P10-C02 | behavior-preserving refactor  | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_33_p10_c02` | Phase 32 Complete / High             | CRA-POL-006              | complete — Complete / High                                                                                  |
|  34 | CRA-P10-C03 | behavior-preserving refactor  | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_34_p10_c03` | Phase 33 Complete / High             | CRA-POL-005, CRA-POL-006 | complete — `Complete / High`                                                                                |
|  35 | CRA-P10-C07 | cleanup                       | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_35_p10_c07` | Phase 34 Complete / High             | —                        | complete — `Complete / High`                                                                                |
|  36 | CRA-P05-C05 | cleanup                       | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_36_p05_c05` | Phase 35 Complete / High             | CRA-POL-006              | complete — `Complete / High`                                                                                |
|  37 | CRA-P06-C04 | cleanup                       | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_37_p06_c04` | Phase 36 Complete / High             | CRA-POL-006              | complete — `Complete / High`; approved public break applied                                                 |
|  38 | CRA-P08-C06 | cleanup                       | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_38_p08_c06` | Phase 37 Complete / High             | CRA-POL-006              | complete — `Complete / High`; approved public break applied                                                  |
|  39 | CRA-P07-C03 | cleanup                       | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_39_p07_c03` | Phase 38 Complete / High             | CRA-POL-006              | complete — `Complete / High`; approved public break applied                                                  |
|  40 | CRA-P07-C04 | cleanup                       | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_40_p07_c04` | Phase 39 Complete / High             | CRA-POL-006              | complete — `Complete / High`; approved public/serde break applied                                            |
|  41 | CRA-P06-C06 | cleanup                       | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_41_p06_c06` | Phase 40 Complete / High             | CRA-POL-006              | complete — `Complete / High`; approved public break applied                                                  |
|  42 | CRA-P06-C07 | cleanup                       | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_42_p06_c07` | Phase 41 Complete / High             | CRA-POL-006              | complete — `Complete / High`; approved public break applied                                                  |
|  43 | CRA-P09-C07 | cleanup                       | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_43_p09_c07` | Phase 42 Complete / High             | CRA-POL-006              | complete — `Complete / High`; approved public/serde break applied                                           |
|  44 | CRA-P10-C06 | cleanup                       | `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_44_p10_c06` | Phase 43 Complete / High             | CRA-POL-006              | complete — Complete / High; Master final audit pending                                                       |

## Phase Goal Initialization Contract

When a Phase becomes active, initialize its `PLAN.md`, `EXPERIMENTS.md` and `EXPERIMENT_NOTES.md` before any new
production search, test or change. The child PLAN must include all single/Phase fields from the authoring guide and:

- exact Candidate ID, work type, priority axes/subtotal and current disposition;
- current code/data/test paths proving the problem, refreshed against that activation's dirty tree;
- current authority, producer/consumer and target authority;
- preserved behavior/serialization/order/failure invariants and exact removal/consolidation scope;
- direct CRA-POL IDs, external gate and no-open-policy statement;
- candidate-specific focused baseline and the Final test-catalog first proof;
- production mutation authorization source and precise affected repositories;
- DTO, RON, save/checkpoint, deployment and compatibility disposition;
- canonical documents to update if and only if actual contract behavior changes;
- focused/broad validation, exact stop conditions and completion evidence matrix.

The activated child records current repository status, target-file diffs and a content manifest sufficient to prove
that pre-existing user changes survive. Prior Phase results can be inputs but are never substituted for a current
baseline when code/data/dependency state changed.

## Shared Migration And Version Strategy

### DTO and wire

No DTO/tag/nesting migration is approved. P09-C04 changes preview content under the existing union representation;
P10-C02~C05 preserve exact serialized results, setup/update/resync topology, host overlays and error codes. A wire
shape change stops the active Phase for policy and consumer rollout approval.

### Live RON/data

No RON schema or authored balance/content migration is approved. Publication corrections reject newly explicit
invalid combinations but every current production bundle must still load. Silent normalization, partial startup and
fixture-only production exceptions are forbidden.

### Save/checkpoint

No V3 shape/version migration is approved. F-C01 adds strict current-document graph admission; P02-C03/C04 and
P03-C03 preserve exact fields, exclusions and restore behavior. Repair/rebind/old-version readers require new policy.

### Public Rust/API and cleanup

CRA-POL-006 requires consumer inventory plus deprecation/versioned migration or new breaking approval before
narrowing public/serialized/package surfaces. A cleanup Phase that cannot obtain required evidence becomes
`externally_blocked`; it does not silently retain a shim or permit automatic activation of the next Phase.

## Deployment And Compatibility

- Default rollout assumption: Core, game_server and in-workspace tests/data are validated from one repository state.
- Behavior-preserving server/Core refactors must prove exact old/new output and ordering before old duplication is
  removed.
- Accessible external Unity canonical/source is a consumer evidence surface, not authorization to change UX/wire.
- Unknown deployed Rust/Unity/standalone consumers remain explicit risk and, when required for removal, a completion
  blocker.
- No temporary compatibility is approved. If unavoidable, record supported versions/topology, duration, removal
  condition and tests, then obtain user approval before implementation.

## Focused Validation Policy

Every Phase starts with the exact first-proof row assigned in `EXACT_LINEAR_IMPLEMENTATION_ORDER.md` and the detailed
test family in `TEST_REBASELINE_CATALOG.md`. The test must execute a nonzero case count and directly observe the
contract—not only a private helper shape. Corrections require a defect-reproducing negative/red test where feasible;
refactors require characterization/golden parity; cleanup requires consumer inventory and assertion transfer first.

Each child records exact commands and runner-reported pass/fail/skip counts. It runs the smallest affected unit,
integration, live-RON, server mapping or real-file suite before broader package validation.

## Broad And Cross-Phase Validation

The active Phase selects the smallest sufficient affected package suite. In addition, the Master requires boundary
validation after Phases 15, 18, 34 and 44. Commands are revalidated against current Cargo metadata at execution time;
the intended minimum set is:

```text
cargo check -p game_core --all-targets
cargo check -p game_server --all-targets
cargo test -p game_core
cargo test -p game_server
cargo test -p game_core --test ron_loading -- --nocapture
```

Master final validation must also run the broadest safe current workspace all-target test/check supported by Phase
44's retained target/tool inventory. Exact executed test counts, zero-test targets, ignores/skips and any baseline
failures are recorded; compile-only success never proves runtime behavior.

Cross-Phase review specifically checks:

- F-C01 admission → P02/P03 mapping/mutation;
- P08 occurrence/stat/publication → committed damage → P09 projectile motion;
- P09 typed footprint → usefulness → union preview → P10 exact transport;
- P03 grant + P04 paired session → common HQ transaction;
- P07 retained Rapier/default backend → ghost movement field cleanup;
- public/debug/range/tool retention → final Cargo package cleanup.

## Omitted Validation

- Unity build/runtime and fixture synchronization are omitted by explicit user scope decision. The 2026-08-12
  read-only comparison confirmed the client is independently stale at wire/DTO/feature boundaries; this campaign
  neither changes nor claims compatibility with it. Core/game_server producer and mapping validation remains
  mandatory.
- Live server/WebSocket probes are omitted when typed producer, server mapping and actor-order tests directly close
  the same contract. A Phase must use the registered probe if message ordering or a real transport path cannot
  otherwise be proven.
- Performance benchmarks are omitted unless a candidate changes a hot movement/projectile/damage algorithm and an
  existing reproducible benchmark or regression threshold applies.
- Repository-wide formatting and unrelated lint cleanup are always omitted.

Every omission is reassessed by the active Phase and cannot hide a required completion condition.

## Phase Completion Audit Gate

After implementation and validation, each Phase creates `COMPLETION_AUDIT.md` and independently applies
`goal_completion_review_guide.md`. Required exit evidence includes:

- completion-condition evidence matrix with policy, runtime/data/contract and actual validation evidence;
- refreshed current authority path, producer/consumer, target authority and removal/consolidation result;
- focused and broad commands with actual test counts and baseline/new failure distinction;
- DTO/RON/save/consumer/compatibility disposition;
- legacy/fallback/placeholder search and scoped diff/dirty preservation;
- canonical alignment and cross-component seam delivered to the next Phase;
- primary verdict `Complete`, fit `High`, improvement class and explicit status transition.

Only `Complete / High` changes a Phase to `complete` and allows Master to make the next Phase `ready`. Medium/Low,
partial, document-only, policy/scope decision or external block prevents automatic activation. Correction requires
the same Phase audit to be rerun from the beginning.

## Cross-Phase Validation And Master Completion Audit

After all 44 Phases are complete, Master performs an independent final pass rather than trusting registry status.
It verifies exact 44-ID coverage, CRA-POL traceability, final source-of-truth ownership, absence of temporary
cross-Phase shims, final V3/RON/wire shape, live data, in-scope Core/game_server consumers, canonical alignment,
broad validation
and preservation of the initialization dirty state plus recorded concurrent changes.

The Master creates `COMPLETION_AUDIT.md` with the review-guide template and must receive `Complete / High` before
changing `Implementation Complete` to `true`.

## Master Completion Audit Evidence Matrix

| ID      | Completion condition or invariant                                                   | Required evidence                                                 | Current status |
| ------- | ----------------------------------------------------------------------------------- | ----------------------------------------------------------------- | -------------- |
| RBI-C01 | exact 44 open candidates implemented once in approved order                         | registry, child evidence and final catalog set comparison         | pass           |
| RBI-C02 | all Phase dependencies and one-active-Phase rule preserved                          | Master transition log and child completion audits                 | pass           |
| RBI-C03 | CRA-POL-001~006 fully implemented without implicit policy drift                     | policy→Phase→runtime/test/canonical traceability                  | pass           |
| RBI-C04 | 17 corrections close named defects atomically and directly                          | defect-focused tests plus current runtime/data evidence           | pass           |
| RBI-C05 | 17 refactors leave one authority with exact observable parity                       | characterization/goldens and removal audit                        | pass           |
| RBI-C06 | 10 cleanup items transfer assertions and satisfy consumer gates                     | consumer inventories, retained tests and legacy absence           | pass           |
| RBI-C07 | current V3, live RON and Core/server wire producer contract preserved                | exact shape/loading/server-mapping validation                     | pass           |
| RBI-C08 | no fallback, dual path, alias, placeholder or duplicate temporary authority remains | in-scope symbol/schema/fixture/canonical audit                    | pass           |
| RBI-C09 | required focused, boundary and final broad validation passes                        | exact commands and runner-reported counts                         | pass           |
| RBI-C10 | durable canonical matches the final implemented current contract                    | scoped canonical diff and runtime/data comparison                 | pass           |
| RBI-C11 | initial/concurrent user dirty work and unrelated artifacts preserved                | initialization baseline, per-Phase manifests and final diff audit | pass           |
| RBI-C12 | independent Master review is Complete / High                                        | Master `COMPLETION_AUDIT.md`                                      | pass           |

## Completion Conditions

This Master is complete only when:

- all 44 registry rows are `complete` with `Implementation Complete: true` and `Complete / High` child audits;
- no candidate is silently skipped, merged, split or declared solved by another Phase without current evidence and
  an explicit Master/user disposition;
- all policy coverage, migrations, consumer classifications and retained/removed legacy scopes are final;
- every required focused, boundary and broad validation has actual counts and passes or an approved baseline-failure
  disposition;
- final runtime/data/DTO/save/canonical and in-scope Core/game_server consumers describe one current contract;
- the independently stale Unity client is not modified, validated as compatible or counted as an in-scope consumer;
- Master final evidence matrix is all pass and the review-guide verdict is `Complete / High`.

## Stop Conditions

Stop the active Phase and do not activate the next when:

- a new gameplay, balance, UX, DTO/RON/V3 shape, compatibility or deployment policy is required;
- the candidate expands into an independent contract boundary or needs a different Master order;
- target files contain overlapping user work that cannot be safely distinguished or preserved;
- a required baseline cannot be obtained or runner output cannot prove the named contract;
- required external consumer/repository/tool evidence is unavailable;
- implementation would require fallback, dual schema/path, silent normalization, placeholder or future-Phase code;
- a current fact invalidates the candidate identity, target authority or dependency edge;
- a Phase audit is not `Complete / High`;
- production mutation has not been explicitly authorized.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
