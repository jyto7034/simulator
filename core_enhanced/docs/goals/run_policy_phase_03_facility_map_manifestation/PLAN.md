# Facility Map, Assignment, And Manifestation Goal

Status: complete; Complete / High 2026-07-27

Implementation complete: true

Goal name: `run_policy_phase_03_facility_map_manifestation`

Master Goal: `run_persistence_facility_monolith_policy_master`

Master Phase: P03

Created: 2026-07-27

Execution authority: 2026-07-27 사용자 요청

Direct predecessors:
- `../run_policy_phase_01_failure_checkpoint_active_run_persistence/COMPLETION_REVIEW.md`
  — Complete / High
- preferred serial baseline:
  `../run_policy_phase_02_local_compendium_persistence/COMPLETION_REVIEW.md`
  — Complete / High

## Objective

절차적 depth/lane graph와 사후 category repair를 제거하고 live RON authored facility template,
semantic slot/tag, directed connection, deterministic pool과 constrained assignment를 Core authority로
수직 구현한다. 일반 탐사, 조건부 보너스 발현과 BossOmen capstone이 같은 template/navigation/save
계약을 사용하고 Unity가 exact local template resource 위에 Core marker/reveal만 표시하게 한다.

## Execution Sources

- Master P03 registry/scope/exit gate:
  `../run_persistence_facility_monolith_policy_master/PLAN.md` 9, 10.1, 14, 18~22절
- 승인 policy:
  `../../decisions/facility_map_and_node_manifestation_policy_2026-07-25_26.md`
- predecessor reviews:
  `../run_policy_phase_01_failure_checkpoint_active_run_persistence/COMPLETION_REVIEW.md`,
  `../run_policy_phase_02_local_compendium_persistence/COMPLETION_REVIEW.md`
- governance/canonical:
  `../../README.md`, `../../engineering_workflow.md`, `../../goal_completion_review_guide.md`,
  `../../game_rulebook.md`, `../../core_runtime_contract.md`, `../../data_loading_contract.md`
- external Unity canonical:
  `/mnt/f/unity projects/ark/docs/unity_core_contract.md`,
  `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`,
  `/mnt/f/unity projects/ark/docs/ui/screen_spec.md`

## Readiness Result

- Standard Floor 1~3, Endless initial pool, puzzle와 BossOmen capstone의 actual Core template RON 및 exact
  Unity slot/reveal resource는 현재 없다.
- 현재 Unity `NodeMapTemplateCatalog`는 `act_01_floor_a`용 계산형 depth/lane placeholder 하나뿐이다.
- 승인 policy 2.2, 3.2, 3.3, 5.1, 6절과 마지막 handoff는 room 수/topology/weight를 초기 live
  authoring으로 허용하고 추가 사용자 결정을 요구하지 않는다.
- 따라서 누락된 actual template/anchor는 외부 blocker가 아니라 이 phase의 필수 production authoring
  범위다. 빈 placeholder나 fallback으로 완료하지 않는다.

## Approved Boundary

- live RON template가 room slot, allowed category, tags, typed terminal과 directed connection topology를
  소유한다. Runtime node별 독립 edge authority를 저장하지 않는다.
- Core가 pool/schedule, deterministic constrained assignment, navigation/selectability, manifestation와
  capstone map switch를 소유한다.
- 모든 room slot은 Floor 생성 transaction에서 category/kind/content/seed까지 결정한다. 조건부 node는
  concealed 상태로 precommit하고 성공 조건에서 reroll 없이 단조 발현한다.
- Unity-local resource는 exact template/version/slot/anchor/reveal/ForwardOnly visual을 소유하지만
  topology, visibility 조건과 selectable을 계산하지 않는다.
- P04 difficulty reference/data는 만들지 않는다. Floor-generation typed extension seam까지만 제공한다.
- P05의 monolith eligibility나 hidden-boss content는 만들지 않고 generic Shop-tag/capstone 입력 seam만
  제공한다.
- P01 checkpoint/active-run envelope에는 새 run-local facility state를 같은 phase에서 추가한다.
  P02 local compendium을 복사하거나 결합하지 않는다.

## Out Of Scope

- 최종 시설 아트, 연출, 사운드
- certified combat power, difficulty profile/reference와 encounter scaling
- monolith eligibility, wandering merchant exchange, hidden boss identity/content
- AnyOf, item/Event outcome 조건, chained manifestation와 dynamic topology mutation
- 좌표/이미지에서 topology/selectability를 추론하는 Unity logic
- 구 generator fallback, compatibility alias, dual-read/write

## Implementation Plan

1. current Core generator/types/progression/BossOmen/save state, live RON loader/validation, server DTO와 Unity
   placeholder/template/view의 dirty baseline을 감사한다.
2. facility template, pool/schedule, assignment profile와 node-definition reference schema를 live RON strict
   loader/validation에 추가한다.
3. Standard intro/branching/final, Endless initial/cycle, puzzle와 BossOmen capstone actual templates 및
   profiles를 저작하고 cross-reference/topology/assignment feasibility를 검증한다.
4. procedural graph 생성/repair를 template selection과 deterministic constrained assignment로 교체하고
   pool order/cycle/assignment를 run state에 고정한다.
5. authored connection 기반 navigation과 terminal resolution, generic atomic capstone switch를 구현한다.
6. precommitted conditional manifestation 성공/negative/rollback/reconnect state를 구현한다.
7. checkpoint/active-run round-trip에 template pool/order/cycle/assignment/manifestation/current map을
   포함하고 profile boundary 회귀를 검증한다.
8. Unity exact template resources, ForwardOnly/reveal visuals, strict cross-project validator와 marker render를
   구현한다.
9. canonical을 갱신하고 focused/live RON/Core/server/Unity/broad/legacy 검증 뒤 completion review한다.

## Test Requirements

- schema/reference/duplicate/unknown tag/connection/typed terminal와 topology deadlock rejection
- Standard room-count progression, Floor schedule, guardian dominator와 exact category constraints
- Endless stage/cycle/open-ended pool, no-repeat exhaustion과 invalid-template fail-fast
- deterministic full assignment, min/max/exact/allowed-category feasibility와 no repair/fallback
- completed-node transit, Bidirectional/ForwardOnly navigation과 terminal closure
- BossOmen exploration -> Boss/Gate capstone atomic switch, conceal/reveal와 checkpoint rollback
- conditional manifestation all success orders, retreat/failure/cancel negative, idempotence와 no reroll
- checkpoint/active-run full facility state round-trip와 reconnect non-reroll
- Unity exact template/slot/reveal validation, marker/concealed/ForwardOnly rendering
- live RON, Core/server broad, original Unity Editor focused/full와 depth/lane legacy search

## Completion Conditions

- Master P03 exit gate를 모두 충족한다.
- production map topology와 assignment authority가 live authored template/profile 하나로 수렴한다.
- reconnect/load/checkpoint가 template/pool/assignment/manifestation을 재추첨하지 않는다.
- 일반 map, puzzle와 BossOmen capstone이 같은 typed template/navigation mechanism을 사용한다.
- Unity actual resource가 모든 live Core template slot/reveal과 exact 대응하고 fallback coordinate가 없다.
- P04/P05 extension seam은 typed generic input만 제공하며 dangling difficulty/content reference가 없다.
- completion review verdict가 `Complete / High`다.

## Stop Conditions

- policy로 결정되지 않은 user-visible topology/content 의미가 구현에 필수임
- 실제 Unity template/anchor/reveal resource를 만들거나 Editor에서 검증할 수 없음
- overlapping dirty 변경의 의도를 보존하며 안전하게 migration할 수 없음
- checkpoint/active-run에 새 facility 결정 상태를 완전하게 표현할 수 없음
- P04/P05 전용 balance/content를 선행하지 않고 generic seam을 만들 수 없음

정책 결정이 필요하면 Status를 `policy_decision_required`, 외부 asset/runner 차단이면
`externally_blocked`로 바꾸고 영향받는 production migration을 중단한다.
