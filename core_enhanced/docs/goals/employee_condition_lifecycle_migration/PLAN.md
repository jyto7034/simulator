# Employee Condition Lifecycle Migration

Status: complete

Implementation complete: true

Created: 2026-07-18

Last updated: 2026-07-18

Goal type: `employee_domain_implementation_master` Phase 3 Child Goal

## 목표

직원의 장기 condition source를 physical damage 0~6과 trauma 0~100으로 complete cutover하고, 전투 시도·결과 commit·RetreatBattle rollback·사망 자산 회수·Treatment·목적별 eligibility·checkpoint·Core/server/Unity projection 전체가 같은 condition과 derived state를 사용하게 한다.

## 선행 계약

- Phase 1 `employee_definition_profile_contract`: complete
- Phase 2 `employee_recruitment_queue_ownership`: complete
- Phase 2 review: `docs/goals/employee_recruitment_queue_ownership/COMPLETION_REVIEW.md`
- Master: `docs/goals/employee_domain_implementation_master/PLAN.md`
- 승인 정책: `docs/decisions/employee_domain_policy_2026-07-17.md`

Phase 1의 definition/Profile과 Phase 2의 queue/session/roster ownership을 변경하지 않는다.

## 대상 저장소와 경로

- Core employee/battle/world/resources/DTO/tests/canonical: `/mnt/f/work/simulator/core_enhanced`
- live RON/policy/consumables/map content: `/mnt/f/work/simulator/game_resources`
- game_server transport/tests/docs: `/mnt/f/work/simulator/game_server`
- Unity roster/loadout/item/support/battle/maintenance consumers/tests/canonical: `/mnt/f/unity projects/ark`

## 목표 Source of Truth

- persistent employee condition: roster `EmployeeCondition { physical_damage, trauma }`
- life/availability/breakdown/eligibility: condition과 action purpose에서 중앙 evaluator로 파생
- attempt-local condition: active battle employee attempt roster + processed runtime-instance ledger
- battle result: staged attempt condition diff를 persistent roster에 한 번 commit
- death: central transaction이 condition transition, equipment reverse refs와 shared fragment 회수를 함께 commit
- recovery: typed Treatment만 physical damage, 기존 Rest/SavePoint/Gate는 trauma-only
- rollback/checkpoint: battle entry snapshot과 in-memory run checkpoint
- public projection: typed Core DTO; server는 transport, Unity는 consumer

## 승인된 불변식

- physical damage 범위는 0~6이며 6만 사망이다.
- trauma 범위는 0~100이며 100은 Breakdown이지 사망이 아니다.
- Run HP, generic injury list, mutable availability와 별도 life-state authority를 병행하지 않는다.
- 전술적 RuntimeUnit 전투불능은 runtime instance마다 정확히 한 번 attempt-local condition에 반영한다.
- 같은 battle attempt 안의 재배치는 갱신된 attempt condition을 사용한다.
- committed result만 persistent roster에 반영하고 RetreatBattle은 entry snapshot으로 정확히 복원한다.
- death-prevent는 central death transition 안에서 한 번 소비되고 부분 사망/부분 자산 회수가 없다.
- 일반 사망 직원 record는 roster에 유지하되 모든 부적합 action에서 제외한다.
- 장비와 shared skill fragments는 일반 사망 시 승인 정책대로 원자 회수한다.
- purpose별 eligibility는 한 중앙 evaluator와 typed reason priority를 사용한다.
- compatibility, fallback, alias, dual read/write를 허용하지 않는다.
- Phase 2 recruitment ownership과 stable definition identity를 보존한다.

## 구현 범위

1. current Employee condition, battle attempt/result/retreat, consumable, recovery/failure, checkpoint, DTO/server/Unity 소비자를 최신 dirty tree에서 조사한다.
2. 새 condition aggregate와 derived state/evaluator를 설계하고 focused unit tests를 먼저 고정한다.
3. Run HP, generic injury, mutable availability와 life-state mutable authority를 complete cutover한다.
4. battle start stats, repeated incapacitation, same-attempt redeploy와 result diff/commit을 새 condition으로 전환한다.
5. central death/death-prevent/asset recovery transaction을 구현한다.
6. Treatment와 기존 trauma-only recovery를 명시적으로 분리한다.
7. failure/recovery, Event/loadout/item/deployment/maintenance eligibility를 중앙 evaluator로 전환한다.
8. checkpoint/admin/consumable/live RON과 strict rejection을 migration한다.
9. Core DTO, game_server, Unity 모든 접근 가능한 reader/presenter/test/fixture/canonical을 complete cutover한다.
10. focused/broad tests, legacy 검색과 completion review를 수행한다.

## 명시적 비범위

- Phase 1 authored Profile 재설계
- Phase 2 recruitment queue/session 재설계
- Phase 4 trust/memory Psychology migration
- account memorial/legacy
- 승인되지 않은 특별 Treatment 수치 authoring
- 새 전투 장르나 새 gameplay content

## DTO Migration

필요. legacy Run HP/health/life_state/availability/injuries projection을 승인 condition, derived status와 typed eligibility로 complete cutover한다. 실제 final field name과 nesting은 producer/consumer 조사 뒤 하나로 고정한다.

## Data 및 Live RON Migration

필요. Run HP 기반 policy/effect/consumable schema와 recovery authoring을 새 condition 의미로 전환하고 구형 필드는 strict loader가 거부하게 한다. 승인되지 않은 수치는 임의로 만들지 않는다.

## Save 및 Checkpoint Migration

필요. roster condition, consumable/death asset state와 UUID/inventory reverse reference를 in-memory checkpoint와 battle entry snapshot에서 정확히 복원한다. 외부 durable save가 발견되면 임의 migration하지 않는다.

## 필수 Focused 검증

- physical 0~6, trauma 0~100 clamp/transition과 derived death/breakdown
- 중앙 purpose eligibility와 reason priority
- repeated RuntimeUnit incapacitation idempotence
- same-attempt redeploy가 updated condition 사용
- result commit one-shot과 RetreatBattle exact rollback
- death-prevent single consumption
- 일반 사망 장비/shared fragment 원자 회수와 roster record 유지
- Treatment physical recovery와 Rest/SavePoint/Gate trauma-only
- no-living/no-deployable recovery/failure
- checkpoint/admin/consumable migration과 invalid legacy RON rejection
- exact Core JSON/server transport/Unity reader 및 주요 화면 flow

## 필수 Broad 검증

- Core `cargo test --all-targets`
- game_server `cargo test --all-targets`
- Unity focused/full EditMode와 필요한 PlayMode
- live RON strict load와 relevant generated fixture exact match
- legacy symbol/schema/mock/document search

## 생략할 검증과 근거

아직 없음. 조사로 직접 무관함이 증명된 계층만 기록 후 생략한다.

## 중단 조건

- 승인 수치 없이 구현할 수 없는 physical Treatment가 필수로 드러남
- 외부 durable save migration이 완료에 필수임
- Phase 1/2 stable contract가 깨졌음
- dirty 사용자 변경과 안전하게 병합할 수 없음
- 접근 가능한 필수 Unity/server consumer를 검증할 수 없음
- condition cutover 중 구형/신형 authority 병행 없이는 진행할 수 없음

## 완료 조건

모든 persistent/attempt/rollback/death/recovery/eligibility/persistence transition이 새 condition 하나를 사용하고, legacy authority와 compatibility가 제거되며, Core/server/Unity/live data/canonical 증거가 일치한 뒤에만 complete로 전환한다.

## 완료 결과

- persistent와 attempt-local 직원 상태는 `EmployeeCondition { physical_damage, trauma }` 하나로 전환됐다.
- Run HP, generic injury, mutable life state/availability와 구형 DTO/admin/Unity reader는 제거됐다.
- combat instance-idempotence, result commit, retreat rollback, death-prevent, 사망 자산 회수, Treatment, trauma-only recovery와 checkpoint를 focused tests로 고정했다.
- Core 775, game_server 30, Unity EditMode 240, 일반 PlayMode 57이 통과했고 Rust 생성 fixture 두 개가 checked-in JSON과 정확히 일치한다.
- 상세 증거와 알려진 lint debt는 `COMPLETION_REVIEW.md`에 기록했다.
