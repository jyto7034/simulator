# Employee Domain Implementation Master Goal

Status: complete

Implementation complete: true

Created: 2026-07-18

Last updated: 2026-07-18

Goal type: orchestration master. 이 Goal은 runtime, live data, DTO, server 또는 Unity 동작을 직접 구현하지 않는다.

## 목표

승인된 직원 도메인 정책을 네 개의 완결된 계약 Phase로 순차 구현하고, 각 Phase가 자신의 source of truth를 완전히 전환·검증한 뒤에만 다음 Phase로 진행하도록 조정한다.

이 Master Goal은 다음을 소유한다.

- Phase 범위와 순서
- Phase 사이의 구현 의존성
- Phase 진입·완료 게이트
- 정책·범위·외부 차단 중단 규칙
- Phase 사이의 legacy/compatibility 누수 방지
- 모든 Phase 완료 후 통합 검증과 최종 종료 판단

각 Phase는 독립적인 Goal 문서와 실행 기록을 소유한다. Master Goal은 gameplay, data, save 또는 wire 계약의 최종 source of truth가 아니다.

## 사용자-visible 문제

현재 직원 도메인은 authored definition, 채용 소유권, 직원 condition, 신뢰·기억이 기존 단일 `Employee` 구조와 여러 legacy 필드에 얽혀 있다. 네 계약 경계를 한 번에 수정하면 검증 범위가 지나치게 넓어지고, 반대로 결합된 항목을 더 잘게 나누면 구형·신형 authority가 동시에 존재할 위험이 있다.

사용자가 보게 될 최종 결과는 다음과 같다.

- 모든 직원의 authored identity와 Profile이 하나의 정의 카탈로그에서 일관되게 제공된다.
- 채용 후보가 seed와 checkpoint에 따라 결정적으로 순환하고 중복 소유·채용되지 않는다.
- 신체 손상과 트라우마가 전투, 치료, 사망, rollback과 모든 참여 가능 여부의 단일 근거가 된다.
- 성격은 authored Profile에, 신뢰와 의미 기반 기억은 run-local Psychology에 일관되게 보존된다.

## 감사 Finding ID

없음. 사용자 요청과 `employee_domain_implementation_difficulty_audit`에서 도출된 구현 Master Goal이다.

## 대상 저장소와 경로

- Core와 Master/Phase 문서: `/mnt/f/work/simulator/core_enhanced`
- production live data: `/mnt/f/work/simulator/game_resources`
- game_server: `/mnt/f/work/simulator/game_server`
- 접근 가능한 Unity 소비자와 canonical 문서: `/mnt/f/unity projects/ark`
- 승인 정책 및 난이도 근거: `docs/decisions/employee_domain_policy_2026-07-17.md`
- 실제 코드 감사 기록: `docs/goals/employee_domain_implementation_difficulty_audit/`

## 현재 Runtime 및 Live Data 근거

2026-07-18 난이도 감사가 확인한 baseline 구조는 다음과 같다.

- starter와 recruitment 직원은 별도 RON 및 별도 DB에 있고, 공통 authored identity uniqueness가 없다.
- Core `Employee`가 run UUID, health/Run HP, life state, availability, trauma, injury, loadout, fragments, profile과 trust를 함께 소유한다.
- HQ 후보는 node seed로 catalog를 매번 shuffle/truncate하며, finite run queue나 definition dedupe가 없다.
- 무기 적용 단계가 기본 공격 주기를 직접 덮어쓰며 직원 고유 공격 속도 단계가 없다.
- personality는 trust 내부에서 employee UUID modulo로 생성된다.
- battle attempt-local roster, RuntimeUnit instance 처리 ledger, staged result commit과 entry snapshot retreat 기반은 이미 존재한다.
- Event, loadout, consumable, deployment와 Unity가 서로 다른 life/health/availability 조건을 사용한다.
- checkpoint는 in-memory clone payload이고 roster는 포함하지만 recruitment queue는 없다.
- game_server는 Core DTO를 주로 직렬화·전달하며 별도 직원 의미를 계산하지 않는다.
- Unity는 legacy health, life state, availability와 candidate `id`를 직접 읽고 일부 affordance를 로컬에서 추론한다.

각 Phase는 착수 시 관련 파일과 dirty 상태가 변경되었는지 다시 조사해야 한다. 이 Master의 baseline을 미래 구현 사실로 간주하지 않는다.

## Source of Truth와 소유 계층

목표 소유권은 다음과 같다.

| 계약 | 목표 source of truth | 소유 계층 |
| --- | --- | --- |
| authored 직원 정체성과 Profile | 통합 `EmployeeDefinition` catalog | Core data/schema 및 strict loader |
| run 내 미합류·세션·합류 소유권 | `RecruitmentQueue` / active session / roster exactly-one state | Core run/world state |
| 직원 condition과 목적별 eligibility | physical damage, trauma 및 중앙 evaluator | Core employee/world/battle state |
| 전투 시도 중 condition | attempt-local employee state와 instance-idempotent ledger | Core battle/world transaction |
| 신뢰와 기억 | run-local Psychology aggregate | Core employee run state |
| public projection | typed Core DTO | Core producer; game_server는 transport |
| presentation | Core 계약을 소비하는 Unity reader/store/presenter | Unity consumer; gameplay 의미 재추론 금지 |

우선순위는 실제 runtime mutation, production-loaded live data, typed DTO/serialization, server mapping/ordering, Unity 소비자, canonical 문서, Goal history 순서로 확인한다. 현재 구현을 승인 정책으로 자동 승격하지 않는다.

## 공통 불변식

- Core가 직원 identity, ownership, condition, eligibility와 psychology mutation의 authority다.
- Unity와 game_server는 Core 계약을 다시 추론하거나 별도 gameplay authority를 만들지 않는다.
- 한 run에서 같은 employee definition은 Queue, active session, roster 중 정확히 한 곳에만 존재한다.
- `employee_definition_id`는 authored identity이고 `employee_uuid`는 roster 합류 후 run-local identity다.
- retreat와 checkpoint load는 해당 시점의 승인된 run-local 직원 상태를 정확히 복원한다.
- battle result commit과 일반 사망 자산 회수는 부분 적용되지 않는다.
- 파생 상태명, eligibility와 trust band를 중복 mutable state로 저장하지 않는다.
- 승인되지 않은 compatibility, fallback, dual-read, dual-write 또는 schema alias를 만들지 않는다.
- Phase가 제거 대상으로 지정한 legacy code/data/DTO/fixture/mock/test/document는 그 Phase 완료 전에 함께 제거한다.
- 기존 사용자 변경과 범위 밖 dirty 파일은 보존한다.

## 승인된 정책

정책의 상세 수치와 의미는 `docs/goals/employee_domain_implementation_difficulty_audit/PLAN.md`의 `Approved Policies` 및 `docs/decisions/employee_domain_policy_2026-07-17.md`의 2026-07-18 승인 결정을 따른다.

Master가 고정하는 핵심 방향은 다음과 같다.

- 통합 authored 직원 catalog와 definition/run identity 분리
- exactly-one recruitment ownership과 seed/checkpoint 결정성
- personality 두 축, 가시성용 `CombatAptitude`, authored base stats와 공격 속도 percent
- specialties production 미추가
- Run HP, generic injury와 mutable availability 제거
- physical damage 0~6 및 trauma 0~100의 독립 condition
- Breakdown은 사망이 아니며 physical damage 6만 사망
- attempt-local 전투불능 누적, same-battle redeploy, result commit 및 retreat rollback
- 중앙 목적별 typed eligibility
- 명시 Treatment만 physical damage를 회복하고 기존 Rest/SavePoint/Gate는 trauma-only
- 일반 사망 자산 원자 회수와 사망 roster 기록 유지
- run-local trust score/band 및 의미 기반 merged memory
- high-risk/refusal/modifier/dialogue 콘텐츠는 보류
- 구형 계약 compatibility 없음

## 미결정 정책

현재 네 Phase를 시작하는 데 필요한 사용자-visible 정책은 모두 결정되어 있다.

다음은 미결정이 아니라 명시적 비범위 또는 후속 authoring이다.

- 직원별 최종 스탯, personality 조합과 aptitude 배정
- 실제 physical Treatment 수치와 특별 trauma override 수치
- trust 기반 high-risk/refusal, 전투/Event modifier와 dialogue/reaction 콘텐츠
- account memorial/legacy와 run 밖 영구 직원 기록

Phase 구현 중 위 항목이 없으면 승인된 완료 조건을 증명할 수 없다는 사실이 발견되면 임의 값을 정하지 않고 `policy_decision_required`로 전환한다.

## Master 범위

- 네 Phase의 순서, 경계, 의존성과 상태 관리
- 각 Phase 시작 시 독립 Goal 문서 초기화 여부 확인
- 이전 Phase 완료 증거와 legacy 제거를 다음 Phase 진입 전에 검토
- Phase 사이 DTO/data/checkpoint/Unity 계약 일치 확인
- 마지막 Phase 후 combined blast radius broad validation
- canonical 문서와 실제 최종 계약 일치 확인
- Master 최종 완료 보고

## 구현 범위

Master 자체의 구현 범위는 Phase orchestration 문서, Phase 전환 게이트와 최종 통합 검증 기록으로 한정한다. Production 구현은 하지 않는다.

각 Child Goal의 구현 범위는 해당 Phase의 책임 경계와 사용자-visible 완료 조건 안에서 Core runtime, live data, typed DTO, game_server transport, 접근 가능한 Unity 소비자, fixture/test와 canonical 문서를 완결된 계약으로 함께 전환하는 것이다. Child가 시작되면 실제 조사 결과를 근거로 그 Child PLAN에 구체적인 파일·mutation·migration 범위를 기록한다.

## 명시적 비범위

- Master 문서에서 직접 production 코드, live RON, DTO, save, server 또는 Unity 구현
- 네 Phase 밖의 Dice System 또는 employee-check Event 구현
- 보류된 trust 콘텐츠나 최종 밸런스 authoring
- account memorial/legacy
- 외부 durable save/database가 발견될 경우 승인 없는 migration 정책 결정
- 관련 없는 리팩토링, 콘텐츠 정리 또는 dirty 파일 수정

## Phase 지도

| Phase | Child Goal | 난이도 | 선행 조건 | 상태 |
| --- | --- | --- | --- | --- |
| 1 | `employee_definition_profile_contract` | 높음 | 없음 | complete |
| 2 | `employee_recruitment_queue_ownership` | 높음 | Phase 1 complete | complete |
| 3 | `employee_condition_lifecycle_migration` | 매우 높음 | Phase 1·2 complete | complete |
| 4 | `employee_psychology_state_contract` | 높음 | Phase 1·2·3 complete | complete |

`not_started`는 Phase 표에만 사용하는 진행 표기이며 Goal Status가 아니다. Phase Goal이 생성되면 그 문서는 허용된 Status인 `active`, `scope_decision_required`, `policy_decision_required`, `externally_blocked`, `complete` 중 하나를 사용한다.

### Phase 1 — EmployeeDefinition/Profile 계약

Child Goal: `employee_definition_profile_contract`

책임 경계:

- starter/recruitment 정의를 하나의 strict `EmployeeDefinition` catalog로 통합
- `employee_definition_id`와 `employee_uuid` 역할 분리
- starter eligibility, role/background, loadout과 roster profile projection
- authored personality 두 축과 `CombatAptitude`
- 직접 authored base stats 및 `basic_attack_speed_percent`
- weapon base interval 뒤 직원 percent를 적용하는 stat pipeline
- candidate/roster typed DTO, server serialization과 Unity reader migration
- specialties가 production schema/runtime/DTO/save에 없음을 유지

사용자-visible 완료 조건:

- starter, recruitment와 roster가 같은 definition identity와 Profile을 표시한다.
- duplicate definition ID와 필수 Profile 누락이 startup에서 명시적으로 거부된다.
- 직원 공격 속도가 승인된 공식으로 무기 기본 공격 주기에 적용된다.
- Unity는 legacy candidate/Profile shape나 UUID-derived personality를 사용하지 않는다.

완료 게이트:

- 통합 catalog가 유일한 authored 직원 source다.
- 구형 두 candidate DB/RON 계약과 UUID trait 생성이 제거됐다.
- 실제 Core JSON 생산자와 접근 가능한 Unity 소비자가 같은 shape를 사용한다.
- focused 및 영향 범위 broad 검증, live data strict loading과 canonical 문서 갱신이 완료됐다.

### Phase 2 — RecruitmentQueue 소유권

Child Goal: `employee_recruitment_queue_ownership`

책임 경계:

- 미합류 definition의 run-seeded finite queue 초기화
- 미선택 starter의 queue 편입
- Queue → active recruitment session → roster exactly-one ownership
- reroll 방지, 미선택 후보의 tail 복귀와 definition dedupe
- 다른 HQ 행동, reconnect, checkpoint 및 Floor carry-forward 복원
- recruitment snapshot/command와 Unity 소비자 migration

사용자-visible 완료 조건:

- 같은 run/seed/checkpoint에서 후보 순서가 안정적이고 반복 요청으로 reroll되지 않는다.
- 선택하지 않은 후보가 승인된 순서로 다시 순환한다.
- 같은 definition은 중복 채용되거나 Queue/Session/Roster에 중복 존재하지 않는다.
- HQ 이탈·재접속·checkpoint load·Floor 전환 뒤에도 소유권이 정확히 복원된다.

완료 게이트:

- Phase 1 definition identity만 queue key로 사용한다.
- 모든 정상·오류·중단 전이가 exactly-one invariant를 보존한다.
- queue/session이 checkpoint와 필요한 carry-forward payload에 포함된다.
- Core producer와 Unity 소비자가 동일한 recruitment contract를 사용한다.

### Phase 3 — Employee Condition 생명주기 Migration

Child Goal: `employee_condition_lifecycle_migration`

책임 경계:

- Run HP, generic injury, mutable availability/life-state 중복 제거
- physical damage 0~6과 trauma 0~100, 파생 damage level/Breakdown
- Starting Battle HP 공식
- RuntimeUnit instance-idempotent incapacitation과 same-attempt redeploy
- death prevention, 일반 사망과 equipment/shared fragment 원자 회수
- physical Treatment와 trauma-only recovery
- 목적별 central eligibility, typed reason priority와 no-deployable recovery/failure
- battle result diff/commit, RetreatBattle rollback, checkpoint와 consumable/admin migration
- Core DTO, server serialization과 모든 접근 가능한 Unity condition 소비자 migration

사용자-visible 완료 조건:

- physical damage와 trauma가 승인 구간, 표시와 모든 참여 가능 여부에 일관되게 반영된다.
- 반복 전투불능, 재배치, 결과 commit과 전투 포기가 같은 attempt-local condition을 사용한다.
- 사망 방지, 사망 자산 회수, 치료와 no-deployable 흐름이 부분 적용 없이 동작한다.
- Unity는 Run HP/legacy availability를 재추론하지 않고 Core typed eligibility를 따른다.

완료 게이트:

- condition lifecycle 전체에서 legacy와 신규 authority가 공존하지 않는다.
- 전투 시도, 결과 commit, 전투 포기와 checkpoint 각각에 상태 전이·원자성 증거가 있다.
- live RON의 obsolete Run HP 관련 schema/content가 정리되고 invalid 구형 data가 거부된다.
- 실제 producer-consumer JSON과 Core/server/Unity focused 및 broad 검증이 통과한다.

Phase 3은 크지만 별도 Goal로 분할하지 않는다. Child PLAN 안에서 조사·구현·검증 milestone을 세분화할 수 있으나, 모든 lifecycle cutover가 끝나기 전에는 Phase를 `complete`로 처리하지 않는다.

### Phase 4 — Employee Psychology State 계약

Child Goal: `employee_psychology_state_contract`

책임 경계:

- trust 내부 UUID-derived trait 제거와 Phase 1 personality 참조
- trust score -100..100, 초기 0과 파생 band persistence
- typed memory kind/subject, 의미 identity, occurrence count, intensity, 최초/최근 발생과 optional expiry
- authoritative source dedupe 및 central memory resolver
- recent reaction과 durable memory 분리
- 사망 roster record, checkpoint/snapshot/DTO와 Unity psychology reader
- dormant high-risk/refusal/modifier/cue surface가 active gameplay/public contract로 노출되지 않음을 확인

사용자-visible 완료 조건:

- 직원 성격은 definition에서, 신뢰도와 기억은 run-local Psychology에서 일관되게 보인다.
- 같은 authoritative 사건의 중복 command/reconnect가 기억을 중복 생성하지 않는다.
- 반복 사건은 승인된 semantic key로 병합되고 사망 직원의 run 기록에도 남는다.
- 보류된 trust 콘텐츠가 활성 gameplay나 public 계약처럼 노출되지 않는다.

완료 게이트:

- personality, trust score/band와 memory의 소유 계층이 분리되고 중복 authority가 없다.
- merge/dedupe/expiry와 checkpoint/death persistence가 실제 상태 전이 테스트로 증명된다.
- Core DTO와 Unity consumer가 같은 Psychology 계약을 사용한다.
- legacy append-only/UUID trait 및 dormant public surface가 정리됐다.

## 수행 순서와 의존성

```text
Phase 1: EmployeeDefinition/Profile
  -> Phase 2: RecruitmentQueue ownership
  -> Phase 3: Employee Condition lifecycle
  -> Phase 4: Employee Psychology state
  -> Master final integration review
```

순서는 고정한다.

1. Phase 1이 definition identity와 authored Profile을 고정한다.
2. Phase 2가 그 identity를 run ownership key로 사용한다.
3. Phase 3이 안정된 EmployeeRunState와 checkpoint 위에서 가장 큰 condition cutover를 수행한다.
4. Phase 4가 Phase 1 personality와 Phase 3 death/checkpoint lifecycle 위에서 Psychology persistence를 고정한다.

이 순서를 바꾸거나 두 Phase를 동시에 production 구현하지 않는다. 순서 변경이 필요해지는 새로운 runtime 근거가 발견되면 Master에 근거를 기록하고 사용자 승인을 받는다.

## Phase 문서 시작 및 재개 절차

아직 시작하지 않은 Child Goal 디렉터리는 미리 만들지 않는다. Phase를 실제 시작할 때 다음 세 문서를 생성한다.

- `docs/goals/<child_goal>/PLAN.md`
- `docs/goals/<child_goal>/EXPERIMENTS.md`
- `docs/goals/<child_goal>/EXPERIMENT_NOTES.md`

각 Child Goal은 사용자가 제공한 `단일 Goal 조사·구현·검증 지침`을 독립적으로 준수한다. Master 요약만으로 코드 수정이나 migration을 시작하지 않는다.

Phase 시작 전에는 다음을 수행한다.

1. Master의 세 문서를 처음부터 끝까지 읽는다.
2. 직전 Phase의 세 문서와 완료 증거를 읽는다.
3. 현재 Phase 디렉터리와 문서 존재/완전성을 확인한다.
4. 신규 Phase라면 관련 저장소 dirty 상태와 경로 충돌을 기록하고 세 문서를 먼저 만든다.
5. 현재 runtime, live data, DTO, server, Unity와 canonical 문서를 다시 조사한다.
6. production 변경 전 dirty baseline과 관련 테스트 baseline을 확보한다.

기존 Phase를 재개할 때는 그 Phase의 세 문서를 먼저 끝까지 읽고 마지막 기록 이후부터 이어서 수행한다.

## Phase 전환 게이트

다음 Phase는 선행 Phase에 대해 아래 조건을 모두 확인한 뒤 시작한다.

- Child Status가 `complete`다.
- 모든 사용자-visible 완료 조건과 불변식에 실제 증거가 연결됐다.
- 필수 focused/broad 검증과 실제 test 수가 기록됐다.
- producer와 접근 가능한 consumer가 같은 승인 계약을 사용한다.
- 제거 대상 legacy와 승인되지 않은 compatibility가 남지 않았다.
- canonical 문서가 실제 계약과 일치한다.
- 기존 baseline 실패와 새 실패가 구분됐다.
- dirty worktree의 사용자 변경이 보존됐다.
- 다음 Phase가 의존하는 DTO/data/checkpoint shape가 안정됐다.

선행 Phase가 `scope_decision_required`, `policy_decision_required` 또는 `externally_blocked`이면 다음 Phase를 시작하지 않는다.

## DTO Migration

- Phase 1: candidate, roster Profile과 identity DTO를 complete cutover한다.
- Phase 2: recruitment queue/session snapshot과 command result를 complete cutover한다.
- Phase 3: condition, eligibility, battle result/recovery projection을 complete cutover한다.
- Phase 4: Psychology score/band/memory projection을 complete cutover한다.

각 Phase는 실제 생산자와 접근 가능한 모든 소비자를 같은 Phase에서 전환한다. 승인 없는 alias, optional fallback 또는 dual schema는 사용하지 않는다.

## Data 또는 Live RON Migration

- Phase 1: starter/recruitment data를 통합 EmployeeDefinition catalog로 migration한다.
- Phase 2: authored data 변경은 원칙적으로 없고 run queue 초기화가 Phase 1 catalog를 소비한다. 조사 결과 live schema가 필요하면 Child 범위 안에서 근거를 기록한다.
- Phase 3: Run HP 관련 policy/consumable/effect schema와 physical Treatment data를 승인 계약으로 complete cutover한다.
- Phase 4: content-driven Psychology memory definition이 실제로 필요할 때만 승인 범위 안에서 추가한다. 보류된 dialogue/modifier 콘텐츠를 만들지 않는다.

구형 live RON을 조용히 수용하지 않고 strict startup rejection을 검증한다.

## Save 및 Checkpoint Migration

현재 확인된 persistence는 Core in-memory run checkpoint다.

- Phase 1: roster가 definition identity/Profile 참조를 보존한다.
- Phase 2: Queue/Session/Roster ownership과 seed/order를 보존한다.
- Phase 3: physical damage, trauma와 condition transaction state를 보존하고 legacy Run HP 등을 제거한다.
- Phase 4: trust score와 semantic memory를 사망 record까지 보존한다.

외부 durable save/database 또는 배포된 payload가 발견되면 현재 지원 대상과 migration 정책을 임의로 정하지 않는다. 완료에 필수면 해당 Child를 `policy_decision_required` 또는 `externally_blocked`로 전환한다.

## 필수 Focused 검증

각 Phase의 PLAN은 조사 결과에 따라 정확한 focused 명령을 확정한다. 최소 계약 증거는 다음과 같다.

- Phase 1: strict employee data loading, duplicate/missing field rejection, stat/attack interval pipeline, starter/roster exact JSON과 Unity contract
- Phase 2: queue determinism/ownership, session/HQ/reconnect/checkpoint/Floor integration, duplicate recruitment negative case와 Unity flow
- Phase 3: condition/eligibility unit tests, repeated incapacitation/redeploy/result/RetreatBattle integration, death asset atomicity, recovery/failure, live RON rejection, server DTO와 Unity flows
- Phase 4: trust band, memory merge/dedupe/expiry, checkpoint/death persistence, exact JSON과 Unity psychology contract

각 focused 검증은 해당 Phase의 사용자-visible 완료 조건, 상태 전이, exact serialized shape와 producer-consumer 일치를 직접 증명해야 한다.

## 필수 Broad 검증

- 각 Child는 자신의 최종 blast radius에 맞는 Core, live data, game_server와 Unity broad 검증을 수행한다.
- Master final integration review는 네 Phase 완료 후 최종 runtime/data/DTO/server/Unity 조합을 대상으로 combined broad validation을 별도로 수행한다.
- broad validation은 production 변경 전 dirty baseline과 비교해 기존 실패와 새 실패를 구분한다.
- 실제 실행된 test 수와 pass/fail/skip 수를 runner 출력 그대로 기록한다.

Master 최종 검증은 네 Phase의 focused 증거를 다시 실행한 것으로 대신하지 않는다. 최종 runtime/data/DTO/server/Unity 조합을 대상으로 영향 범위 broad validation을 별도로 수행한다.

테스트 0개, compile-only 성공, mock끼리만 맞는 테스트, 실제 Core producer를 사용하지 않는 cross-layer 테스트 또는 의미 확인 없는 snapshot 갱신은 완료 증거가 아니다.

## 생략할 검증과 근거

- Master 문서 초기화 단계의 Rust/game_server/Unity tests: production behavior/schema를 변경하지 않았으므로 생략한다.
- Master 문서 초기화 단계의 formatter: Markdown 문서만 생성했고 관련 없는 dirty 파일을 포맷하지 않기 위해 생략한다.
- 각 Child에서 현재 계약과 무관한 저장소·계층 검증: 관계없다는 실제 조사 근거를 Child 문서에 기록한 경우에만 생략할 수 있다.
- 필수 사용자-visible 완료 조건이나 producer-consumer 일치를 증명하는 검증은 생략할 수 없다.

## Canonical 문서

각 Phase는 구현이 안정된 후 변경 책임에 가장 가까운 canonical 문서를 갱신한다.

- gameplay 직원 의미: `docs/game_rulebook.md`
- Core runtime, transaction 또는 rollback: `docs/core_runtime_contract.md`
- live RON/loading: `docs/data_loading_contract.md`
- Unity DTO/reader: Unity `docs/unity_core_contract.md`
- server transport 또는 ordering 변경이 있다면 해당 server canonical 문서

초안 또는 Master/Phase Goal 문서를 durable 계약 대신 사용하지 않는다. 직원 정책 논의 문서 앞부분과 최신 승인 결정의 차이는 각 책임을 처음 구현하는 Phase에서 canonical 계약으로 정리한다.

## 중단 조건

다음 경우 영향받는 수정과 migration을 중단한다.

- 하나의 Phase 안에서 독립적인 source-of-truth 경계가 추가로 발견됨
- 승인되지 않은 Unity DTO, live RON, save, UX, 밸런스, 콘텐츠 또는 compatibility 결정이 필요함
- 필수 외부 consumer/save/service/Unity runner에 접근할 수 없어 완료 조건을 검증할 수 없음
- dirty 사용자 변경과 안전하게 병합할 수 없음
- 선행 Phase의 완료 계약이 실제 runtime에서 깨졌거나 다음 Phase의 전제로 사용할 수 없음
- 이전 Phase legacy를 다시 도입하지 않고는 구현할 수 없는 구조가 발견됨

해당 Child는 근거에 따라 `scope_decision_required`, `policy_decision_required` 또는 `externally_blocked`로 전환한다. Master는 다음 Phase로 진행하지 않고 필요한 사용자 결정이나 환경 변화를 보고한다.

## Compatibility

Compatibility, fallback, dual-read, dual-write, schema alias와 unknown-value fallback은 기본적으로 허용하지 않는다.

배포된 외부 consumer 때문에 compatibility가 필요하다는 근거가 발견되면 구현 전에 지원 대상, 기간, 종료 조건, 제거 작업, 전용 테스트와 사용자 승인을 기록한다. 승인 전에는 compatibility를 구현하지 않는다.

## Master 사용자-visible 완료 조건

- Phase 1~4가 고정 순서로 각각 `complete`다.
- authored definition, recruitment ownership, condition lifecycle과 Psychology의 source of truth가 서로 분리되고 연결 identity가 명확하다.
- 네 Phase에 포함된 production Core/live data/DTO/server/Unity 소비자가 같은 승인 계약을 사용한다.
- 모든 legacy Run HP/injury/availability/UUID trait/append-only memory 및 구형 candidate/recruitment 계약이 각 소유 Phase에서 제거됐다.
- checkpoint, battle rollback/result, death record와 reconnect가 최종 직원 상태를 일관되게 보존한다.
- 승인되지 않은 compatibility나 보류 콘텐츠가 public/live 계약에 남지 않는다.
- 각 Phase focused/broad 검증과 Master 최종 broad 검증이 통과하고 실제 test 수가 기록됐다.
- canonical 문서와 실제 최종 구현이 일치한다.
- 기존 사용자 변경이 보존되고 네 Phase 밖 변경이 섞이지 않았다.
- 필수 미검증 외부 소비자나 migration 위험이 남지 않았다.

## Master 완료 판단

Phase 4 완료 후 다음을 수행한다.

1. 네 Child Goal의 PLAN, EXPERIMENTS, EXPERIMENT_NOTES와 최종 diff를 재검토한다.
2. 각 Master 완료 조건을 실제 Phase 증거에 연결한다.
3. 최종 코드에서 제거 대상 legacy symbol/schema/fixture/mock/test/document를 검색한다.
4. Core producer, server transport와 Unity 소비자의 최종 serialized shape를 비교한다.
5. 최종 dirty 상태에서 combined broad validation을 실행하고 baseline과 비교한다.
6. canonical 문서와 runtime/live data 계약을 비교한다.
7. 결과를 Master의 EXPERIMENTS와 EXPERIMENT_NOTES에 기록한다.
8. 증거가 모두 충족된 경우에만 Status를 `complete`, Implementation complete를 `true`로 변경한다.

## 현재 실행 상태

- Master 문서 초기화: complete
- Phase 1: complete
- Phase 2: complete
- Phase 3: complete
- Phase 4: complete
- Master final integration review: complete
