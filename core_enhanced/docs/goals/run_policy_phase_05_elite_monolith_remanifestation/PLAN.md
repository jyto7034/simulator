# Elite Monolith Remanifestation Goal

Status: complete

Implementation complete: true

Goal name: `run_policy_phase_05_elite_monolith_remanifestation`

Master Goal: `run_persistence_facility_monolith_policy_master`

Master Phase: P05

Created: 2026-07-27

Execution authority: 2026-07-27 사용자 요청 및 같은 날 범위 선택 `2번`

Direct predecessors:

- `../run_policy_phase_01_failure_checkpoint_active_run_persistence/COMPLETION_REVIEW.md`
  — Complete / High
- `../run_policy_phase_02_local_compendium_persistence/COMPLETION_REVIEW.md`
  — Complete / High
- `../run_policy_phase_03_facility_map_manifestation/COMPLETION_REVIEW.md`
  — Complete / High
- `../run_policy_phase_04_combat_power_floor_difficulty/COMPLETION_REVIEW.md`
  — Complete / High

## Objective

Endless의 자연 Elite를 run당 한 번만 소비하고, 최초 승리로 열린 환상체별 모노리스를 Floor 4+의
떠돌이 상인에게 구매해 일반 Combat node를 2·3단계 강화 재현으로 원자 변환한다. 세 번째 승리의
고유 스킬 파편, 잔여 모노리스 네 개의 다음 Floor 교환 보장과 변질된 모노리스 획득까지를
Core/live RON/server/Unity/persistence에서 하나의 완료 가능한 모노리스 시스템으로 수직 구현한다.

변질된 모노리스는 이 phase에서 run-local dormant key로 끝난다. 실제 hidden boss 전투·관리자 VN·
전용 capstone·최종 보스 완료 interception은 Master P06에 보류하며, P05는 기존 Endless 최종 보스와
일반 완료 흐름을 변경하지 않는다.

## Execution Sources

- Master P05 scope/readiness/exit gate/seams: `../run_persistence_facility_monolith_policy_master/PLAN.md`
  8~10, 16, 19~25절
- Approved gameplay policy:
  `../../decisions/elite_monolith_remanifestation_compendium_policy_draft_2026-07-26.md`
- Facility seam:
  `../../decisions/facility_map_and_node_manifestation_policy_2026-07-25_26.md`
- Compendium seam:
  `../../decisions/local_encounter_compendium_persistence_policy_draft_2026-07-25.md`
- Canonical: `../../game_rulebook.md`, `../../core_runtime_contract.md`,
  `../../data_loading_contract.md`
- External Unity: `/mnt/f/unity projects/ark/docs/unity_core_contract.md`,
  `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`,
  `/mnt/f/unity projects/ark/docs/ui/screen_spec.md`

## Readiness Result

- P01~P04 completion review는 모두 Complete / High다. Checkpoint/active-run, local compendium,
  facility Shop-tag seam과 `MonolithRemanifestation` difficulty role을 재사용할 수 있다.
- Live Shop에는 `unknown_exploration_shop`과 high-value preset이 있으나 현재 offer 4개, refresh 없음,
  typed monolith merchandise 없음이며 승인된 3-slot/1-refresh 계약으로 이관해야 한다.
- Live facility에는 Floor 4+ Endless pool에서 사용할 수 있는 도달 가능한 `shop` tag slot이 있다.
- Live VN에는 모노리스 교환 VN과 Unity-local script cue가 없다. P05-C에서 실제 두 선택 교환 흐름으로
  함께 저작해야 한다.
- Price/weight/refresh cost, stage stat/reward 계수는 승인 decision 12절 범위에서 보수적인 초기 live
  값으로 저작하고 strict validation 및 고정-seed test로 고정할 수 있다.
- 사용자는 모노리스 시스템 전체를 지금 구현하고 실제 보스 구현을 뒤로 미루는 `2번` 범위를
  명시적으로 선택했다.

Readiness: **complete**.

## Approved 2026-07-27 Phase Split

P05가 지금 구현하는 범위:

- 자연 Elite hard exclusion과 pool exhausted ordinary-combat fallback
- Floor 4+ 떠돌이 상인, 정확히 3개인 첫 page, eligible 수에 따른 0/1/2 monolith offer
- 한 번의 유료 item-only 전체 refresh와 high-risk/high-return item pool
- 환상체별 run-local monolith ledger, 한 환상체당 한 번 구매, 3 uses
- NodeConfirm의 Core-projected monolith 선택과 순수 Available Normal Combat 원자 변환
- stage 2/3 강화 재현, 2회 attempt, 승리 progression, 세 번째 승리의 고유 파편 정확히 한 번 지급
- residual ledger, 네 개 threshold, 다음 Floor Shop-tag 및 떠돌이 상인 exactly-once guarantee
- accept/decline 교환 VN, 잔재 네 개 원자 소비와 변질된 모노리스 획득
- 위 모든 state의 checkpoint/active-run round-trip 및 실제 server/Unity 소비자

P06으로 보류하는 범위:

- 일반 FinalBoss pool 재분류와 일반 완료 interception
- 관리자 hidden-boss VN
- `흉탄의 사수` actor, skill, encounter, battlefield/wave와 reward content
- 전용 single-terminal capstone manifestation
- hidden boss compendium entry/producer와 typed special completion

P05에서 획득한 변질된 모노리스는 P06 전까지 소비되지 않고 run 종료 시 다른 run-local state와 함께
소멸한다. 이 dormant-key 경계는 hidden boss placeholder나 불완전한 진행을 live build에 노출하지 않는다.

## Approved Runtime Boundary

- 기능은 Endless 전용이며 떠돌이 상인은 Floor 4+에서만 등장한다.
- 자연 Elite 승리/완전 실패는 같은 run 자연 pool에서 hard exclusion한다. 미방문 배정은 소비가 아니다.
- 자연 pool 소진은 explicit `elite_pool_exhausted` state이며 authored ordinary combat candidates를 쓴다.
- 전용 모노리스는 inventory/equipment/consumable이 아닌 abnormality별 run-local ledger merchandise다.
- 첫 상인 page는 eligible 0/1/2+에 따라 monolith를 0/1/2개 포함하고 합계는 항상 3개다. 두 monolith를
  모두 살 수 있다. 한 번의 유료 refresh는 page 전체를 정확히 3개의 item-only offer로 바꾼다.
- 순수 Available Normal Combat만 변환하며 node/slot/category/topology는 유지하고 content/reward와
  1 use 차감을 원자 commit한다. 한 monolith identity당 미해결 node는 하나다.
- 재현 node는 2회 시도다. 첫 패배는 같은 node 재시도, 두 번째 패배는 authored reward 없이 완료한다.
  Retreat는 attempt나 use를 소비하지 않는다.
- 승리만 stage를 올리며 세 번째 승리는 정확히 한 번 고유 skill fragment를 지급한다.
- 세 번째 승리 때 use가 정확히 1 남았으면 source identity를 보존한 residual이 된다. 0 use여도 파편은
  지급하지만 residual은 만들지 않는다.
- residual 네 개 threshold를 처음 넘으면 다음 Floor Shop-tag template과 떠돌이 상인을 정확히 한 번
  보장한다. 보장된 상인을 방문하지 않아도 guarantee는 소비되며, VN 미시작이면 이후 random merchant가
  교환을 다시 제시할 수 있다.
- 상인 진입 시 네 잔재가 있으면 shop보다 먼저 accept/decline 교환 VN을 한 run 한 번 시작한다.
  decline은 되돌릴 수 없고 accept는 oldest four residual을 원자 소비해 변질된 모노리스 하나를 준다.
  Enkephalin은 주지 않으며 이후 같은 shop을 계속 사용한다.
- 새 run-local 상태는 checkpoint/active-run에 포함하고 `loads_used`와 영구 compendium은 되감지 않는다.
- P04 role profile과 P02 generic compendium producer를 소비하며 별도 power/profile authority를 만들지 않는다.

## Out Of Scope

- Der Freischutz를 hidden boss로 이름만 바꾸거나 기존 encounter를 재사용하는 대체 구현
- hidden boss actor/encounter/skill/wave/counterplay, 관리자 VN, 전용 capstone과 특수 완료
- 일반 FinalBoss 3종 재분류 또는 변질된 모노리스 보유에 따른 기존 Endless completion 변경
- 빈 reward/actor/encounter/VN placeholder 또는 임시 test-only production content
- 결정문에 없는 monolith 판매/분쇄/장착/inventory stack UX
- P01~P04 source-of-truth의 재설계
- 과거 배포 save compatibility 또는 schema alias

## Implementation Plan

1. P05 scope split을 master/child에 고정하고 production 대상 dirty diff와 pre-change baseline을 확보한다.
2. P05-A: 자연 Elite consume/hard exclusion, explicit pool exhaustion, eligible wandering-merchant offers,
   typed merchandise, purchase exclusion과 run-local ledger를 구현한다.
3. P05-A checkpoint/active-run rollback, deterministic offer/refresh, high-value item pool과 duplicate purchase를
   focused 검증한다.
4. P05-B: Core-projected monolith selection, ordinary Combat atomic conversion, 2-attempt node state,
   stage/reward/fragment progression을 구현한다.
5. P05-B cancel/reject/first loss/second failure/victory/checkpoint paths와 P04 role/ordinary quota 분리를
   focused 검증한다.
6. P05-C: residual ledger, next-Floor Shop guarantee, exchange VN, accept/decline와 corrupted key를 구현한다.
7. Core/server/Unity/persistence/canonical을 같은 typed 계약으로 수직 연결하고 기존 final-boss completion이
   변하지 않았음을 고정한다.
8. 모든 새 state의 active-run current/backup/checkpoint round-trip, fixed-seed full Endless flow,
   canonical/legacy/broad completion review를 수행한다.

## Completion Conditions

- Master P05 exit gate가 실제 live Endless 결합 flow로 모두 입증된다.
- Live RON과 Unity local content가 monolith economy, remanifestation과 교환 VN을 실제 authored content로
  소유한다.
- 거래/변환/보상/교환이 partial 또는 duplicate commit되지 않는다.
- 변질된 모노리스가 dormant run-local key로 획득·저장되며 hidden boss나 일반 완료 interception을
  시작하지 않는다.
- Core/server/Unity/persistence/canonical이 같은 typed 계약을 사용한다.
- Completion review verdict가 `Complete / High`다.

## Stop Conditions

- 승인된 모노리스 정책으로 답할 수 없는 새 gameplay/UX/content/save 의미가 발견된다.
- 기존 사용자 변경과 안전하게 병합할 수 없거나 predecessor source of truth를 교체해야 한다.
- Unity runner, host persistence 또는 실제 live content verification을 사용할 수 없다.
- 구현이 P06의 실제 hidden-boss 콘텐츠나 일반 완료 interception을 필요로 한다.
