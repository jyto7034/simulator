# Boss Runtime v1 Implementation Master Goal

Status: complete
Implementation Complete: true
Goal Type: master
Execution Mode: orchestration_only
Production Mutation Authorized: no — production mutation belongs only to an explicitly activated child Phase Goal
Program Direction Approved: 2026-08-09
Simulator Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
Active Phase: none
Next Phase Candidate: none
Pending Policy Decision: none; P06 is explicitly `not_applicable_to_v1` from audited P05 evidence
Goal Name Authority: agent-assigned from the approved program scope and repository naming convention

## Objective

Boss Runtime v1을 main track으로 구현하고, 기존 Core 리팩터링 후보를 prerequisite, coimplementation,
independent sub-track, post-slice cleanup과 first-public-wire gate로 조정한다.

이 Master는 Phase 순서, 공통 불변식, dependency, activation gate와 최종 cross-Phase 완료 감사를 소유한다.
실제 production 조사·구현·검증은 한 번에 명시적으로 활성화된 child Phase Goal이 소유한다.

## User/Contract-Visible Problem

현재 Core의 ordinary DefenseRoute enemy는 authored route, block, basic attack과 기존 leaf subsystem으로
동작하지만 Phase/Pattern/State/Task 기반 Boss orchestration, Boss 전용 RON catalog와 encounter/actor binding은
없다. 승인된 Boss 정책을 한 patch로 삽입하면 다음 현재 correctness와 migration 위험이 섞인다.

- Same-time `BattleEvent` order가 모든 unequal event에 대해 total order가 아니다.
- World live advance가 후처리 실패 전에 Core/revision을 일부 변경할 수 있다.
- Advance API별 terminal/max-time 의미가 다르다.
- Authored route/basic target/leaf action authority와 새 Boss target/movement/task owner 사이에 명시적 seam이
  없다.
- Live RON publication, setup binding, Core checkpoint, server field mapping과 stale Unity presentation의
  수명주기가 서로 다르다.
- 전체 refactor catalog를 선행시키면 Boss와 무관한 변경이 gate가 되고, 실제 Boss seam을 보기 전 cleanup이
  목표 ownership을 잘못 고정할 수 있다.

사용자-visible 최종 결과는 ordinary enemy behavior를 보존하면서 실제 live reference Boss가 opt-in할 수 있는
production-ready Boss Runtime v1이다. 리팩터링은 이 결과를 안전하게 만드는 dependency-classified sub-track이며
독립적인 전체 완료 목표로 Boss main track을 막지 않는다.

## Audit Finding ID

없음. 이 Master는 신규 결함 감사가 아니라 완료된 readiness audit과 사용자가 승인한 프로그램 방향을 실행
단위로 조정한다. CRA ID는 기존 refactor audit candidate이며 구현 권한이 아니다.

## Approved Program Direction

- 최종 목적은 `BP-01`~`BP-34`를 만족하는 production-ready Boss Runtime v1과 최초 live reference Boss다.
- Boss Runtime이 main track이다.
- 리팩터링은 전체 선행 campaign이 아니라 dependency가 확인된 sub-track이다.
- 백야, 고요한 오케스트라와 종말새의 전체 고유 콘텐츠는 별도 future content Goals가 소유한다.
- P00 preflight가 current tree, active-work collision과 Phase boundary를 확인하기 전 production Phase를
  활성화하지 않는다.

## Required Evidence Inputs

- `../../README.md`
- `../../engineering_workflow.md`
- `../../goal_authoring_and_execution_guide.md`
- `../../goal_completion_review_guide.md`
- `../../game_rulebook.md`
- `../../core_runtime_contract.md`
- `../../data_loading_contract.md`
- `../../skill_target_contract.md`
- `../boss_runtime_implementation_readiness_audit/PLAN.md`
- `../boss_runtime_implementation_readiness_audit/BOUNDARY_AUDIT.md`
- `../boss_runtime_implementation_readiness_audit/POLICY_COVERAGE_MATRIX.md`
- `../boss_runtime_implementation_readiness_audit/IMPLEMENTATION_READINESS_REPORT.md`
- `../boss_runtime_implementation_readiness_audit/COMPLETION_AUDIT.md`
- `../../tmp/boss_behavior_state_machine_policy_notes_2026-08-07.ko.md`의 `BP-01`~`BP-34`
- `../core_enhanced_refactoring_audit_master/PLAN.md`
- `../core_enhanced_refactoring_audit_final_catalog_consolidation/FINAL_CANDIDATE_CATALOG.md`

위 readiness audit은 `Complete / High`, implementation readiness는 `conditionally_ready`다. P00은 이를
처음부터 재감사하지 않고 current-tree material drift와 실행 충돌만 확인한다.

각 Phase 시작·재개 시에는 위 공통 workflow/canonical 문서의 current version, Master registry, 해당 child의
필수 문서 3개를 production 조사보다 먼저 읽는다. Wire 또는 presentation 경계를 여는 Phase는 current
game_server canonical과 실제 접근 가능한 Unity source/canonical도 먼저 확인한다. 과거 audit 결론은 current
source 또는 canonical과 충돌할 때 우선하지 않는다.

## Required Start And Resume Procedure

1. Master의 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`를 끝까지 읽고 active Phase가 없거나 재개 대상과
   일치하는지 확인한다.
2. 위 workflow/completion guide와 변경 주제의 current canonical을 읽고, 이전 audit 이후 material 문서 변경을
   해당 child의 discovery input으로 기록한다.
3. Master registry의 prerequisite/completion verdict, child 경로 충돌과 current dirty 상태를 확인한다.
4. 신규 child면 필수 문서 3개를 먼저 만들고, 기존 child면 세 문서를 끝까지 읽은 뒤 마지막 evidence부터
   재개한다. 그 전에는 production 검색·수정·test/formatter를 실행하지 않는다.
5. 활성화·조사·구현·검증·완료 감사 결과를 bounded unit 직후 child 문서에 기록하고 Phase transition은 같은
   시점에 Master에도 반영한다.

## Current Baseline Evidence At Master Creation

다음은 completed readiness audit의 final evidence를 Master 입력으로 고정한 것이며 P00의 current-tree
검증을 대체하지 않는다.

- Current live bundle은 manifest 외 22개 source를 수동 inventory하고 strict startup candidate를 한 번에
  publish하지만 Boss catalog, generation/hot reload와 LKG publisher는 없다.
- `PveEncounter`, combat preview/setup, `BattleScenario`와 `ScenarioUnitSpawn`에는 Boss encounter/actor binding이
  없다. Official live publication 전 manifested difficulty와 full setup feasibility 검증이 부족하다.
- Ordinary enemy authored route는 setup에서 `EnemyMovementPlan::PathAlongCells`로 resolve되어 runtime unit에
  전달된다. Boss opt-in이 없을 때 이 authority를 유지해야 한다.
- Current movement planner에는 Boss intent seam, activation-owned target lock, deterministic path-cost query와
  monotonic forward route cursor가 없다. Existing movement/block/attack/skill/effect/projectile/death는 leaf로
  재사용 가능하다.
- `BattleEvent::Ord`의 catch-all equality, unbounded same-time drain, max-time API divergence와 fallible
  post-step world projection은 P01 correctness gate다.
- Core seq/time event delta, checkpoint와 revision은 reusable하다. Normal server update/setup은 top-level field를
  수동 재열거하여 최초 public Boss contract 때 exact parity가 필요하다.
- Unity는 current Core/game_server보다 stale한 consumer baseline이다. Concurrently added
  `ArkClient.SpinePresentation`은 wire-independent leaf primitive일 뿐 battle event/checkpoint adapter가 아니다.
- Current active battle is Core session memory; public Unity checkpoint is a reconcile projection, not a durable Core
  restore format.

## Source Of Truth And Ownership

### Evidence precedence

1. 활성 Phase 시작 시 current runtime code와 authoritative mutation path
2. production live RON/data와 loader/validation
3. actual Core DTO, game_server mapping과 접근 가능한 current Unity consumer
4. current durable canonical
5. BP-01~BP-34 target decision record
6. this Master, child Goal과 completed audit evidence

### Target ownership

| Concern | Target authority | Downstream/leaf owner |
| --- | --- | --- |
| Boss authoring/publication | separate Boss domain catalog in one atomic live bundle | setup compiler and strict validation |
| Encounter phase/objective/mechanic orchestration | `BossEncounterRuntime` | existing wave/objective/mechanic adapters |
| Per-actor phase/pattern/state/task/target/intent | `BossActorBehaviorRuntime` | existing movement/combat leaf subsystems |
| Spatial movement/collision/block | existing BattleCore spatial authority | Core events/checkpoint projection |
| Attack/skill/effect/projectile/damage/death | existing typed leaf modules | Boss owner lease consumes typed result only |
| Same-time order and atomic state mutation | BattleCore/world Core authority | server transports committed result only |
| Public semantic event/checkpoint | Core DTO/event log | game_server envelope, Unity replay/presentation |
| Internal active battle state | Core session runtime | never reconstructed by Unity |
| Durable current contract | closest canonical after implementing Phase | Goal docs remain historical evidence |

Current canonical inputs are `game_rulebook.md`, `core_runtime_contract.md`, `data_loading_contract.md` and
`skill_target_contract.md`. Unity-facing work must use the current external documents and actual source under
`/mnt/f/unity projects/ark`; stale consumer text never overrides the Core producer.

## Policy ID To Phase Coverage

한 정책이 definition과 runtime 또는 internal/public boundary를 함께 가질 때 여러 Phase가 같은 ID를 공유한다.
각 Phase는 자기 계약만 완료하며 P07이 최종 production coverage를 정확히 한 행씩 재감사한다.

| Policy IDs | Definition/foundation owner | Runtime/vertical owner | Live/public/final owner |
| --- | --- | --- | --- |
| BP-01 | P02 strict optional catalog/setup binding | P04 non-opt-in parity | P05 official opt-in |
| BP-02~BP-03 | P02 closed supported disposition/result schema | P03 path/intent/result seam; P04 tracking/rejoin proof | P05 live content usage |
| BP-04~BP-05 | P02 resolved encounter/actor definition and setup identity | P03 two-layer runtime; P04 lifecycle proof | P05 live actor binding |
| BP-06~BP-15 | P02 selector/graph/outcome/control validation | P03 lifecycle/scheduler implementation; P04 supported reference subset | P05 only implemented vocabulary may publish |
| BP-16~BP-17 | P01 total-order substrate; P02 priority uniqueness | P03 cross-scope/node arbitration; P04 permutation proof | P07 final combined audit |
| BP-18~BP-21 | P02 typed target query schema | P03 lock/invalidation/path-cost implementation; P04 target slice | P05 live authored query |
| BP-22~BP-27 | P02 condition/event-memory/result/timeout/cooldown schema | P03 Core battle-time runtime; P04 required reference subset | P05 strict live usage |
| BP-28~BP-29 | P02 separate catalog, atomic candidate and typed parameter binding | P03 immutable captured frame | P05 full feasibility and official publication |
| BP-30 | P02/P03 keep graph state internal; P04 proves existing-Wire slice | P04 event/checkpoint reconnect proof | P06 only if public lifecycle is required; P07 validates final disposition |
| BP-31 | P01 world atomicity substrate | P03 transition stage/validate/commit; P04 failure proof | P07 final combined audit |
| BP-32~BP-34 | P01 total order/atomic substrate; P02 static validation | P03 candidate arbitration/activation/microstep; P04 lifecycle proof | P07 final combined audit |

## Master Boundary

### In scope

- Phase registry, dependency와 activation 상태
- BP-01~BP-34의 Phase coverage
- Boss main track과 refactor sub-track disposition
- Phase 사이 typed/data/runtime/wire seam
- 공통 migration, rollout과 consumer gate
- Phase별 독립 완료 감사와 Master 최종 cross-Phase 감사

### Out of scope for this Master document

- production code/data/test/canonical/server/Unity mutation
- child Phase의 상세 구현 일지
- 승인되지 않은 gameplay, schema, save 또는 compatibility 정책
- 모든 리팩터 후보의 선행 구현
- 최종보스 3종의 전체 콘텐츠, 밸런스, art/VFX/animation

## Implementation Scope

이 Master 자체의 구현 범위는 Phase registry, dependency, policy coverage, activation/termination 기록과 최종
cross-Phase 감사뿐이다. Production 구현은 활성 child 하나가 자신의 승인 범위에서 수행한다. 예상되는
production 경계는 P01 correctness substrate, P02 authored data/setup, P03 Core runtime, P04 fixture vertical
slice, P05 live content, 조건부 P06 public producer-consumer cutover다. P07은 수정 권한 없는 독립 감사가
기본이며 material correction이 필요하면 해당 구현 Phase를 다시 열거나 별도 승인된 correction Goal을 쓴다.

P00은 조사 문서만 작성한다. P00 완료 전에는 위 production 경계 중 어느 것도 구현 범위로 활성화되지
않는다.

## Durable Evidence Rule

대화와 model context는 실행 evidence 저장소가 아니다. Master initialization, Phase activation/transition,
cross-Phase finding과 최종 검증은 발생 시점에 `EXPERIMENTS.md`와 `EXPERIMENT_NOTES.md`에 기록한다.
P00와 각 child Phase도 bounded investigation unit 직후 자체 Goal artifact에 evidence를 누적한다.

## Preliminary Dependency Model

```text
P00 current-tree execution preflight
  -> P01 G0 battle lifecycle correctness
      -> P02 Boss catalog and typed setup foundation
          -> P03 Boss runtime kernel and target/movement seam
              -> P04 TrackAndRejoin fixture vertical slice
                  -> P05 official live reference Boss
                      -> P06 public Wire/server/Unity integration when required
                          -> P07 Master final completion audit

Independent refactor sub-track
  -> only when P00 proves non-overlap and an independently approved Goal owns it
```

이 구조는 P00의 current-tree evidence로 확정하기 전 preliminary registry다. 승인된 최종 결과를 축소하지
않는 범위에서 atomic change boundary에 따라 인접 Phase를 병합하거나 분리할 수 있다.

## Preliminary Refactor Sub-track Registry

| Timing | Candidate IDs | Preliminary relation |
| --- | --- | --- |
| Before Boss runtime integration | CRA-P06-C01, CRA-P06-C02, CRA-P06-C03 | P01 correctness gate |
| With Boss foundation | CRA-P01-C01, CRA-P05-C04 | P02 coimplementation |
| Before official live opt-in | CRA-P01-C02, CRA-P05-C02 | P05 publication/setup gate |
| At first public Boss Wire change | CRA-P10-C03 | P06 Core/server parity gate |
| After vertical slice | CRA-P07-C03, CRA-P07-C04, CRA-P09-C05 | cleanup after actual seam is proven |
| Independent | CRA-P07-C01, CRA-P07-C02, CRA-P08-C01, CRA-P08-C05, CRA-P09-C03, CRA-P09-C06, CRA-P10-C02, CRA-P10-C04 | separate Goals only; not Boss completion gates |

## Preliminary Phase Registry

| Phase | Child Goal name | Coordination status | Hard prerequisite | Planned output |
| --- | --- | --- | --- | --- |
| P00 | `boss_runtime_v1_implementation_master_phase_00_execution_preflight` | complete — 2026-08-09, `Complete / High`, `ready_for_g0` | readiness audit Complete/High | current drift/collision audit, final registry and G0 handoff |
| P01 | `boss_runtime_v1_implementation_master_phase_01_battle_lifecycle_correctness` | complete — 2026-08-09, `Complete / High`, `ready_for_p02` | P00 Complete/High + `ready_for_g0` | terminal parity, total event order, advance atomicity |
| P02 | `boss_runtime_v1_implementation_master_phase_02_catalog_and_typed_setup` | complete — 2026-08-09, `Complete / High`, `ready_for_p03` | P01 Complete/High | strict Boss catalog, compiled definitions, typed setup binding |
| P03 | `boss_runtime_v1_implementation_master_phase_03_runtime_kernel_and_target_movement` | complete — 2026-08-09, `Complete / High`, `ready_for_p04` | P02 Complete/High | encounter/actor runtime, lifecycle/scheduler, target/movement seam |
| P04 | `boss_runtime_v1_implementation_master_phase_04_track_and_rejoin_vertical_slice` | complete — 2026-08-09, `Complete / High`, `ready_for_p05_decision` | P03 Complete/High | existing-event `TrackAndRejoin` fixture vertical slice |
| P05 | `boss_runtime_v1_implementation_master_phase_05_live_reference_boss` | complete — 2026-08-09, `Complete / High`, `ready_for_p07_final_audit` | P04 Complete/High; reference identity approval; live publication gates | first approved live reference Boss and canonical/data completion |
| P06 | `boss_runtime_v1_implementation_master_phase_06_public_lifecycle_cutover` | `not_applicable_to_v1` — 2026-08-09; existing event/checkpoint path proven sufficient | actual public lifecycle requirement was not present | no production child; future public lifecycle requires a new decision/contract |
| P07 | `boss_runtime_v1_implementation_master_phase_07_completion_audit` | complete — 2026-08-09, `Complete / High` | P00~P05 Complete/High; P06 explicitly dispositioned | independent cross-Phase Master completion audit |

Child directories are not created until their Phase is explicitly activated. P00 must validate whether P02/P03 and
P05/P06 are the correct atomic boundaries; this preliminary table is not implementation authorization.

## Phase Boundary Drafts

P00 performs the first refinement of child names and adjacent boundaries when current-tree ownership, atomic deployment
or validation evidence requires it. Later implementation and audit findings may also trigger controlled replanning
under `Plan Change Control`. No refinement may remove policy coverage, turn independent refactors into Boss gates or
implicitly activate production work.

### P00 — Execution preflight

Execution mode: `investigation_only`

Scope:

- Revalidate only material drift since the readiness audit in scheduler/terminal/advance, data/setup, movement/target,
  server DTO and relevant Unity consumer paths.
- Inventory active/incomplete Goals and dirty file/authority collisions.
- Convert all 19 relevant CRA candidates into an execution registry with owner, overlap, entry/exit and timing.
- Confirm the Phase split and produce a ready-to-use P01 G0 implementation handoff.
- Define the evidence and approval gate that will decide whether the eventual first live reference Boss requires P06;
  do not invent the reference identity or decide its presentation contract in P00.

Exit gate:

- Current-tree drift is either nonmaterial or fully reflected in the registry/handoff.
- No unresolved active-work collision makes P01 ownership ambiguous.
- P01 source/test owner, internal milestone order, failure injection surface and validation commands are concrete.
- P05 reference-content decision and P06 public-contract decision have explicit owners, evidence and latest decision
  points without blocking P01.
- P00 completion audit returns `Complete / High` and its preflight verdict is `ready_for_g0`.

P00 does not mutate production or create later child directories.

### P01 — G0 battle lifecycle correctness

Execution mode: `implement_and_verify`

Candidate scope:

- CRA-P06-C01: one terminal/max-time authority across all advance entrypoints.
- CRA-P06-C02: total deterministic `BattleEvent` order for every unequal variant/payload.
- CRA-P06-C03: failed advance leaves Core/world/resource/event/revision/cursor state unchanged.

Starting implementation order is C01 → C02 → C03 so the atomic shell is built around final terminal/order semantics.
P00 must verify whether one child with three reviewable milestones is safer than multiple child Goals in the current
tree. Boss schema/runtime/Unity work is prohibited in P01.

Exit gate:

- Direct, until-time, next-event and server-used advance paths have exact terminal parity.
- All relevant event insertion permutations drain identically and unequal events never compare equal.
- Injected VN/reconcile/projection/revision/cursor failures prove full no-mutation.
- Existing finalized, attack-before-end, Pause and cursor regression invariants remain protected.
- Focused and smallest sufficient broad Core/server validation pass with actual counts.
- Canonical describes the corrected current runtime and completion audit returns `Complete / High`.

### P02 — Boss catalog, compiled definition and typed setup foundation

Execution mode: `implement_and_verify`

Scope:

- Separate logical Boss catalog collections for Pattern, actor behavior and encounter definitions.
- Strict source inventory, parsing, duplicate/ref/variant/outcome/result/capability/loop/parameter validation.
- Immutable compiled definitions and typed argument frames in the atomic candidate DB.
- Optional `PveEncounter` reference and typed actor binding resolved before battle start into setup/scenario.
- CRA-P01-C01 and CRA-P05-C04 coimplementation.

This Phase may add an empty or test-fixture-capable catalog, but it does not opt an official live encounter into Boss
behavior. Dynamic hot reload/LKG, generic scripting, deep override and future placeholder variants are excluded.

Exit gate:

- Invalid or unsupported definitions reject the entire candidate without fallback or partial publication.
- No-binding production setup is semantically unchanged.
- Resolved IDs/frames enter an immutable typed battle setup without tick-time raw catalog lookup.
- Exact source inventory and relevant strict loader/setup tests pass.
- Data/runtime canonical is updated only for behavior actually implemented and completion audit is `Complete / High`.

### P03 — Boss runtime kernel and target/movement seam

Execution mode: `implement_and_verify`

Scope:

- Core-owned `BossEncounterRuntime` and per-actor `BossActorBehaviorRuntime`.
- Phase/Pattern/State/Task activation identities, single ownership, cancellation lease and stale epoch no-op.
- Same-snapshot candidate collection, cross-scope arbitration, atomic transition plan and bounded microsteps.
- Typed condition/event memory, Task result/deadline and selector cooldown using battle time.
- Activation-owned target lock, typed lexicographic query/path cost and immediate invalidation.
- Explicit Boss movement intent before ordinary planner fallback, deterministic static-grid query and monotonic
  `RejoinForwardRoute` cursor/result adapter.

Existing movement, blocking, attack, skill, effect, projectile, damage and death remain leaf authorities. P03 must not
add official Boss live content or public internal graph fields.

Exit gate:

- Definition/setup types from P02 are consumed without duplicate authority.
- Non-opt-in runtime path remains unchanged under characterization tests.
- Admission/transition failure is atomic across graph/intent/leaf/resource/cooldown/RNG/event/revision state.
- Pause, same-time completion/timeout, stale callback, reentry, released delivery and microstep overflow are
  deterministic.
- Target/path/route cursor permutations and result exhaustiveness pass focused tests.
- Completion audit returns `Complete / High`.

### P04 — `TrackAndRejoin` fixture vertical slice

Execution mode: `implement_and_verify`

Scope:

- One test-fixture encounter and actor binding plus a paired no-binding ordinary-route fixture.
- One encounter/actor/Phase and a finite reference Pattern that acquires a deployed hostile Unit, approaches it and
  rejoins the authored forward route.
- Only implemented selector/query/Task/condition vocabulary; unsupported future variants remain strict errors.
- Existing movement/attack/damage/death event and checkpoint projection only.

Exit gate:

- Authoring→atomic bundle→typed setup→spawn→runtime→target/movement→Task result→graph transition→event/checkpoint→
  server path is exercised end-to-end.
- Success, no target, invalid target, path unavailable, timeout, lifecycle interrupt and reconnect cases are protected.
- Target does not churn, route cursor never regresses, no lifecycle result fires twice and reconnect does not reroll.
- Normal server transport remains normalized-equivalent and no new public Boss variant/field is present.
- Existing Unity-supported movement/damage projection remains sufficient for this fixture evidence.
- Completion audit returns `Complete / High`.

### P05 — Official live reference Boss

Execution mode: `implement_and_verify`

Activation condition:

- The user has approved the first live reference Boss identity and the required gameplay vocabulary/presentation
  boundary. This deferred content decision does not block P00~P04.

Scope:

- CRA-P01-C02 all-manifested-difficulty validation and CRA-P05-C02 full setup/Boss-binding feasibility.
- One separately approved reference Boss definition, encounter opt-in and only its required implemented vocabulary.
- Official live content, balance/asset identity and closest gameplay/data/runtime canonical.
- End-to-end Core/server and accessible consumer verification for the actual reference behavior.

This Phase does not implement all three final-boss concepts. It must not use an unimplemented selector/action/mechanic
as a silent no-op or placeholder.

Exit gate:

- Every manifested live difficulty and resolved battle setup validates before publication.
- The approved live encounter starts, executes, fails/interrupts/reconnects and terminates deterministically.
- Ordinary live encounters remain unchanged.
- Required live RON, canonical, focused/broad tests and actual consumer evidence agree.
- P06 activation requirement is decided from the actual reference Boss public presentation contract.
- Completion audit returns `Complete / High`.

### P06 — Conditional public Boss lifecycle cutover

Execution mode: `implement_and_verify` when activated

Activation condition:

- The approved live Boss requires a public phase/objective/telegraph/duration/facing/semantic action that existing
  movement/damage/death events and checkpoint cannot represent completely.

Scope when required:

- Core semantic lifecycle DTO/event with stable presentation instance, cause, time/seq and reconnect-active checkpoint.
- CRA-P10-C03 exact normalized parity across normal update, setup and resync.
- Current Unity battle-event adapter, lifecycle registry/readiness, same-UUID reset, cancel/end idempotence, checkpoint
  rebuild and actual Spine/VFX mapping.
- Strict producer-consumer rollout with no gameplay recomputation in Unity.

If P05 proves the approved v1 is fully existing-event-only, P06 may be `not_applicable_to_v1` only through an explicit
Master disposition that records evidence and future owner. It is not silently skipped.

Exit gate when activated:

- Core/server/Unity use one semantic generation and exact serialized meaning.
- Reconnect restores active lifecycles without replaying completed cues; cancel/end/death occurs exactly once.
- Internal graph/target/task/epoch remains private.
- Focused producer, transport, Unity consumer and cross-component validation pass.
- Completion audit returns `Complete / High`.

### P07 — Master final completion audit

Execution mode: `audit_only`

Scope:

- Re-audit BP-01~BP-34 against final code, live data, tests, canonical, server and current Unity consumer.
- Verify all Phase seams together, refactor sub-track dispositions, legacy/compatibility absence and dirty preservation.
- Run or validate the smallest sufficient cross-component broad suite and actual first live reference Boss path.
- Confirm P06 completion or justified `not_applicable_to_v1` disposition.

Exit gate:

- All Master completion matrix rows pass.
- No material policy, consumer, migration, legacy or validation gap remains.
- Independent verdict is `Complete / High`; only then may Master Status become `complete` and
  `Implementation Complete: true`.

## Activation Rules

- Master production mutation is always forbidden.
- 기본적으로 한 번에 하나의 production Phase만 `active`다.
- Independent refactor work may run in a separate Goal only after P00 records non-overlapping files, authority,
  migration and test surfaces.
- A child Phase claiming completion does not unlock the next Phase. Its completion audit must return
  `Complete / High` and the Master registry must be updated explicitly.
- P00 itself is investigation-only. Its required terminal verdict is `ready_for_g0`,
  `policy_decision_required`, `scope_decision_required` or `externally_blocked`.
- No Boss production code, schema, live data or Unity work begins while P00 is merely `ready`.

## Plan Change Control

현재 Phase registry와 순서는 의존성에 근거한 execution baseline이지 불변 일정이 아니다. P00뿐 아니라 실제
구현, focused/broad 검증, Phase 완료 감사와 P07 최종 감사에서 새 current evidence가 발견되면 계획을 변경할
수 있다. 계획 변경은 편의나 대화 맥락이 아니라 repository evidence와 완료 조건으로 정당화한다.

### Replanning triggers

- 실제 source-of-truth, mutation path 또는 producer-consumer boundary가 계획한 Phase ownership과 다름
- 원자적 data/schema/runtime/deployment 또는 rollback을 위해 인접 Phase의 분할·병합·재정렬이 필요함
- 구현·검증에서 새 prerequisite, collision, migration, external consumer 또는 testability dependency가 확인됨
- 완료 감사에서 이미 완료된 Phase evidence를 무효화하는 material defect나 cross-Phase gap이 발견됨
- 최초 live reference Boss가 확정되어 P06의 activation/not-applicable disposition 또는 P05/P06 경계를 바꿔야 함

### Permitted plan changes

- 승인된 최종 결과 안에서 Phase를 분할·병합·재정렬하거나 correction Phase를 추가하고 conditional Phase를
  연기 또는 `not_applicable_to_v1`로 disposition할 수 있다.
- 승인 범위 안의 기술적 실행 순서, child 이름과 evidence owner 조정은 새 gameplay 정책 승인이 아니다.
  하지만 새 child 생성·activation은 갱신된 registry와 `Activation Rules`를 따라야 한다.
- BP-01~BP-34 coverage, 최초 live reference Boss라는 최종 결과, 공통 불변식과 required consumer/migration
  closure를 제거·축소할 수 없다. Independent CRA candidate를 근거 없이 Boss completion gate로 바꿀 수도 없다.
- 사용자-visible/gameplay 의미, 승인된 범위, compatibility policy 또는 최종 완료 의미를 바꾸는 변경은
  evidence와 선택지·영향을 제시하고 사용자 승인을 받기 전 구현하지 않는다.

### Required change procedure

1. 영향받는 production 작업을 안전한 경계에서 중단하고 current Phase status를 실제 상태로 유지한다.
2. 변경을 촉발한 source/test/audit evidence, 원래 가정이 틀린 이유와 검토한 대안을 해당 child의
   `EXPERIMENTS.md`와 `EXPERIMENT_NOTES.md`에 bounded unit 직후 기록한다.
3. 완료된 Phase의 code/data/test/canonical evidence가 여전히 유효한지 판정한다. 무효화되면 해당 Phase를
   correction 대상으로 표시하고 그 evidence를 다음 Phase prerequisite로 재사용하지 않는다.
4. 추가 production 작업 전에 Master의 Phase registry, dependency, BP/CRA coverage, migration/rollout,
   validation과 completion matrix 영향을 갱신하고 Master experiments/notes에 변경 전후를 기록한다.
5. 정책 승인이 필요하면 관련 Phase를 `policy_decision_required`로 두고, 구조적 변경만 필요하면 갱신된
   prerequisite와 독립 완료 감사 gate를 충족한 뒤 activation 절차를 다시 따른다.
6. `audit_only` Phase는 production correction을 직접 수행하지 않는다. 영향받는 구현 Phase를 다시 열거나
   범위가 독립적이면 별도 correction Goal을 registry에 추가한 뒤 감사를 재수행한다.

계획 변경 기록은 대화나 compact된 context로 대체하지 않는다. 변경이 확인된 시점에 문서화하며, 여러
변경을 Phase 종료 시 한꺼번에 소급 작성하지 않는다.

## Common Invariants

- Boss opt-in이 없는 enemy의 authored-route behavior는 변하지 않는다.
- Core가 graph, target, transition, timing, RNG, event order와 authoritative state를 소유한다.
- Existing movement/block/attack/skill/effect/projectile/damage/death subsystem이 leaf execution authority를
  유지한다.
- Same-time ordering, battle time/Pause, activation epoch, stale events와 checkpoint/reconnect는 분리하지 않는다.
- Failed admission/transition/advance는 state, resource, cooldown, RNG, intent, event, revision과 cursor 일부를
  남기지 않는다.
- Invalid/missing/unimplemented RON은 strict rejection하며 fallback/no-op/partial generation을 허용하지 않는다.
- Internal graph/target/task identity는 public presentation contract가 아니다.
- Refactor audit candidate는 production 권한이 아니며 활성 child Goal에서만 구현한다.
- 현재 dirty/untracked 사용자 변경을 덮어쓰거나 campaign cleanup으로 흡수하지 않는다.
- Future mechanic을 위한 placeholder enum/state/DTO/schema를 선행 구현하지 않는다.

## Migration And Rollout Strategy

### DTO migration

- P01은 public Boss schema/Wire를 추가하지 않는 correctness correction이다.
- P02/P03의 definition, compiled frame과 runtime identity는 Core 내부 typed contract가 기본이다. Phase 조사에서
  serialized boundary가 확인되면 해당 child가 producer, 모든 접근 가능한 consumer, version과 제거 조건을
  명시하기 전에는 shape를 추가하지 않는다.
- P04는 existing public battle events/checkpoint만 사용하는 Wire-neutral slice가 완료 기준이다.
- 최초 public Boss lifecycle이 실제로 필요할 때만 P06이 Core semantic DTO, normal/setup/resync server mapping과
  current Unity consumer를 단일 cutover로 변경한다. 내부 graph/target/task identity는 DTO로 승격하지 않는다.

### Data and live RON migration

- P02는 별도 Boss catalog를 atomic live-bundle candidate에 포함하되 official encounter opt-in 없이 strict
  empty/test-fixture foundation으로 시작한다. Source inventory, parse, reference와 feasibility 중 하나라도
  실패하면 candidate 전체를 reject한다.
- Official live RON reference와 encounter/actor opt-in은 CRA-P01-C02/P05-C02 gate 뒤 P05가 한 배포 단위로
  소유한다. Old/new dual catalog, partial generation, missing-definition fallback은 두지 않는다.

### Save and checkpoint migration

- 현재 active battle은 Core session memory이며 Unity checkpoint는 reconcile projection이지 durable Core restore
  format이 아니다. 이 Master는 기존 durable save를 Boss runtime state로 자동 확장하거나 in-place 변환하는
  migration을 승인하지 않는다.
- 기본 cutover 가정은 새 binary/data generation에서 새로 시작한 battle만 Boss opt-in하며, 이미 실행 중인
  battle을 새 Boss runtime으로 변환하지 않는 것이다. P00은 실제 deployment/session lifetime을 확인하고 이
  가정이 성립하지 않으면 P05 activation 전 정책 결정을 올린다.
- P03 graph/activation/target/task state는 authoritative session 내부 상태로 남는다. P06이 필요하면 reconnect용
  public semantic lifecycle projection만 checkpoint에 추가하며 internal graph restore schema는 추가하지 않는다.

### Version policy

- P01~P05는 새 public Boss schema version을 선행 예약하지 않는다. Live catalog/setup은 current atomic bundle
  generation 안에서 함께 publish되고, incomplete generation은 외부에 노출하지 않는다.
- P06이 필요하면 current transport의 실제 version mechanism을 조사해 Core/server/Unity가 같은 semantic
  generation을 식별하게 한다. Version field가 필요하다는 결론 자체는 P06 조사 전 확정하지 않으며, implicit
  mixed-version compatibility를 허용하지 않는다.
- Hot reload/LKG나 여러 Boss schema generation의 동시 지원은 v1 비범위다.

### Deployment and producer-consumer rollout

- P01~P05는 각 child completion audit을 통과한 원자적 Phase 단위로만 다음 Phase를 연다. Intermediate Phase가
  완료되지 않은 schema/data/runtime를 다음 Phase production이 보완한다는 가정은 허용하지 않는다.
- P06이 활성화되면 Core producer, game_server normal/setup/resync와 current Unity consumer는 같은 semantic
  generation으로 배포·검증한다. 한 consumer를 stale shape로 남긴 채 Master를 완료하지 않는다.
- 접근 불가능한 필수 consumer나 실제 배포 lifetime을 확인할 수 없으면 해당 Phase는 `externally_blocked`다.

### Compatibility policy

- 승인되지 않은 dual schema, dual read/write, fallback, compatibility alias와 placeholder를 두지 않는다.
- Boss opt-in이 없는 ordinary encounter parity는 영구 compatibility shim이 아니라 유지해야 할 현재 동작이다.
- 조사 결과 temporary compatibility가 불가피하면 지원 대상, 승인 근거, 제거 Phase와 focused test를 명시하고
  사용자 승인 전 구현하지 않는다.

## Focused Validation

- P00: source/artifact traceability, material drift, Goal/file/authority collision, dirty preservation과 exact registry
  coverage를 검증한다. Production runner는 실행하지 않는 것이 기본이다.
- P01: terminal/max-time entrypoint parity, unequal event total order/permutations와 injected advance rollback을
  characterization/failure-injection test로 검증한다.
- P02: source inventory, strict parse/reference/variant/outcome/capability/loop/typed-parameter validation, atomic
  candidate rejection과 no-binding setup parity를 검증한다.
- P03: lifecycle arbitration, atomic transition, Pause/same-time/stale/reentry/microstep, target lock/path cost/route
  cursor/result exhaustiveness를 검증한다.
- P04: fixture와 paired non-opt-in path의 authoring-to-server vertical slice, 실패/interrupt/reconnect를 검증한다.
- P05: manifested difficulty, full setup feasibility, actual live Boss start/lifecycle/termination/reconnect와 ordinary
  live encounter parity를 검증한다.
- P06 when required: Core DTO, server normal/setup/resync parity, Unity lifecycle idempotence/readiness/checkpoint rebuild
  및 실제 presentation mapping을 검증한다.
- 각 child는 exact command, 실제 test 수와 pass/fail/skip을 자체 `EXPERIMENTS.md`에 즉시 기록하며 독립
  completion audit을 수행한다. Runner가 수치를 제공하지 않으면 추측하지 않는다.

## Broad Validation

- 각 구현 child는 focused test 뒤 영향 범위의 smallest sufficient Core/workspace/server suite를 실행하고,
  current baseline failure와 새 failure를 구분한다.
- Core battle lifecycle/runtime를 바꾸는 P01/P03/P04는 관련 Core 전체 battle regression surface를, live
  data/setup을 바꾸는 P02/P05는 strict RON loading과 setup integration surface를 포함한다.
- Server contract를 통과하는 P04/P05와 조건부 P06은 normal/setup/resync 경로의 broad integration surface를
  포함한다. P06이 활성화되면 current Unity consumer의 실제 available test/build surface도 포함한다.
- Exact suite와 command는 P00 또는 해당 child 조사에서 current runner를 확인한 뒤 고정하며, Master가 실행하지
  않은 명령을 미리 성공 evidence로 기록하지 않는다.

## Omitted Validation And Rationale

- 이 Master 문서 수정에는 production mutation이 없으므로 runtime/test runner를 실행하지 않는다. 문서 구조,
  ID coverage, child 미생성과 Master-excluded dirty preservation만 검증한다.
- P00은 `investigation_only`이므로 drift 사실 확인에 꼭 필요한 read-only 명령 외 runtime test를 생략한다.
  current evidence가 stale하거나 collision 판정에 재현이 필요하면 이유를 기록하고 범위 안 검증만 실행한다.
- P04가 Wire-neutral임을 입증한 경우 새 public lifecycle 전용 Unity test는 생략할 수 있지만 기존 event 소비가
  충분하다는 source/evidence를 남긴다. P06이 필요해지면 이 생략은 허용되지 않는다.
- 미래 final-boss 3종 content, 미승인 art/VFX/animation과 독립 refactor candidate 검증은 명시적 비범위라
  이 Master completion suite에 포함하지 않는다.

## Cross-Phase Validation

- P01 correctness가 P03 scheduler/transition의 실제 substrate인지 재검증한다.
- P02 authored definition/setup identity가 P03 runtime과 P04 fixture에서 중복 lookup 없이 소비되는지 확인한다.
- P04의 non-opt-in parity와 deterministic reconnect가 P05 live data에서도 유지되는지 확인한다.
- Public lifecycle이 존재하면 Core DTO, server normal/resync/setup과 Unity playback/checkpoint가 같은 generation을
  사용하는지 P06에서 검증한다.
- P07은 BP-01~BP-34 → production evidence → live data → tests → canonical → consumer trace를 다시 감사한다.

## Master Completion Conditions

1. BP-01~BP-34가 final production runtime/data/test/canonical과 정확히 연결된다.
2. Boss opt-in 없는 current enemy behavior가 보존된다.
3. Boss catalog, typed setup, encounter/actor runtime, lifecycle, target/movement와 transition atomicity가
   production path에서 완결된다.
4. 최소 tracking slice와 최초 live reference Boss가 실제 end-to-end 경로로 검증된다.
5. 필요한 public Boss presentation이 Core/server/current Unity에서 reconnect-safe하게 완결되거나, v1이
   existing-event-only임이 승인된 범위와 evidence로 명시된다.
6. Refactor sub-track candidate가 prerequisite/coimplement/independent/deferred/public-wire disposition대로
   처리되고 전체 catalog가 불필요한 gate가 되지 않는다.
7. Production live data, strict validation, DTO/save/checkpoint/migration과 consumer rollout이 한 최종 계약을
   사용한다.
8. 모든 required Phase가 자체 `Complete / High`를 받고 P07 cross-Phase audit도 `Complete / High`다.
9. 관련 canonical이 실제 구현된 current contract로 갱신된다.
10. 초기/Phase별 dirty 사용자 변경과 범위 밖 파일이 보존된다.

## P00 Entry And Exit Gate

P00은 다음을 확인한다.

- readiness audit 이후 material current-tree drift
- active/incomplete Goal과 file/authority collision
- preliminary Phase split의 atomicity와 deployment lifetime
- 19개 relevant refactor candidate의 current execution disposition
- P01 G0 owner, exact characterization tests, implementation order와 validation commands
- first live reference Boss가 정해진 뒤 public Wire Phase 필요 여부를 판정할 evidence와 latest decision point

P00은 `Complete / High`와 `ready_for_g0`를 모두 받아야 P01을 `ready`로 바꿀 수 있다. Material policy,
scope, external access 또는 current-tree collision blocker가 있으면 해당 terminal verdict를 기록하고 P01을
활성화하지 않는다.

## Open Policy Decisions

- P05 reference-content gate was resolved on 2026-08-09. The user approved option 1 in
  `P05_CONTENT_DECISION_PACKET.md`: Little Red is a replaceable test/reference live Boss rather than final authored
  character content; current balance/rewards/skill and generic opponent presentation remain, and no dedicated
  bounty lifecycle, art or VFX is added.
- P06 필요 여부는 위 reference Boss의 실제 public presentation contract로 결정한다. P00은 decision gate와
  필요한 evidence만 정의하고, P05가 actual content evidence를 만든 뒤 P06 activation 또는
  `not_applicable_to_v1` disposition을 확정한다.
- 현재 deployment가 active battle을 binary/data generation 사이에 유지해 기본 no-in-place-migration 가정을
  깨는지 P00이 조사한다. 깨지지 않으면 정책 질문이 아니며, 깨진다면 P05 전에 session cutover 의미를
  사용자에게 결정받는다.

새 질문이 target policy나 v1 completion meaning을 바꾸면 관련 Phase activation을 중단하고 evidence,
선택지, 권장안과 영향을 기록한다.

## Stop Conditions

- BP-01~BP-34 또는 최초 live reference Boss의 의미를 바꾸는 사용자 정책이 필요함
- Current tree가 readiness evidence와 material하게 달라 현재/목표 경계를 확정할 수 없음
- 사용자 dirty change나 active Goal이 다음 Phase authority를 중간 migration 상태로 소유함
- 필수 Core/live data/server/Unity consumer 또는 validation 환경에 접근할 수 없음
- Phase를 독립적으로 완료 감사할 수 없는 temporary dual authority가 필요함
- Master가 직접 production mutation을 해야만 진행 가능함

## Completion Audit Evidence Matrix

| ID | Master completion evidence | Required artifact | Status |
| --- | --- | --- | --- |
| BRM-C01 | BP-01~BP-34 exact final coverage | P07 final policy matrix | pass |
| BRM-C02 | P00 current-tree and collision gate | P00 completion audit + `ready_for_g0` | pass |
| BRM-C03 | G0 correctness substrate | P01 completion audit | pass |
| BRM-C04 | Definition/publication/setup foundation | P02 completion audit | pass |
| BRM-C05 | Runtime/lifecycle/target/movement foundation | P03 completion audit | pass |
| BRM-C06 | Tracking vertical slice | P04 completion audit | pass |
| BRM-C07 | First live reference Boss | P05 completion audit | pass |
| BRM-C08 | Required public consumer cutover | P06 completion audit or explicit approved not-required evidence | pass — `not_applicable_to_v1` |
| BRM-C09 | Refactor sub-track final disposition | Master registry and child evidence | pass |
| BRM-C10 | Canonical/data/DTO/checkpoint alignment | final cross-component audit | pass |
| BRM-C11 | Focused/broad validation | Phase results and P07 integration validation | pass |
| BRM-C12 | Dirty/user-change preservation | initial/Phase/final scoped status evidence | pass |
| BRM-C13 | Incremental durable evidence | Master and child experiments/notes | pass |
| BRM-C14 | Independent Master completion audit | P07 `Complete / High` | pass |
