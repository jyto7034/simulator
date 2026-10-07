# Boss Runtime Implementation Readiness Audit — Completion Audit

Audit Mode: `goal_exit`  
Audit Target: `boss_runtime_implementation_readiness_audit`  
Audit Date: 2026-08-08  
Goal Type: `single`, `investigation_only`  
Current Status at review start: `active`  
Implementation Complete Claim: `not_applicable`  
Policy Source: `docs/tmp/boss_behavior_state_machine_policy_notes_2026-08-07.ko.md` BP-01~BP-34  
Completion Procedure: `docs/goal_completion_review_guide.md` 1~567 EOF re-read  
Simulator Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`  
Unity Stale-Baseline Commit: `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`

## Audit target and verdict distinction

This review decides whether the **read-only readiness audit** completed its approved evidence/documentation work. It
does not claim that Boss Runtime is implemented.

- Goal completion verdict: `Complete / High`
- Implementation readiness verdict produced by the Goal: `conditionally_ready`
- Live Boss/public Wire publication verdict: `not_yet_ready_to_publish`

The latter two are findings, not reasons to leave this investigation Goal active. Their prerequisite/future owner is
fully dispositioned rather than unresolved.

## Completion Evidence Matrix

| ID | Completion condition | Policy/plan basis | Runtime/data/contract evidence | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- |
| BRA-C01 | Exact BP-01~BP-34 coverage | PLAN policy matrix rule | `POLICY_COVERAGE_MATRIX.md` has target/current/seam/disposition/owner/test columns for each policy. | Row count 34; expected/actual symmetric difference empty; duplicate output empty. | pass |
| BRA-C02 | End-to-end current path | PLAN investigation step 2 | `BOUNDARY_AUDIT.md` traces live source→preview/setup→scenario→BattleCore→leaf→event/checkpoint→server→Unity. | Source/symbol reads are recorded in 19 chronological `EXPERIMENTS.md` units. | pass |
| BRA-C03 | Current/target authority separation | BRA-I03~I09, BP-01~34 | Boundary authority table and matrix distinguish ordinary route/basic target/leaf owners from target Boss encounter/actor owners. | Competing-authority and preservation checks are explicit in boundary/report. | pass |
| BRA-C04 | Existing refactor intersection | PLAN named candidate list | `BOUNDARY_AUDIT.md` gives 19 named candidates one five-way classification. | 19 rows; classification totals 5 prerequisite, 2 coimplement, 4 defer, 8 independent; no duplicate gate. | pass |
| BRA-C05 | Minimum prerequisite slice | BP-16/BP-31/BP-34 | Readiness G0 limits prerequisites to CRA-P06-C01/C02/C03 and gives mutation/order/terminal acceptance. | Candidate source catalogs were read and current source counterexamples recorded. | pass |
| BRA-C06 | Minimum tracking vertical slice | Approved implementation direction | Readiness G1/G2 fixes strict fixture RON→typed binding→runtime→target/movement→existing event/checkpoint path and exclusions. | Test-first tables cover success, domain failure, timeout, lifecycle interrupt, reconnect and non-opt-in parity. | pass |
| BRA-C07 | Final-boss stress without scope expansion | BRA-I11 | Boundary/report map WhiteNight, Silent Orchestra and Apocalypse Bird to generic extension points and separate future owners. | Design-note source ranges and comparison results are recorded; no concept content is promoted to v1. | pass |
| BRA-C08 | Required Core/server/Unity consumer checked | Evidence precedence and in-scope list | Current Core/game_server source and stale Unity parser/store/presenter/tests were read. Concurrent Spine work was re-audited and remains wire-independent. | Consumer paths, reverse-reference empty result and timestamp/status drift are recorded. No required repository is inaccessible. | pass |
| BRA-C09 | Policy questions resolved or gated | PLAN Open Policy Decisions | Unresolved ledger closes every initially unverified item. Repath/hot reload/public presentation/final content have strict future owners. | Search found only historical unresolved text linked to the closure ledger. | pass |
| BRA-C10 | Existing/new test safety net disposition | PLAN verification step 5 | Boundary and readiness classify retain/strengthen/merge/remove/unverified families and enumerate future failing acceptance. | No target test is falsely claimed green; runtime runner omission is explicitly justified. | pass |
| BRA-C11 | No production mutation and dirty preservation | BRA-I01/I02 | Simulator Goal-excluded status is identical to baseline. Unity drift is a timestamped concurrent Spine campaign; re-audited, preserved, and not reverted. | Simulator 211 entries and initial SHA exact; Goal status lists only seven Markdown artifacts after this file. Unity read-only status/timestamp/reverse-ref evidence recorded. | pass |
| BRA-C12 | Independent completion review | Completion guide | This document separately rechecks artifacts, source-of-truth, consumers, compatibility, dirty state, validation and fit. | Primary verdict and status mapping below are explicit. | pass |
| BRA-C13 | Full rulebook read and drift audit | PLAN explicit completion gate | `game_rulebook.md` 1~961 EOF plus three adjacent canonicals are compared to runtime/data/target in boundary tables. | Exact read ranges/commands are in `EXPERIMENTS.md`; current 21-versus-22 source and Unity-link drift are dispositioned. | pass |
| BRA-C14 | Incremental durable evidence | BRA-I14 and durable protocol | Findings were appended per source/consumer/policy/refactor/stress/report unit before final synthesis; unresolved ledger points forward without deleting history. | 19 dated experiment units and 16 durable/closure checkpoints exist before this completion document; no final claim relies on conversation-only evidence. | pass |

## Invariant Review

| ID | Invariant | Evidence | Verdict |
| --- | --- | --- | --- |
| BRA-I01 | No production code/data/test/canonical/server/Unity mutation | Only Goal Markdown files appear in Simulator Goal path; Goal-excluded Simulator hash is unchanged; Unity commands were reads. | pass |
| BRA-I02 | Preserve user dirty/untracked state | No restore/reset/clean/stash/formatter/destructive command ran. Concurrent Unity Spine artifacts remain present. | pass |
| BRA-I03 | Preserve ordinary route behavior | Current authority and paired non-opt-in parity acceptance are fixed in boundary, BP-01 and G2. | pass |
| BRA-I04 | Core owns gameplay; Unity does not recalculate | BP-18/BP-22/BP-30 and G1/G3b keep graph/target/RNG/order Core-only. | pass |
| BRA-I05 | Reuse leaf subsystems | Boundary owner table and report require intents/adapters over existing movement/block/attack/skill/effect/death. | pass |
| BRA-I06 | Audit time/order/epoch/checkpoint together | Scheduler, terminal, active runtime/checkpoint and reconnect sections cross-link these concerns. | pass |
| BRA-I07 | Failure leaves no partial state | Current violation is evidenced and P06-C03/G0/BP-31 acceptance gates Boss integration. It is not hidden as ready. | pass |
| BRA-I08 | Invalid/unimplemented RON fails closed | BP-02/BP-06/BP-28/BP-29 rows and strict data tests reject unknown/future/dangling/partial definitions. | pass |
| BRA-I09 | Separate internal graph from presentation identity | BP-30/G3b keep internal IDs private and require stable semantic presentation instance only when public. | pass |
| BRA-I10 | Refactor priority is not implementation authority | All 19 candidates are source-correlated and only eight named dependencies appear in readiness sequencing. | pass |
| BRA-I11 | Final-boss concepts are stress inputs | Three-concept matrices list extension/future owners and reject placeholders/universal VM. | pass |
| BRA-I12 | Independent defects are recorded, not auto-fixed | Canonical/source-count, world atomicity, server amplification and Unity debt are dispositioned without production edits or new Goals. | pass |
| BRA-I13 | Unity is stale consumer evidence | Every Unity conclusion is producer-relative; concurrent Spine primitive is rechecked but not elevated to current Core canonical. | pass |
| BRA-I14 | Conversation is not evidence storage | Chronological experiments/notes, exact commands/results, matrix/report and unresolved ledger form the durable chain. | pass |

## Runtime and state evidence

The audit directly confirmed the current runtime rather than inferring it from the target design:

- `BattleCore` has usable battle-time scheduling and leaf action/movement delivery but no Boss catalog, opt-in,
  encounter runtime, actor runtime, graph, Task owner, target lock or movement intent.
- `BattleEvent::Ord` is not total for all unequal variants; recursive same-time draining has no microstep limit;
  direct/until advance disagree at max time.
- World live advance mutates execution before fallible trigger/reconcile/projection/revision/cursor work. A future
  `after_seq` can fail after mutation, so current no-mutation canonical is not satisfied.
- Existing hard-CC/action epochs and detached projectile/delivery behavior match the cancellation primitive needed by
  BP-09, but a State activation lease is missing.
- Target selection, route progress and movement stop causes remain leaf/basic authorities. The exact missing Boss seams
  are activation target lock, typed lexicographic query/path cost, monotonic route cursor, intent admission and closed
  result delivery.

These are readiness findings and prerequisites, not implementation completion claims.

## Data evidence

- Current `GameplayRonSourceSet`/parsed/DB-parts inventory has 22 sources outside the manifest, while the adjacent data
  canonical says 21. The drift is recorded and Boss source must update one exact inventory authority.
- Startup load is strict parse→cross-validation→candidate DB→preview validation→single `Arc<GameDataBase>` return.
  That is sufficient for a v1 atomic startup candidate; it has no dynamic publisher or LKG generation owner.
- `PveEncounter`, setup/scenario and `RuntimeUnit` have no Boss reference/binding/runtime today. Preview validation is
  floor-0/five-seed baseline only, not full manifested difficulty/setup feasibility.
- Readiness separates empty/foundation catalog integration, test fixture opt-in and later official live content
  publication so a fixture-only path cannot be mistaken for live readiness.

## DTO, transport and persistence evidence

- Core already owns append-only seq/time events, checkpoint projection and revision. The internal tracking slice can
  use existing movement/damage/death variants without exposing Boss graph state.
- `BattleResync` retains the typed Core update, while normal update/setup manually re-enumerate top-level fields.
  Existing selected-field tests do not prove normalized Core/server parity; P10-C03 is therefore tied to first public
  Boss Wire change rather than the internal slice.
- Active battle is in-memory `ActiveBattleSession`; public Unity checkpoint is a reconcile projection, not a Core
  restore format, and active battle is absent from durable run SavePoint payload.
- Future public duration presentation needs stable lifecycle identity/cause, active checkpoint state and synchronized
  server/Unity consumers. No unauthorized dual schema or compatibility fallback is recommended.

## Consumer evidence

| Consumer class | Result |
| --- | --- |
| Current workspace producer/transport | Core and game_server source directly checked. |
| Canonical external consumer | Unity project was accessible and directly checked as a stale baseline, not only through docs. |
| Concurrently changed consumer surface | New `ArkClient.SpinePresentation` was re-read; it is caller-driven/sandbox-only, wire-independent and not referenced by battlefield/runtime. |
| Inaccessible known consumer | None required for this verdict. |
| Unknown deployment | No evidence of another required deployed Rust/Unity consumer; not a blocker for this investigation-only result. |
| Explicitly excluded | Unity PlayMode visual execution/assets, unrelated external consumer deployment and final-boss content implementation; reasons remain valid. |

## Legacy and compatibility audit

No production schema or implementation was introduced, so there is no Goal-created legacy/dual-read/write surface to
remove. The design audit nevertheless searched the future boundary and explicitly rejects:

- silent fallback for unimplemented movement disposition, selector, Task/action, condition or query variants;
- deep RON override, last-wins duplicate ID and partial catalog generation;
- using public checkpoint to reconstruct internal gameplay state;
- treating unknown Unity event parse tolerance as presentation support;
- exposing internal graph IDs or keeping an implicit route fallback for opted-in behavior;
- placeholder final-boss mechanics and an unrestricted script/expression VM.

Current unrelated legacy/debt is preserved and dispositioned rather than corrected inside this Goal.

## Scoped diff and dirty preservation

### Simulator/Core/game_server

- Initial Goal-excluded production scope: 211 status entries, SHA-256
  `7358c0145d11bbc1799400d20429174996a1a20ad12461e19ce761a983565cfe`.
- Completion precheck: exactly the same 211 entries and SHA-256.
- Goal path contains only `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, `BOUNDARY_AUDIT.md`,
  `POLICY_COVERAGE_MATRIX.md`, `IMPLEMENTATION_READINESS_REPORT.md` and this `COMPLETION_AUDIT.md`.
- No production diff is attributable to this Goal.

### Unity

- Initial full/supporting status: 2059/306 entries with hashes
  `55d940dc476ee297b267ed15afffd40570d448289cdccb0f172957bf254c1dcf` /
  `50ab153889caa2437bed4d0518dd67ac50c5b190dde08b7460ff6fcbedaaff14`.
- Precompletion full/supporting status: 2108/350 with hashes
  `8803df459017a04f9c627a17c5e0f0af01695adc6d3eb4f09c04f2fed4715876` /
  `77ab49c4d3ca0fdf178e6ccc3b1314e1dfe7fd5188144d050e086d236cd755b3`.
- Timestamp scan finds 22 post-start files concentrated in the concurrently active Spine presentation campaign,
  written 22:49~23:13 after this Goal's 22:42 start. The Boss audit's exact commands contain no Unity write, runner,
  asset builder or import command.
- The changed area was re-audited because it could affect a finding. Nothing was reverted, moved or reformatted.

The external hash difference is therefore preserved concurrent user/environment work, not a hidden claim of identical
Unity state. It does not fail dirty preservation.

## Canonical alignment

- `game_rulebook.md` was read 1~961 EOF; `core_runtime_contract.md` 775 lines,
  `data_loading_contract.md` 305 lines and `skill_target_contract.md` 373 lines were also read in full.
- Current rulebook gameplay meaning materially matches existing route/block/time/authority behavior. Boss target
  policy is a future opt-in extension and has not been falsely written into current canonical.
- Recorded current drift: data canonical source count 21 versus code 22; runtime no-mutation promise versus live
  advance partial commit; rulebook's external Unity canonical link versus a stale consumer.
- Because production behavior did not change, this Goal correctly did not mutate canonical. The future implementation
  Goal must update the closest canonical together with actual behavior/data/wire changes.

## Validation results

### Focused audit validation

- Policy matrix: 34 rows; duplicate list empty; expected `BP-01`~`BP-34` symmetric difference empty.
- Primary policy dispositions: 28 new foundation, 4 refactor coordination, 2 existing-primitive correction.
- Refactor intersection: 19 rows; 5 prerequisite, 2 coimplement, 4 defer and 8 independent.
- Artifact structure: six prerequisite artifacts were non-empty before this audit; this file supplies the seventh.
- Durable chronology: 19 dated experiment units and 16 durable/closure checkpoints before completion review.
- Simulator production scope: 211 entries and baseline SHA exact.
- Source commits remain the frozen Simulator and Unity start commits.

Exact commands and actual outputs are in `EXPERIMENTS.md`, including the completion-precheck command family.

### Runtime test result

No Cargo, server, Unity EditMode or PlayMode runner was executed, so no pass/fail/skip count is reported. This omission
is approved by PLAN and remains valid:

- the Goal made no production change for a before/after regression suite;
- Boss target behavior has no implementation or executable target tests;
- broad dirty-tree runners would not prove policy readiness and could mix unrelated failures;
- source/test-body inspection closed every fact needed for the verdict;
- the concurrent Unity Spine tests belong to a different active campaign and are not claimed here.

The future implementation report names the exact failing characterization/contract tests required before mutation.

### Broad validation

Omitted for the same investigation-only reason. There is no package/runtime behavior change whose broad green result
could validate this Goal. Artifact integrity, source traceability and dirty preservation are the smallest sufficient
validation set.

## Baseline failures

No runner was executed and no baseline test failure is claimed or reclassified. The only changing external baseline is
the concurrent Unity worktree state documented above; it is not presented as a test result.

## Cross-component findings

1. Boss lifecycle integration must wait for Core terminal parity, total same-time order and rollback-safe world
   advance. These are shared correctness primitives, not Boss-owned duplicates.
2. Existing-event tracking can remain Core/server/Unity Wire-neutral. The first new public lifecycle is a distinct
   semantic DTO + server exact-parity + Unity lifecycle/reset cutover.
3. Unity's new semantic Spine leaf abstraction improves a possible future presentation adapter but does not consume
   battle events or own seq/checkpoint lifecycle; it does not change the gate.
4. Official live opt-in is later than isolated foundation/fixture work because current publication validation does not
   cover every manifested difficulty and fully resolved Boss setup binding.

## Long-term direction review

| Candidate group | Work type | Current→target authority result | Preservation/result | Verification | Verdict |
| --- | --- | --- | --- | --- | --- |
| P06-C01/C02/C03 | correctness | Existing terminal/order/advance owners corrected before Boss integration. | Avoids parallel Boss finalization/order/transaction. | Source counterexamples + G0 acceptance. | pass |
| P01-C01, P05-C04 | coimplementation | One source inventory and immutable typed setup owner. | No duplicate catalog lookup/binding authority. | Data/setup trace + G1 strict tests. | pass |
| P01-C02, P05-C02 | live publication gates | Full manifestation/setup feasibility before official opt-in. | Fixture work does not masquerade as live readiness. | G3a boundary. | pass |
| P07-C03/C04, P09-C05, P10-C03 | deferred | Cleanup/wire changes wait for observed Boss seam or actual public contract. | Prevents premature removal or needless amplification. | 19-row boundary classification. | pass |
| Remaining eight named candidates | independent | Leaf/tactical/projection owners remain separate. | Whole refactor catalog is not a Boss prerequisite. | Candidate source correlation. | pass |

- Fit: `High`
- Improvement class: `none` for this investigation Goal
- Reason: ownership is separated by authoring, Core orchestration, existing leaf execution, transport and presentation;
  dependency gates are minimal; future mechanics are not prebuilt; every current defect/gap has a concrete owner and
  acceptance without expanding implementation authority.

## Primary verdict

`Complete`

All 14 completion conditions and 14 invariants pass. No material policy, evidence, consumer-access, compatibility or
scope finding is hidden. The output's `conditionally_ready` implementation verdict is fully actionable and bounded.

## Status transition

- from: `active`
- to: `complete`
- `Implementation Complete`: remains `not_applicable`
- next implementation activation: denied by this audit alone; requires a separately approved G0 Goal

## Remaining risk and required action

These are explicit follow-up gates, not incomplete audit work:

1. Separately authorize and complete CRA-P06-C01/C02/C03 with the G0 acceptance suite.
2. Then authorize G1 foundation + G2 test-fixture slice, still without official live content/public Wire.
3. Gate official live Boss opt-in on P01-C02/P05-C02 and content/canonical approval.
4. Gate the first public lifecycle on Core semantics, P10-C03/server parity and synchronized Unity lifecycle/reset
   integration.
5. Keep final-boss mechanics and their navigation/VN/visibility/deployable/presentation systems in separately approved
   future Goals.

Open policy questions: none.
