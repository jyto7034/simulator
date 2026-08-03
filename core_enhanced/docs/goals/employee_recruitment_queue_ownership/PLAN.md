# Employee RecruitmentQueue Ownership Contract

Status: complete

Implementation complete: true

Created: 2026-07-18

Last updated: 2026-07-18

Goal type: `employee_domain_implementation_master` Phase 2 Child Goal

## 목표

Phase 1의 stable `employee_definition_id`를 key로 사용해 미합류 직원 definition을 run-seeded finite `RecruitmentQueue`, active recruitment session과 roster 중 정확히 한 곳에서만 소유하고, HQ 선택·이탈·재접속·checkpoint·Floor 전환 전체에서 후보 순서와 ownership을 결정적으로 보존한다.

## 해결할 사용자-visible 문제

현재 HQ 후보는 node seed로 unified recruitment catalog를 다시 shuffle/truncate하므로 반복 노드에서 이미 합류한 definition이 다시 보이거나, 선택하지 않은 후보의 순환 순서가 run 상태로 보존되지 않는다. 플레이어는 같은 run/seed/checkpoint에서 안정적인 유한 후보 순환과 중복 없는 채용을 경험해야 한다.

## 감사 Finding ID

없음. 승인된 직원 도메인 정책과 완료된 난이도 감사에서 도출된 Master Phase 2다.

## Master와 직접 의존성

- Master: `docs/goals/employee_domain_implementation_master/PLAN.md` Phase 2
- 선행 Phase: `docs/goals/employee_definition_profile_contract/`
- 선행 completion review: `docs/goals/employee_definition_profile_contract/COMPLETION_REVIEW.md`
- 승인 정책: `docs/goals/employee_domain_implementation_difficulty_audit/PLAN.md`, `docs/decisions/employee_domain_policy_2026-07-17.md`

Master와 Phase 1 문서는 Phase 2 초기화 전에 읽었으며, Phase 1의 unified definition/Profile shape와 identity 분리를 변경하지 않는다.

## 대상 저장소와 경로

- Core runtime/state/DTO/tests와 Goal/canonical 문서: `/mnt/f/work/simulator/core_enhanced`
- production live data: `/mnt/f/work/simulator/game_resources`
- game_server transport/tests: `/mnt/f/work/simulator/game_server`
- Unity recruitment reader/command/presentation/tests/canonical: `/mnt/f/unity projects/ark`

## 현재 Runtime 및 Live Data 근거

조사 결과 기존 runtime은 HQ 진입 때 non-starter catalog를 node seed로 매번 shuffle/truncate했고, `HeadquartersContactSessionState`가 full `EmployeeDefinition` 사본을 보유했으며 `RunState`, checkpoint payload와 Floor carry-forward에는 채용 소유권 상태가 없었다. HQ 채용은 roster/UUID를 먼저 변경한 뒤 node를 완료해 completion 실패 시 부분 적용 위험도 있었다. game_server는 Core DTO의 얇은 transport였고 Unity는 이미 `employee_definition_id` 기반 후보/명령 계약을 소비하므로 public wire shape 변경은 필요하지 않았다.

## Source of Truth와 소유 계층

- 미합류 definition order: Core run-local `RecruitmentState.queue`
- 현재 제안 중 definition: Core `RecruitmentState.active_session`
- 합류 instance: Core roster의 `employee_definition_id + employee_uuid`
- identity key: Phase 1 `employee_definition_id`
- checkpoint/Floor persistence: Core run state와 checkpoint payload
- public projection/command validation: typed Core DTO와 behavior
- transport/presentation: game_server pass-through와 Unity consumer

Queue/session은 ID만 저장하며 HQ active content에는 후보 사본을 저장하지 않는다. DTO/snapshot은 active session ID를 unified catalog에 즉시 투영한다. HQ action은 staged queue/roster/resource와 node completion을 하나의 rollback 가능한 transaction으로 commit한다.

## 유지할 불변식

- 같은 definition은 Queue, active session, roster 중 정확히 한 곳에만 존재한다.
- Queue와 session은 `employee_definition_id`만 ownership key로 사용한다.
- roster 합류 시에만 새 run-local `employee_uuid`를 만든다.
- 시작 미선발 definition과 unified catalog의 non-starter definition을 승인된 run-seeded 초기 queue에 포함한다.
- candidate session 생성은 queue head에서 정의된 수를 원자적으로 이동하며 반복 snapshot/request로 reroll하지 않는다.
- 채용한 definition은 roster로 이동하고 선택하지 않은 후보는 승인 순서로 queue tail에 돌아간다.
- 다른 HQ action/이탈/error도 definition을 유실·복제하지 않는다.
- reconnect, checkpoint load와 Floor carry-forward는 queue/session/order/roster ownership을 정확히 복원한다.
- snapshot과 command validation은 같은 ownership state를 사용한다.
- Phase 1 Profile DTO와 authored catalog를 재설계하지 않는다.
- compatibility, fallback, dual ownership flag나 duplicate derived state를 추가하지 않는다.
- 기존 사용자 변경과 범위 밖 dirty 파일을 보존한다.

## 승인된 정책

- 미합류 직원 순서는 run seed로 한 번 결정한 순환 대기열이 소유한다.
- 시작 선발에서 선택하지 않은 직원은 queue에 남는다.
- session은 queue에서 정해진 수의 후보를 가져와 고정한다.
- 선택한 직원은 roster, 선택하지 않은 직원은 queue tail로 이동한다.
- Queue/Session/Roster는 exactly-one ownership을 유지한다.
- checkpoint는 후보 목록과 queue 순서를 복원한다.
- 같은 definition은 한 run에서 중복 합류하지 않는다.
- no-compatibility complete cutover를 사용한다.

## 미결정 정책

없음. 현재 UX에는 HQ session을 별도로 취소하는 command가 없고, 채용 이외의 실제 종료 분기인 응급지원과 HQ 상점 진입은 session 순서를 그대로 queue tail에 반환한다. 오류는 staged clone을 폐기해 기존 session/queue를 유지한다.

## 범위

- run-seeded finite queue 초기화
- 시작 미선발과 non-starter definition의 queue 편입
- Queue → active HQ session → roster/tail transaction
- session 고정과 reroll 방지
- definition dedupe/exactly-one invariant validation
- 다른 HQ action, node completion/error/abandonment 전이
- reconnect/snapshot, SavePoint checkpoint load, Floor carry-forward
- recruitment DTO/command/result와 Unity consumer migration
- 관련 fixtures/tests/canonical 문서와 legacy node-seeded random path 제거

## 명시적 비범위

- Phase 1 EmployeeDefinition/Profile 재설계
- Phase 3 condition/eligibility/death/treatment
- Phase 4 Psychology/trust/memory
- 새 recruitment 콘텐츠, 후보 수 밸런스나 HQ UI 전면 재설계
- durable external save compatibility
- 관련 없는 map/HQ/shop refactor

## 구현 단계

1. 승인 정책, Phase 1 completion review와 current dirty overlap을 읽는다.
2. queue/session/roster mutation, checkpoint/Floor persistence, DTO/server/Unity와 tests를 조사한다.
3. production 변경 전 focused/broad baseline을 기록한다.
4. run-seeded queue와 exactly-one invariant를 구현한다.
5. HQ session의 모든 정상/오류/비채용 action transition을 원자화한다.
6. checkpoint/Floor/reconnect 및 public DTO/Unity consumer를 complete cutover한다.
7. node-seeded reroll/duplicate legacy와 관련 fixture/mock/test를 제거한다.
8. focused/broad tests, legacy search, canonical sync와 completion review를 수행한다.

## 사용자-visible 완료 조건

- 같은 run/seed/checkpoint에서 후보 순서가 안정적이고 반복 요청으로 reroll되지 않는다.
- 선택하지 않은 후보가 승인된 순서로 다시 순환한다.
- 같은 definition은 중복 채용되거나 Queue/Session/Roster에 중복 존재하지 않는다.
- HQ 비채용 action·이탈·재접속·checkpoint load·Floor 전환 뒤에도 ownership과 순서가 정확히 복원된다.
- Core producer와 Unity consumer가 같은 recruitment contract를 사용한다.

## DTO Migration

필요. 현재 session candidate projection과 command/result를 queue ownership state에 맞춰 complete cutover한다. Phase 1 `EmployeeDefinition` shape와 `employee_definition_id` command identity는 유지한다.

## Data 또는 Live RON Migration

원칙적으로 없음. Phase 1 unified catalog를 소비한다. 조사 결과 authored queue policy schema가 반드시 필요하면 승인 근거와 migration을 먼저 기록한다.

## Save 및 Checkpoint Migration

필요. run queue, active session ownership과 order를 SavePoint checkpoint 및 Floor carry-forward에 포함한다. 외부 durable save가 발견되면 승인 없이 migration/compatibility를 정하지 않는다.

## 필수 Focused 검증

다음 focused 검증으로 증명했다.

- same seed/order deterministic queue initialization
- 미선발 starter + non-starter pool completeness/uniqueness
- Queue/Session/Roster exactly-one invariant
- session repeat/reconnect no-reroll
- recruit one + unselected tail order
- HQ non-recruit actions와 error rollback
- checkpoint save/load와 Floor carry-forward exact restore
- duplicate recruitment rejection without partial mutation
- exact Core JSON/server request와 Unity recruitment flow

## 필수 Broad 검증

Core 전체, game_server 전체, Unity focused/full EditMode와 필요한 PlayMode flow를 실행하고 baseline과 비교한다. 실제 pass/fail/skip 수를 기록한다.

## 생략할 검증과 근거

- live RON migration: queue는 Phase 1 unified catalog에서 초기화되며 새 authored schema가 없다.
- game_server/Unity production 변경: public candidate/result/command shape가 이미 stable `employee_definition_id` 계약이어서 변경하지 않았다. server 30 tests와 Unity 전체 EditMode/PlayMode로 소비자 일치를 재검증했다.
- active HQ SavePoint 생성: 실제 SavePoint는 planning return에서만 저장되므로 active HQ session 중 checkpoint를 만드는 public 경로가 없다. payload는 `RecruitmentState` 전체를 포함하고, 정상 checkpoint에서는 active session이 `None`이며 다음 후보 목록은 queue order에서 정확히 재구성된다.

## 중단 조건

- Phase 2와 독립적인 source-of-truth 경계가 필수로 발견됨
- 승인되지 않은 DTO/RON/save/UX/balance/compatibility 결정이 필요함
- Phase 1 identity/Profile 계약이 current runtime에서 깨졌음
- dirty 사용자 변경과 안전하게 병합할 수 없음
- 필수 external consumer/runner를 검증할 수 없음

## Compatibility

허용하지 않는다. 필요성이 발견되면 구현 전에 사용자 승인을 요청한다.

## 완료 판단

모든 정상·오류·rollback/persistence transition이 exactly-one invariant와 deterministic order를 보존하고, Core/server/Unity 계약과 focused/broad evidence가 일치해 `complete`로 종료했다. 상세 증거는 `COMPLETION_REVIEW.md`에 있다.
