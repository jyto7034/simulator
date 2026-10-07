# Boss Runtime Implementation Readiness Audit Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: single
Execution Mode: investigation_only
Production Mutation Authorized: no
Implementation Readiness Verdict: conditionally_ready
Completion Audit Verdict: Complete / High
Audit Baseline Date: 2026-08-08
Simulator Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
Unity Start Commit: `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`

## Objective

현재 working tree의 실제 Core, live RON, game_server, Unity consumer와 test를 읽어 승인된 Boss Runtime
정책 `BP-01`~`BP-34`의 구현 가능성, 누락 정책, 기존 리팩터링 후보와의 충돌 및 최소 선행 작업을
근거 기반으로 확정한다.

이 Goal은 Boss Runtime을 구현하지 않는다. 완료 결과는 후속 correctness/refactor Goal과 Boss Runtime
vertical-slice 구현 Goal이 사용할 수 있는 코드 경계 감사, 정책 coverage, dependency 순서와 검증
안전망이다.

## User/Contract-Visible Problem

Boss Runtime 정책은 Phase/Pattern/State/Task, authored route 이탈과 복귀, target lock, event ordering,
atomic transition, RON publication 및 Unity replay까지 하나의 목표 모델로 합의되었다. 그러나 현재
runtime에 이 모델을 직접 삽입하면 다음 위험이 있다.

- 현재 event queue, battle advance와 tactical mutation의 부분 commit 위험 위에 새 transition commit을
  얹을 수 있다.
- authored route, block target, basic attack target과 새 movement/target intent가 competing authority가
  될 수 있다.
- RON publication, setup feasibility와 active checkpoint가 새 Boss catalog/reference를 완전하게 검증·복원하지
  못할 수 있다.
- Core event projection, game_server transport와 Unity replay consumer 사이에 새 duration presentation의
  lifecycle이 누락될 수 있다.
- 기존 리팩터링 후보를 전부 먼저 구현하거나 movement cleanup을 성급히 수행하면 Boss Runtime이 실제로
  필요로 하는 seam을 제거하거나 같은 경계를 두 번 수정할 수 있다.

사용자-visible 목표는 구현 착수 전에 무엇을 먼저 고쳐야 하고, 무엇을 Boss 기반과 함께 만들며,
무엇을 첫 vertical slice 뒤로 미뤄야 하는지 증거로 확정하는 것이다.

## Audit Finding ID

없음. 기존 감사 finding의 correction Goal이 아니라 사용자가 승인한 신규 investigation Goal이다.

## Goal Boundary Choice

이 Goal은 단일 Goal로 수행한다.

- 결과가 하나의 통합된 `Boss Runtime implementation readiness` 판정이다.
- Core/RON/server/Unity는 서로 다른 저장소나 모듈이지만 Boss behavior의 authoring→publication→setup→
  runtime→event/checkpoint→replay라는 하나의 producer-consumer 계약 사슬을 이룬다.
- Production mutation이나 migration을 수행하지 않으므로 각 경계를 독립 구현 Phase로 원자 배포할 필요가
  없다.
- 조사 중 독립적인 여러 구현 campaign으로 분리해야만 완료 결과를 낼 수 있음이 확인되면 이 판단을
  유지하지 않고 `scope_decision_required`로 전환한다. 사용자 승인 없이 Master/Phase Goal을 생성하지
  않는다.

## Approved Policy And Evidence

### Target decision record

- [`boss_behavior_state_machine_policy_notes_2026-08-07.ko.md`](../../tmp/boss_behavior_state_machine_policy_notes_2026-08-07.ko.md)
  의 `BP-01`~`BP-34`가 이번 감사의 승인된 목표 정책이다.
- 위 문서는 temporary target decision record이며 현재 구현을 설명하는 canonical이 아니다.
- 현재 gameplay/runtime/data/wire 계약은 실제 코드·live data와 가장 가까운 durable canonical에서 별도로
  확인한다.

### Approved implementation direction

- Boss behavior는 명시적 RON opt-in이며 일반 route enemy의 기존 authored-route 동작을 바꾸지 않는다.
- 상위 runtime은 `BossEncounterRuntime`과 actor별 `BossActorBehaviorRuntime`의 두 계층을 목표로 한다.
- Phase는 장기 control owner, Pattern은 finite typed subgraph, State는 typed Task의 active owner다.
- 기존 movement, blocking, attack, skill, effect, projectile와 death subsystem은 leaf execution authority를
  유지한다.
- Control transition은 deterministic arbitration 뒤 stage→validate→atomic commit하며 정상 gameplay 실패는
  closed typed result로 처리한다.
- Boss RON은 별도 catalog/bundle publication 경계를 가지며 미구현 enum/action과 invalid reference는
  fail-closed한다.
- Unity는 내부 graph를 재계산하지 않고 기존 battle `events_delta`를 seq/time 순으로 replay한 뒤 checkpoint로
  reconcile한다.
- 첫 구현은 Boss Runtime kernel과 최소 추적형 보스 vertical slice다. 최종보스 3종의 기믹은 확장성 stress
  test이며 이번 감사나 첫 구현의 feature scope가 아니다.

### Existing refactor evidence

- [`core_enhanced_refactoring_audit_master/PLAN.md`](../core_enhanced_refactoring_audit_master/PLAN.md)는
  2026-08-07 current-tree read-only 감사를 완료했다.
- [`FINAL_CANDIDATE_CATALOG.md`](../core_enhanced_refactoring_audit_final_catalog_consolidation/FINAL_CANDIDATE_CATALOG.md)는
  53개 source candidate를 51개 final candidate로 정리했으며 그 자체는 구현 권한이 아니다.
- 이 Goal은 51개 전체를 재감사하지 않는다. Boss 정책과 실제로 교차하는 후보의 현재 evidence와 dependency만
  재확인한다.

## Current Runtime And Live Data Evidence

아래는 Goal 초기화 시점의 preliminary evidence다. 최종 판단으로 재사용하지 않고 실제 current-tree
producer/consumer path에서 다시 확인한다.

- Core battle은 timestamp 기반 event queue와 append-only battle event log를 사용하며 movement는 scheduled
  continuous movement tick으로 진행되는 것으로 알려져 있다.
- DefenseRoute enemy의 현재 진행 authority는 preview/setup route에서 만들어진 per-unit authored movement
  plan인 것으로 알려져 있다.
- `ActiveBattleSession`이 Core execution, deployment/playback state와 Unity delivery cursor를 묶고 world
  advance/tactical handler가 update projection을 만드는 것으로 알려져 있다.
- game_server는 Core battle result를 `command_result`와 `battle_setup_snapshot`/`battle_update` side message로
  분류하는 것으로 알려져 있다.
- Unity `BattleRuntimeState`는 setup snapshot 뒤 `events_delta`를 seq/time으로 replay하고 checkpoint
  `at_seq`에서 reconcile하는 것으로 알려져 있다.
- Unity 프로젝트는 이 Goal 시작 시점에 current Core/game_server 기준으로 stale 상태다. Unity 문서, reader,
  runtime store와 test는 현재 producer 계약의 canonical로 간주하지 않고, start commit에 고정된 stale consumer
  baseline 및 producer-consumer drift evidence로만 읽는다. 실제 stale 범위와 필요한 future synchronization은
  symbol/path 근거와 함께 감사 문서에 기록한다.
- Live RON source publication, generated combat setup validation 및 새 Boss catalog를 넣을 정확한 bundle seam은
  discovery pending이다.
- Phase/Pattern/State/Task runtime, Boss opt-in schema와 Boss-specific presentation event는 현재 존재 여부와
  유사 primitive를 discovery에서 확인한다.

## Source Of Truth And Ownership

### Evidence precedence

1. 현재 working-tree의 authoritative runtime code와 state mutation
2. 해당 runtime이 실제 소비하는 production live RON/data
3. [`README.md`](../../README.md)가 지정하는 가장 가까운 durable canonical
4. 실제 producer type과 game_server mapping, 그리고 stale baseline임을 전제로 한 접근 가능한 Unity
   reader/store/presentation code
5. 실제 runtime/live producer를 사용하는 focused/integration test code와 필요한 read-only probe
6. 완료 Goal과 과거 audit는 supporting evidence만 제공

### Current canonical and stale external consumer evidence to inspect

- [`game_rulebook.md`](../../game_rulebook.md)
- [`core_runtime_contract.md`](../../core_runtime_contract.md)
- [`data_loading_contract.md`](../../data_loading_contract.md)
- [`skill_target_contract.md`](../../skill_target_contract.md)
- stale Unity consumer/contract evidence only: `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- stale Unity consumer/contract evidence only:
  `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`

### Target ownership to validate

- Encounter-wide phase/objective/timer/mechanic ownership: `BossEncounterRuntime`
- Per-actor phase-control/pattern/state/task/target/movement intent ownership: `BossActorBehaviorRuntime`
- Spatial execution and collision/blocking: existing BattleCore movement/blocking authority
- Attack/skill/effect/projectile/damage/death execution: existing typed leaf subsystem authority
- Authoring and publication: separate Boss Runtime RON catalog resolved in an atomic live data bundle
- Durable current battle runtime: Core-only active battle checkpoint/runtime state
- Presentation history/current projection: Core event log plus Unity-facing checkpoint
- Transport envelope, session routing and catch-up cursor: game_server
- Presentation playback and local asset mapping: Unity, without gameplay recomputation

## Invariants

| ID | Invariant |
| --- | --- |
| BRA-I01 | 이 Goal은 production code, live data, test, fixture, canonical, server 또는 Unity artifact를 수정하지 않는다. |
| BRA-I02 | 기존 dirty/untracked 파일과 사용자 변경을 이동·삭제·정리·되돌리지 않는다. |
| BRA-I03 | Boss opt-in이 없는 enemy와 일반 침식 직원의 current authored-route behavior는 후속 구현에서도 보존 대상이다. |
| BRA-I04 | Core가 target, transition, timing, RNG와 event order의 authority이며 Unity가 이를 재계산하지 않는다. |
| BRA-I05 | 기존 movement/blocking/attack/skill/effect/death subsystem을 목표 없이 복제하거나 Boss 전용 parallel executor로 대체하지 않는다. |
| BRA-I06 | Same-time order, Pause/battle time, activation epoch, stale event와 checkpoint/reconnect 결정론을 분리 없이 함께 감사한다. |
| BRA-I07 | 실패 뒤 state/resource/cooldown/intent/event/revision 일부가 남지 않는 atomic boundary를 확인한다. |
| BRA-I08 | Invalid/missing/unimplemented RON은 silent fallback, default inference 또는 no-op으로 수용하지 않는다. |
| BRA-I09 | Internal graph ID와 gameplay presentation ID를 구분하고 Unity checkpoint가 animation timing을 새로 만들지 않는다. |
| BRA-I10 | 기존 refactor candidate의 priority/readiness는 구현 승인이나 Boss 선행 조건으로 자동 해석하지 않는다. |
| BRA-I11 | 최종보스 설계 노트는 표현력 검증 입력일 뿐 v1 kernel에 미래 mechanic placeholder 구현을 요구하지 않는다. |
| BRA-I12 | 조사 중 발견한 독립 결함은 자동 수정·Goal 생성하지 않고 evidence와 disposition만 남긴다. |
| BRA-I13 | Unity 프로젝트와 그 문서·test는 stale consumer baseline이다. 현재 Core/game_server가 여기에 맞는다고 추정하거나 Unity 상태를 current producer 계약의 authority로 승격하지 않는다. |
| BRA-I14 | 대화와 model context는 감사 evidence의 저장소가 아니다. 각 finding, source path/symbol, disposition, 미결 gap은 발견한 조사 단위가 끝날 때 즉시 Goal artifact에 기록하고 다음 조사로 넘어가며, 감사 종료 시점에 한꺼번에 복원하지 않는다. |

## Policy Coverage Matrix

최종 `POLICY_COVERAGE_MATRIX.md`에서는 `BP-01`~`BP-34`를 한 행씩 정확히 disposition한다. 아래 표는
조사 경계와 중복 가능한 정책 묶음을 지정한다.

| Audit boundary | Target policy IDs | Required evidence |
| --- | --- | --- |
| Opt-in, route disposition와 movement-intent seam | BP-01~BP-03, BP-18~BP-21 | current route plan producer, movement planner, block/basic target ownership, path/rejoin query와 result lifecycle |
| Encounter/actor hierarchy와 graph lifecycle | BP-04~BP-17, BP-31~BP-34 | current runtime owner, event scheduling, task/action ownership, same-time microstep 및 cleanup/activation epoch seam |
| Condition, event memory, Task result/timeout와 cooldown | BP-18~BP-27 | target lifecycle, condition/event source, timers, cooldown/resource owner, exhaustive result and stale-event path |
| RON definition, reuse, publication과 setup binding | BP-01~BP-02, BP-06~BP-14, BP-25~BP-29 | source inventory, serde strictness, typed validation, cross-catalog reference, bundle publication and scenario actor binding |
| Atomic execution, event log와 checkpoint | BP-08~BP-09, BP-16~BP-17, BP-22~BP-26, BP-30~BP-34 | BattleCore queue/order, world advance/tactical commit, Core checkpoint, seq/cause/revision and reconnect path |
| Server transport와 Unity replay/presentation | BP-30 and leaf-presentation implications of BP-02~BP-03/BP-09 | Core DTO, server mapping/order/resync, actual Unity parser/store/readiness/cleanup and fixture/test coverage |

각 정책 행은 최소한 다음 disposition 중 하나를 가진다.

- `existing_primitive_sufficient`
- `existing_primitive_needs_correction`
- `new_boss_runtime_foundation_required`
- `coordinate_with_existing_refactor`
- `policy_conflict`
- `unverified_external`
- `not_applicable_to_v1` — 근거와 future owner 필수

## Existing Refactor Candidate Correlation

최소한 다음 후보를 실제 current-tree evidence와 다시 대조한다. 이 목록은 선행 조건 결론이 아니라
필수 교차 감사 시작점이다.

| Candidate | Boss audit question |
| --- | --- |
| CRA-P01-C01/C02 | 새 Boss source/catalog와 authored difficulty publication matrix가 하나의 live source authority에서 검증 가능한가? |
| CRA-P05-C02/C04 | Boss actor binding 및 encounter mechanic reference가 battle start 전 feasibility validation과 typed setup plan에 들어갈 수 있는가? |
| CRA-P06-C01/C02/C03 | terminal outcome, total same-time order와 atomic battle advance가 Boss transition/microstep을 수용하기 전 correction되어야 하는가? |
| CRA-P07-C01/C02 | Boss intent와 tactical command가 final tactical state 및 atomic mutation authority와 충돌하는가? |
| CRA-P07-C03/C04 | movement cleanup이 Boss intent seam을 제거하는지, 또는 vertical slice 뒤로 미뤄야 하는가? |
| CRA-P08-C01/C05 | Boss task가 existing trigger/effect/committed-damage path를 leaf executor로 재사용할 수 있는가? |
| CRA-P09-C03/C05/C06 | target query, delivery usefulness, tile reach와 projectile motion 중 Boss가 재사용할 primitive와 별도 target policy 경계는 무엇인가? |
| CRA-P10-C02/C03/C04 | Boss event/checkpoint projection을 추가할 때 Core DTO authority와 server envelope가 field reauthoring 없이 유지되는가? |

각 교차 후보는 최종적으로 다음 중 하나로 분류한다.

- `prerequisite_before_boss`
- `coimplement_with_boss_foundation`
- `defer_until_after_vertical_slice`
- `independent_existing_wave`
- `policy_or_external_gate`

## In Scope

### Core runtime and tests

- `core_enhanced/src/game/battle/core/**`
- `core_enhanced/src/game/battle/{scenario,event_log,types,enums}.rs` 및 실제 인접 authority
- `core_enhanced/src/game/combat_preview/**`, `combat_setup/**`, `events/combat.rs`
- `core_enhanced/src/game/data/**` 중 live source, PVE/enemy/abnormality/skill 및 cross-catalog validation 경계
- `core_enhanced/src/game/world/{combat,state}.rs`, `behavior.rs`와 실제 battle projection/command path
- 위 경계를 직접 보호하는 module/integration tests와 fixture producer

### Live data

- `game_resources/data/pve/**`
- `game_resources/data/enemies/**`
- `game_resources/data/abnormalities/**`
- `game_resources/data/map/**`
- Boss vertical slice가 재사용할 skill/buff/field-object reference data
- official live source manifest/inventory 및 atomic publication entrypoint

### Server and Unity producer-consumer boundary

- `game_server/src/game/player_game_actor/{messages,state,handlers}.rs`와 관련 transport tests/docs
- `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
- 실제 Unity battle setup/update/event/checkpoint parser, runtime store, presentation readiness/cancellation code와
  관련 EditMode/fixture tests. 정확한 path는 discovery에서 기록한다.

### Design and prior audit evidence

- `BP-01`~`BP-34` target decision record
- `abnormality_skill_design_notes.ko.md`의 최종보스 3종 개념 — stress test only
- 2026-08-07 refactor Master/Phase/Final artifacts 중 Boss 경계에 직접 관련된 candidate/test evidence

## Out Of Scope

- Boss Runtime production code, RON schema/data, event DTO 또는 Unity presentation 구현
- 기존 51개 refactor candidate 전체의 재감사 또는 구현
- unrelated run/map/persistence/economy/VN/reward cleanup
- 최종보스 3종의 mechanic, balance, art, VFX, animation과 실제 콘텐츠 구현
- RON 전용 authoring/editor program 설계·구현
- future `RepathToProtectedObjective`, event-log pruning, LOS/stealth/dynamic-nav와 roaming player target 구현
- arbitrary scripting/expression VM, generic blackboard, unrestricted task registry 선행 설계
- public API/save/wire compatibility 또는 migration 정책의 신규 결정
- Git history 변경, commit, stash, format와 worktree cleanup

## Investigation And Artifact Plan

### 0. Durable incremental evidence protocol

- 대화 기록과 model context는 compact되거나 유실될 수 있는 transient working memory로 취급한다. 아직 문서에
  기록되지 않은 대화상 관찰이나 결론은 감사 evidence로 인정하지 않는다.
- 하나의 bounded investigation unit은 한 source/consumer path, 한 policy 행, 한 refactor candidate 또는 한
  test family 검토를 뜻한다. 각 unit에서 새 finding을 확인하면 다음 unit으로 넘어가기 전에 근거 path/symbol,
  관찰, current/target 구분, disposition과 남은 gap을 관련 Goal artifact에 즉시 누적 기록한다.
- 아직 최종 disposition이 정해지지 않은 관찰도 `EXPERIMENT_NOTES.md`에 즉시 기록하고 `unresolved`로
  표시한다. 결론이 정해지면 `BOUNDARY_AUDIT.md`, `POLICY_COVERAGE_MATRIX.md` 또는
  `IMPLEMENTATION_READINESS_REPORT.md`에 승격·연결하며, 최초 관찰 근거를 대화에서 다시 구성하지 않는다.
- 명령이나 probe를 실행한 경우 다음 조사 전에 정확한 command, 목적, 실제 결과와 해석 한계를
  `EXPERIMENTS.md`에 기록한다.
- 대화 compact, 세션 중단 또는 담당자 변경 뒤에는 Goal artifact를 durable checkpoint로 삼아 마지막 기록된
  unit과 unresolved gap부터 재개한다. 기억에 의존해 미기록 finding을 복원했다고 간주하지 않는다.
- 여러 finding을 감사 마지막에 일괄 작성하는 방식은 허용하지 않는다. 최종 보고서는 이미 누적된 evidence를
  종합하는 단계이며 원시 감사 내용을 처음 기록하는 단계가 아니다.

### 1. Baseline and canonical freeze

- 세 저장소 경계의 start commit, dirty status와 접근 가능한 consumer를 기록한다.
- Unity 프로젝트가 current Core/game_server보다 stale한 외부 consumer baseline이라는 사실을 시작 상태에
  기록하고, Unity evidence마다 확인된 current behavior와 expected future synchronization을 구분한다.
- current canonical, target decision record와 기존 refactor evidence를 서로 다른 문서 등급으로 고정한다.
- 현재 working-tree의 [`game_rulebook.md`](../../game_rulebook.md)를 처음부터 EOF까지 다시 읽는다. 이전
  대화나 과거 감사의 요약을 이번 Goal의 current canonical evidence로 대체하지 않는다.
- 전체 rulebook context를 확인한 뒤 DefenseRoute, authored route와 보호 대상, wave/field condition,
  continuous movement와 blocking, attack/skill/targeting, battle time·Pause, terminal outcome 및
  event/checkpoint/reconnect 관련 절을 Boss 정책과 직접 대조한다. 이 목록을 이유로 다른 rulebook 절의
  교차 계약을 읽지 않고 생략하지 않는다.
- Rulebook 문구를 실제 runtime보다 자동 우선하지 않는다. 관련 항목마다 current code, production live data와
  test evidence를 함께 읽고 일치·drift·target-policy 차이를 `BOUNDARY_AUDIT.md`와
  `POLICY_COVERAGE_MATRIX.md`에 구분해 기록한다.
- 관련 test family를 inventory하고 test 이름이 아니라 실제 setup/action/assertion을 읽는다.

### 2. End-to-end current runtime trace

- Live RON source publication부터 preview/setup/scenario actor spawn까지 추적한다.
- Spawned enemy의 authored route plan, target/block/basic attack와 movement tick authority를 추적한다.
- Battle event scheduling/ordering, world advance/tactical mutation, terminal/finalization과 revision commit을
  추적한다.
- Core event/checkpoint DTO에서 server message와 실제 Unity replay/cleanup까지 추적한다.
- 결과를 `BOUNDARY_AUDIT.md`에 authority→producer→consumer 형태로 기록한다.

### 3. Policy-by-policy implementation readiness

- `BP-01`~`BP-34`를 한 행씩 current runtime/canonical/target policy/test evidence와 대조한다.
- 기존 primitive, 필요한 correctness, 새 Boss foundation, conflict와 external gap을 분리한다.
- 결과를 `POLICY_COVERAGE_MATRIX.md`에 기록한다.

### 4. Refactor intersection and minimum prerequisite slice

- 관련 existing candidate의 현재 evidence를 재확인하고 Boss가 요구하는 seam과 target authority를 비교한다.
- 전체 리팩터링을 선행하지 않고 최소 prerequisite, foundation과 동시 구현, vertical slice 뒤 defer를
  구분한다.
- 공유 stage/validate/commit primitive가 필요해도 서로 다른 transaction lifetime을 하나의 거대 transaction으로
  합치지 않는다.

### 5. Vertical-slice and verification plan

- 최소 추적형 보스가 요구하는 opt-in data, runtime state, movement intent, target query, result, event와
  checkpoint의 최소 end-to-end path를 정의한다.
- 구현하지 않은 최종보스 feature를 placeholder로 추가하지 않고 현재 kernel의 extension point만 stress-test한다.
- Existing test retain/strengthen/merge/remove와 새 test-first safety net을 구분한다.
- 결과를 `IMPLEMENTATION_READINESS_REPORT.md`에 dependency graph와 후속 Goal 경계로 기록한다.

### 6. Independent completion audit

- [`goal_completion_review_guide.md`](../../goal_completion_review_guide.md)를 적용해 별도 pass를 수행한다.
- `COMPLETION_AUDIT.md`에 evidence matrix, cross-component finding, long-term fit, primary verdict와 Status
  transition을 기록한다.

## Required Goal Artifacts

- `PLAN.md`
- `EXPERIMENTS.md`
- `EXPERIMENT_NOTES.md`
- `BOUNDARY_AUDIT.md`
- `POLICY_COVERAGE_MATRIX.md`
- `IMPLEMENTATION_READINESS_REPORT.md`
- `COMPLETION_AUDIT.md`

긴 code listing이나 test log는 붙이지 않는다. 재현 가능한 명령, 요약과 정확한 source path를 기록한다.

## Open Policy Decisions

Goal 초기화 시점에는 없다.

조사에서 목표 정책으로 결정되지 않은 gameplay, RON, checkpoint, wire, compatibility 또는 rollout 의미가
발견되면 다음처럼 처리한다.

- 영향받는 구현 결론만 중단한다.
- 정책과 독립적인 나머지 read-only 조사는 계속한다.
- current runtime/live data/canonical, 선택지, 권장안, migration 영향과 test surface를 기록한다.
- 최종 Goal Status를 `policy_decision_required`로 두고 사용자 결정 전 `complete`로 표시하지 않는다.

## Implementation Scope

없음.

허용되는 write는 이 Goal 디렉터리의 조사·실험·감사 문서뿐이다. Production code/data/test/canonical,
game_server와 Unity 파일은 읽기만 한다. 감사 결과는 별도 승인된 후속 Goal 없이는 구현 권한이 아니다.

## DTO, Data And Save Migration

- DTO/wire migration: not_applicable — 현재/목표 영향과 필요한 future cutover만 감사한다.
- Data/live RON migration: not_applicable — Boss catalog/source/binding 목표 shape와 publication gate만 감사한다.
- Save/checkpoint migration: not_applicable — active battle checkpoint에 필요한 Boss runtime state와 current
  persistence lifetime만 감사한다.
- Event-log version migration: not_applicable — 새 presentation variant가 요구할 version/consumer impact를
  readiness report에만 기록한다.

## Deployment And Compatibility

이 Goal에는 배포가 없다.

후속 구현의 producer-consumer rollout은 discovery에서 다음을 판정한다.

- Core/live data/server/Unity를 하나의 atomic deployment unit으로 볼 수 있는지
- 현재 active battle/save가 새 Boss catalog 또는 event variant를 포함할 수 있는지
- compatibility가 실제로 필요한지, strict cutover가 가능한지

승인되지 않은 dual schema, dual read/write, fallback 또는 wire alias를 추천안에 기본 포함하지 않는다.

## Focused Validation

이 Goal의 필수 검증은 감사 주장 자체의 traceability다.

- `BP-01`~`BP-34` 34개 행의 exact coverage와 중복/누락 검산
- 관련 existing candidate ID와 final disposition의 source artifact 대조
- Authoring→Core mutation→event/checkpoint→server→Unity 경로의 실제 symbol/path 대조
- Relevant test family별 보호 invariant, production reachability와 assertion 의미 검토
- Same-time order, atomic failure/no-mutation, stale activation, setup publication와 replay lifecycle에 대한 existing
  test gap 확인

Static source 대조만으로 concrete evidence gap을 닫을 수 없을 때만 narrow read-only test/probe를 실행한다.
정확한 command와 실제 pass/fail/skip 수는 `EXPERIMENTS.md`에 기록하며 실행 전 최종 filter를 source에서
확인한다. Runner가 수치를 제공하지 않으면 추측하지 않는다.

## Broad Validation

Production 변경이 없는 investigation Goal이므로 전체 `cargo test`, Unity full test suite와 live server
end-to-end probe는 기본 완료 조건이 아니다. Broad green count는 목표 Boss 정책의 구현 가능성을 증명하지
않으며 현재 dirty tree의 unrelated failure를 섞을 수 있다.

조사 중 cross-component runtime fact가 static/test-source evidence만으로 불확실하고 completion verdict에
필수라면 가장 작은 충분한 package/integration runner를 선택해 실행한다. 실행 이유와 실제 결과를 먼저
기록한다.

## Omitted Validation

- Production diff가 없으므로 before/after behavior regression suite는 생략한다.
- Boss Runtime target behavior test는 구현이 없으므로 실행 대상이 아니다. 대신 후속 구현이 먼저 추가할
  failing characterization/contract test를 명시한다.
- Unity rendering/PlayMode visual validation은 presentation 구현 범위가 아니므로 생략한다. Actual parser/store
  contract와 EditMode/fixture evidence는 읽는다.
- Final-boss mechanic simulation과 balance validation은 비범위다.

생략 근거가 discovery에서 더 이상 유효하지 않으면 PLAN을 갱신하고 필요한 validation을 실행한다.

## Completion Conditions

1. `BP-01`~`BP-34`가 각각 정확히 한 행에서 current runtime, current canonical, target policy, code owner,
   test evidence와 implementation disposition에 연결된다.
2. Live RON publication→combat preview/setup→scenario binding→BattleCore spawn/execution→movement/target/task leaf→
   event/checkpoint→game_server→Unity replay의 actual path가 확인된다.
3. Current authority와 target Boss authority가 구분되고 competing authority, missing seam와 reuse 가능한
   primitive가 명시된다.
4. 관련 existing refactor candidate가 `prerequisite`, `coimplement`, `defer`, `independent`, `gated` 중 하나로
   evidence와 함께 분류된다.
5. Boss Runtime 전에 수행할 최소 correctness/refactor 범위와 dependency order가 확정되며 전체 refactor
   catalog를 선행 조건으로 만들지 않는다.
6. 최소 추적형 Boss vertical slice의 end-to-end 구현 경계, 성공·실패·interrupt·checkpoint/replay test-first
   safety net과 후속 Goal 분리가 작성된다.
7. 최종보스 3종 요구를 stress-test해 kernel의 실제 extension point와 v1 비범위를 기록하되 future
   placeholder 구현을 요구하지 않는다.
8. 범위 안 Core/live data/server/Unity consumer가 직접 확인되며 접근하지 못한 필수 consumer가 없다.
9. 발견된 gameplay/RON/wire/save/compatibility 정책 질문이 모두 사용자 결정으로 닫히거나 Status가
   `policy_decision_required`로 남는다.
10. Existing test의 retain/strengthen/merge/remove/unverified disposition과 새 안전망이 실제 invariant에
    연결된다.
11. Production mutation이 없고 초기 dirty/untracked 사용자 상태가 보존된다.
12. 별도 완료 감사가 모든 evidence 행을 검토해 `Complete / High`를 내릴 때만 Status를 `complete`로
    변경한다.
13. Current working-tree `game_rulebook.md` 전체를 EOF까지 읽은 범위가 기록되고, Boss 관련 gameplay
    계약마다 rulebook→runtime/live data/test 대조 결과와 drift disposition이 남는다.
14. 모든 감사 finding과 probe 결과가 bounded investigation unit 직후 durable Goal artifact에 누적되어 있으며,
    대화나 compact 이전 model context에만 존재하는 완료 근거가 없다.

## Completion Audit Evidence Matrix

| ID | Completion condition or invariant | Required evidence | Final status |
| --- | --- | --- | --- |
| BRA-C01 | 34개 정책 exact coverage | `POLICY_COVERAGE_MATRIX.md` row count/ID audit | pass |
| BRA-C02 | End-to-end current path | `BOUNDARY_AUDIT.md` producer/consumer trace | pass |
| BRA-C03 | Current/target authority separation | boundary findings and policy matrix | pass |
| BRA-C04 | Existing refactor intersection | candidate source links and five-way disposition | pass |
| BRA-C05 | Minimal prerequisite slice | readiness dependency graph | pass |
| BRA-C06 | Minimal Boss vertical slice plan | implementation boundary and test-first matrix | pass |
| BRA-C07 | Final-boss stress test without scope expansion | design-note requirement matrix | pass |
| BRA-C08 | Core/server/Unity consumer evidence | actual reader/parser/store/test paths | pass |
| BRA-C09 | Policy questions resolved or correctly gated | `EXPERIMENT_NOTES.md`, Status evidence | pass |
| BRA-C10 | Test safety-net disposition | test-family audit in readiness report | pass |
| BRA-C11 | Production no-mutation and dirty preservation | initial/final scoped status/hash comparison | pass |
| BRA-C12 | Independent completion review | `COMPLETION_AUDIT.md`, `Complete / High` | pass |
| BRA-C13 | Full current rulebook read and code/data drift audit | read range evidence plus boundary/policy-matrix links | pass |
| BRA-C14 | Incremental evidence durability across compact/session interruption | per-unit notes/findings/probe records exist in Goal artifacts before final synthesis; no conversation-only evidence | pass |

## Stop Conditions

- 하나의 readiness 결과로 닫을 수 없는 독립 source-of-truth campaign이 발견되어 Master/Phase 재구조화에
  사용자 승인이 필요함
- 목표 정책으로 결정되지 않은 gameplay, UX, RON, DTO, save/checkpoint, compatibility 또는 rollout 선택이
  readiness 결론을 material하게 바꿈
- 필수 Core/live data/server/Unity 저장소, consumer, test source, 권한 또는 환경에 접근할 수 없음
- Current dirty change가 핵심 runtime path를 중간 migration 상태로 만들어 현재 동작과 목표를 증거로
  구분할 수 없음
- Readiness evidence를 얻기 위해 production mutation, fixture regeneration, formatter 또는 destructive
  command가 필요함
- Target decision record, current canonical 또는 audit 기준이 조사 중 material하게 변경되어 기존 행을
  재평가해야 함
- 사용자 소유 dirty 파일을 수정·이동·삭제해야만 진행할 수 있음

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다. 정책과 독립적인 read-only 조사는 먼저
끝까지 진행하고, 미결 정책과 영향 범위를 evidence로 남긴 뒤 `policy_decision_required`로 보고한다.
