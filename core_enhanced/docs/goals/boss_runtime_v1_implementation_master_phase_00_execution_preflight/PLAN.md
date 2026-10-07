# Boss Runtime v1 — Phase 00 Execution Preflight Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: phase
Execution Mode: investigation_only
Production Mutation Authorized: no
Master Goal: `../boss_runtime_v1_implementation_master/PLAN.md`
Master Phase: P00
Activated: 2026-08-09
Completed: 2026-08-09
Simulator Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## Objective

완료된 Boss Runtime readiness audit 이후의 current-tree drift와 active-work collision을 확인하고, Boss Runtime
v1 구현의 실제 Phase 경계와 P01 G0 correctness handoff를 현재 source/test evidence로 확정한다.

## User/Contract-Visible Problem

Master는 Boss Runtime v1의 최종 결과와 preliminary P00~P07 순서를 정의했지만, 현재 working tree에는 많은
사용자 변경과 진행 중·완료 Goal artifact가 공존한다. 과거 readiness evidence를 그대로 실행 계획으로 사용하면
다음 위험이 있다.

- scheduler/terminal/world advance 또는 data/setup ownership이 audit 이후 material하게 바뀌었을 수 있다.
- 다른 active/incomplete Goal이 P01 또는 후속 Boss Phase의 파일·authority·migration을 이미 소유할 수 있다.
- preliminary P02/P03, P05/P06 경계가 current atomic deployment 및 test lifetime과 맞지 않을 수 있다.
- 리팩터 후보의 과거 분류가 현재 구현·충돌 상태를 반영하지 못할 수 있다.

P00의 contract-visible 결과는 production을 수정하는 것이 아니라, P01을 안전하게 시작할 수 있는 current
owner, characterization/failure-injection surface, exact validation command와 Phase registry를 durable artifact로
제공하는 것이다.

## Audit Finding ID

없음. P00은 completed readiness audit의 finding을 새로 만드는 감사가 아니라 current execution preflight다.
CRA candidate ID는 기존 감사 식별자이며 이 Phase의 구현 권한이 아니다.

## Direct Dependency And Approved Evidence

- Master phase definition: `../boss_runtime_v1_implementation_master/PLAN.md`의 P00, activation, migration,
  validation과 Plan Change Control
- Direct predecessor: `../boss_runtime_implementation_readiness_audit/COMPLETION_AUDIT.md`
  - Goal Status: `complete`
  - Completion verdict: `Complete / High`
  - Implementation readiness: `conditionally_ready`
- Required readiness artifacts:
  - `../boss_runtime_implementation_readiness_audit/BOUNDARY_AUDIT.md`
  - `../boss_runtime_implementation_readiness_audit/POLICY_COVERAGE_MATRIX.md`
  - `../boss_runtime_implementation_readiness_audit/IMPLEMENTATION_READINESS_REPORT.md`
- Target policy: `../../tmp/boss_behavior_state_machine_policy_notes_2026-08-07.ko.md`의 BP-01~BP-34
- Refactor source: `../core_enhanced_refactoring_audit_final_catalog_consolidation/FINAL_CANDIDATE_CATALOG.md`

직접 선행 구현 Phase는 없다. P00은 completed audit evidence를 current tree와 대조하며, audit 문서를 현재
runtime authority로 취급하지 않는다.

## Current Runtime And Live Data Evidence

P00 activation 시점의 확정 사실:

- Simulator commit은 readiness/Master start commit과 같은 `f318414fb0156a67e04c5479000c2a2683fc0c09`다.
- P00 생성 직전 전체 `git status --short --untracked-files=all`은 221개 항목이며 SHA-256은
  `41f02b5b73145453f032c3dfcf2931f1225db3225cb70a7ccaf82de5e882bd8d`다.
- 위 221개에는 Master의 새 Markdown 3개가 포함된다. 나머지 dirty/untracked 변경은 사용자 소유로 보존한다.
- P00 child path는 생성 전 존재하지 않았고 적용 가능한 `AGENTS.md`는 발견되지 않았다.
- Readiness audit 당시에는 Boss catalog/runtime/binding이 없었고, terminal parity, total event order와 atomic
  world advance가 G0 prerequisite였다.

Current source, live data, server, test와 Unity consumer의 audit 이후 material drift는 아직 `discovery pending`이다.
P00 문서 작성 전에는 repository-wide runtime/code search나 test runner를 실행하지 않았다.

## Source Of Truth And Ownership

1. 현재 authoritative runtime code와 state mutation
2. production loader가 소비하는 live RON/data
3. 실제 Core DTO, game_server mapping과 접근 가능한 consumer
4. 현재 durable canonical
5. BP-01~BP-34 target decision record
6. Master, completed readiness/refactor audit와 P00 기록

P00은 source of truth를 새로 소유하지 않는다. Current fact와 target policy, historical audit conclusion을
분리하고 실행 owner/dependency만 확정한다.

## Invariants

- P00은 production code, data, test, fixture, canonical, server 또는 Unity 파일을 수정하지 않는다.
- 기존 dirty/untracked 사용자 변경을 restore, reset, stash, move, delete, format 또는 cleanup하지 않는다.
- P01 전에는 Boss schema/runtime/live data/public Wire를 구현하지 않는다.
- Boss opt-in이 없는 ordinary authored-route behavior는 보존 대상으로 유지한다.
- BP-01~BP-34 coverage를 줄이거나 independent CRA candidate를 근거 없이 Boss gate로 바꾸지 않는다.
- Current runtime, live data와 actual consumer를 과거 Goal 주장보다 우선한다.
- Unity는 직접 확인하기 전 current producer와 일치한다고 주장하지 않는다.
- Material finding은 bounded investigation unit 직후 P00 artifact에 기록하고 마지막에 일괄 복원하지 않는다.

## Refactor Candidate Execution Registry Input

P00은 다음 19개 candidate를 current evidence와 대조한다. 작업 유형과 우선순위의 정의는
`engineering_workflow.md` 및 final candidate catalog를 따르며 여기서 재정의하지 않는다.

| Candidate | Catalog work type | Preliminary Boss relation | P00 required outcome |
| --- | --- | --- | --- |
| CRA-P01-C01 | behavior-preserving refactor | coimplement with foundation | current source inventory owner/overlap/entry gate |
| CRA-P01-C02 | correctness correction | before official opt-in | current publication coverage owner and P05 gate |
| CRA-P05-C02 | correctness correction | before official opt-in | full setup feasibility owner and P05 gate |
| CRA-P05-C04 | behavior-preserving refactor | coimplement with foundation | typed setup boundary and P02/P03 atomicity |
| CRA-P06-C01 | correctness correction | prerequisite | terminal/max-time owner, test-first P01 handoff |
| CRA-P06-C02 | correctness correction | prerequisite | total-order owner, semantic key and permutation safety net |
| CRA-P06-C03 | correctness correction | prerequisite | atomic advance owner and failure-injection safety net |
| CRA-P07-C01 | correctness correction | independent | overlap and non-gate evidence |
| CRA-P07-C02 | correctness correction | independent | shared staging primitive without transaction-lifetime merge |
| CRA-P07-C03 | cleanup | defer after slice | Boss movement seam preservation and cleanup trigger |
| CRA-P07-C04 | cleanup | defer after slice | movement state cleanup dependency/consumer gate |
| CRA-P08-C01 | correctness correction | independent | leaf trigger reuse and non-gate evidence |
| CRA-P08-C05 | behavior-preserving refactor | independent | committed-damage leaf boundary evidence |
| CRA-P09-C03 | correctness correction | independent/policy-gated | Boss query separation from delivery usefulness policy |
| CRA-P09-C05 | behavior-preserving refactor | defer after slice | actual projectile seam trigger after vertical evidence |
| CRA-P09-C06 | behavior-preserving refactor | independent | typed reach is not navigation/path-cost authority |
| CRA-P10-C02 | behavior-preserving refactor | independent | command-result projection separation |
| CRA-P10-C03 | behavior-preserving refactor | first public Wire gate | exact Core/server DTO parity trigger and owner |
| CRA-P10-C04 | behavior-preserving refactor | independent | host snapshot overlay separation |

각 행은 current problem evidence, current/target authority, preserved invariant, overlap, entry/exit, validation safety
net, policy/external gate와 final execution disposition을 가져야 한다. P00은 어떤 candidate도 구현하지 않는다.

## Open Policy Decisions

- 최초 live reference Boss identity, required vocabulary, balance/assets와 presentation boundary는 P05 전 사용자
  결정이며 P00/P01을 막지 않는다.
- P06 필요 여부는 reference Boss의 실제 public presentation contract가 정해진 뒤 판단한다. P00은 evidence와
  latest decision point만 확정한다.
- Deployment가 active battle을 binary/data generation 사이에 유지하는지 current evidence로 확인한다. 기본
  no-in-place-migration 가정이 깨질 때만 P05 전 정책 질문을 올린다.

## In Scope

- readiness audit 이후 scheduler/terminal/advance, data/setup, movement/target, server DTO와 relevant Unity
  consumer의 material drift 확인
- active/incomplete Goal과 dirty file/authority/migration/test collision inventory
- 19개 CRA candidate current execution registry
- P01 G0 owner, milestone order, characterization/failure injection surface와 exact validation command 확정
- P02/P03 및 P05/P06 atomic boundary 검토
- P05 content decision 및 P06 public-contract decision gate 정의
- P00 completion audit과 Master registry handoff

## Out Of Scope

- production implementation, fixture/test/canonical mutation과 formatter
- P01 correctness correction 자체
- Boss catalog/runtime/live content/public DTO/Unity presentation 구현
- 전체 51개 refactor candidate 재감사 또는 독립 candidate Goal 생성
- 최초 live Boss identity, balance, art/VFX/animation 선택
- dynamic hot reload/LKG, durable active-battle restore와 future final-boss mechanic 설계

## Implementation Scope

없음. P00의 write 범위는 이 Goal의 조사·실험·감사 artifact와 P00 activation/completion에 필요한 Master
registry/experiment/note 갱신뿐이다.

## Required Artifacts

- `PLAN.md`
- `EXPERIMENTS.md`
- `EXPERIMENT_NOTES.md`
- `CURRENT_TREE_DRIFT_AUDIT.md`
- `ACTIVE_WORK_COLLISION_AUDIT.md`
- `REFACTOR_EXECUTION_REGISTRY.md`
- `P01_G0_HANDOFF.md`
- `COMPLETION_AUDIT.md`

추가 artifact는 실제 bounded investigation 결과가 생길 때 작성한다. 미확인 내용을 미리 결과처럼 채우지
않는다.

## DTO, Data And Save Migration

- DTO/wire migration: not_applicable — current producer/consumer drift와 future P06 gate만 조사한다.
- Data/live RON migration: not_applicable — source inventory/publication/setup boundary만 조사한다.
- Save/checkpoint migration: not_applicable — active battle lifetime과 no-in-place-migration 가정만 조사한다.
- Version/deployment migration: not_applicable — current topology와 후속 decision gate만 기록한다.

## Deployment And Compatibility

P00에는 배포가 없다. 승인되지 않은 compatibility, dual-read/write, alias, fallback 또는 placeholder를
추천안에 포함하지 않는다. 후속 Phase의 atomic deployment assumption이 current topology와 충돌하면 evidence와
필요한 정책 결정을 기록한다.

## Focused Validation

- required source/canonical/dependency artifact의 존재와 current relevance
- P00 artifact별 source path/symbol traceability
- active/incomplete Goal과 candidate별 file/authority collision matrix
- 19개 CRA ID exact set, 중복/누락과 disposition completeness
- P01의 C01/C02/C03 owner, characterization/failure injection 및 command feasibility
- preliminary Phase 8개와 BP-01~BP-34 coverage 보존
- P00 전후 Master-excluded dirty status preservation

Runner는 source evidence가 stale하거나 P01 command/filter feasibility를 정적으로 확정할 수 없을 때만 narrow
read-only validation으로 실행한다. 실행 시 actual pass/fail/skip 수를 기록한다.

## Broad Validation

Production mutation이 없으므로 전체 Cargo/server/Unity suite는 기본 요구사항이 아니다. Current drift 또는
collision verdict에 필수인 runtime fact를 static source/test-body evidence로 닫지 못할 때만 smallest sufficient
runner/probe를 선택하고 이유와 결과를 먼저 기록한다.

## Omitted Validation

- Boss target behavior test: 아직 구현 Phase가 아니므로 실행 대상이 아니다.
- Before/after regression: production diff가 없어 생략한다.
- Unity visual/EditMode/PlayMode full suite: current consumer/source collision 확인은 read-only로 수행하며 public
  presentation 구현이 없으므로 생략한다.
- Final-boss mechanic/balance/content validation: 명시적 비범위다.

## Completion Conditions

1. Readiness audit 이후 material current-tree drift가 source/test path와 함께 완전히 disposition된다.
2. Active/incomplete Goal 및 dirty file/authority collision이 P01~P07별로 분류되고 P01 ownership에 ambiguity가
   없다.
3. 19개 CRA candidate가 owner, overlap, entry/exit, validation, policy/external gate와 execution timing을 가진다.
4. P01 G0의 C01→C02→C03 순서 또는 evidence-backed 변경 순서, exact mutation owners와 safety-net commands가
   ready-to-execute handoff로 작성된다.
5. P02/P03 및 P05/P06 경계가 current ownership/deployment lifetime에 맞게 확정되거나 Plan Change Control에
   따라 Master registry가 먼저 갱신된다.
6. P05 content approval와 P06 public contract의 owner/evidence/latest decision point가 P01을 막지 않도록
   명시된다.
7. BP-01~BP-34와 Master completion meaning이 축소되지 않는다.
8. Production mutation이 없고 초기 사용자 dirty/untracked 변경이 보존된다.
9. P00 완료 감사가 `Complete / High`와 `ready_for_g0`를 함께 판정할 때만 Status를 `complete`로 바꾼다.

## Completion Audit Evidence Matrix

| ID | Completion evidence | Required artifact | Status |
| --- | --- | --- | --- |
| P00-C01 | current material drift disposition | `CURRENT_TREE_DRIFT_AUDIT.md` | complete |
| P00-C02 | active work/collision disposition | `ACTIVE_WORK_COLLISION_AUDIT.md` | complete |
| P00-C03 | 19-candidate execution registry | `REFACTOR_EXECUTION_REGISTRY.md` | complete |
| P00-C04 | executable G0 handoff | `P01_G0_HANDOFF.md` | complete |
| P00-C05 | Phase boundary and policy-decision gates | drift/collision audits + Master registry | complete |
| P00-C06 | BP/Master scope preservation | P00 completion audit | complete |
| P00-C07 | dirty/user-change preservation | initial/final scoped status evidence | complete |
| P00-C08 | independent completion audit | `COMPLETION_AUDIT.md`, `Complete / High` + `ready_for_g0` | complete |

## Stop Conditions

- Current tree가 readiness evidence와 material하게 달라 current/target boundary를 확정할 수 없음
- P01 ownership이 다른 active Goal이나 unsafe overlapping dirty change와 충돌함
- Phase 경계 변경이 approved Master result 또는 BP coverage를 바꿈
- gameplay/RON/DTO/save/compatibility/rollout 정책 없이는 P01 handoff를 확정할 수 없음
- 필수 Core/live data/server/Unity source, dependency artifact 또는 validation 환경에 접근할 수 없음
- P00 evidence를 얻으려면 production mutation이나 사용자 변경 침해가 필요함

사용자와 의논해 정해야 할 정책이 있으면 영향받는 결론을 중단하고 나머지 안전한 read-only 조사를 먼저
완료한 뒤 실제 Status와 필요한 입력을 보고한다.
