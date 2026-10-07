# DefenseRoute 전술 지휘 Phase 7 — 원자 자리 교대

Status: complete

Implementation Complete: true

Goal Type: phase

Execution Mode: implement_and_verify

Production Mutation Authorized: yes — 이 Phase의 `DR-14` 범위에 한정

Parent Master: [defense_route_tactical_command_master](../defense_route_tactical_command_master/PLAN.md)

Predecessor Audit:
[Phase 6 Complete / High](../defense_route_tactical_command_phase_06_relocation_occupancy/COMPLETION_AUDIT.md)

Phase: 7

Goal Name: `defense_route_tactical_command_phase_07_atomic_swap`

Target Repository: `/mnt/f/work/simulator`

Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## 목표

서로 다른 두 active Player employee의 position과 preview가 지정한 새 facing을 명시적
`TacticalSwapUnits` command 하나로 원자 교환한다. 양쪽 source/destination/effect를 모두 사전 검증하고 전술
포인트 5를 한 번 지불하며 양쪽 employee-local relocation cooldown을 10000ms로 시작한다.

## 승인된 정책과 근거

- Primary policy: `DR-14`
- Target source:
  [DefenseRoute 전술 지휘 목표 정책 결정](../../decisions/defense_route_tactical_command_policy_2026-08-06.md#dr-14--자리-교대)
- Master boundary:
  [Phase 7](../defense_route_tactical_command_master/PLAN.md#phase-7--자리-교대)
- Predecessor relocation/occupancy contract:
  [Phase 6 Complete / High](../defense_route_tactical_command_phase_06_relocation_occupancy/COMPLETION_AUDIT.md)

## 확정 정책

- Swap은 occupied relocation fallback이 아닌 별도 command/result/event/transaction이다.
- 서로 다른 active Player employee 두 명만 대상이다. DefenseObject, enemy, inactive/dead/withdrawn, same employee는
  거부한다.
- 양쪽 모두 Defense, relocation cooldown ready, Hard CC와 unreleased basic windup/skill cast-focus가 없어야 한다.
  Silence와 block은 막지 않는다. FieldWorking integration은 Phase 9가 소유한다.
- 양쪽 current logical/world position을 한 command/revision에서 교환하고 각 destination의 player zone,
  destination weapon affinity, valid/static tile을 검사한다. Occupancy에서는 pair만 제외하며 제3 Player actor가
  있으면 전체 거부한다.
- Command는 양쪽 새 facing을 각각 받는다. 기존 facing이 UX 기본값이지만 Core/server가 facing을 자동 복사하거나
  교환하지 않는다.
- HP/buff/resonance/stance/action cooldown/runtime identity와 unit-owned state는 각 employee에 남는다.
- 양쪽 engagement를 먼저 해제하고 final 위치에서 block/target/range/area와 Ground/Platform tile effect를 batch
  재계산한다. Released delivery는 Phase 6 tracking 의미를 유지한다.
- 성공은 tactical relocation policy cost 5를 command당 한 번 소비하고 양쪽 cooldown을 frozen/current
  battle time+10000ms로 설정한다. Swap 전용 tuning은 없다.
- 모든 preflight 실패는 양쪽 position/facing/effect/resource/cooldown/event/revision을 바꾸지 않는다.

## 유지할 불변식

- 중간 duplicate/empty occupancy, sequential relocation, walking/path/reservation/arrival state를 외부에 노출하지 않는다.
- Phase 6 common destination/occupancy와 same-instance preservation 의미를 축소하지 않는다.
- Ground↔Platform은 destination affinity/profile을 각 employee weapon으로 검사하고 두 tile effect를 batch 전환한다.
- Pause는 clock만 동결하고 swap command는 frozen latest state에서 평상시 eligibility를 검사한다.
- Tactical resource와 cooldown policy는 기존 `tactical_relocation` authority를 재사용한다.
- Unity runtime/contract/tests와 Phase 8~9 placeholder는 비범위다.

## 범위

- Swap command/source/destination preflight와 atomic Core batch mutation
- 양쪽 facing/cooldown/tile effect/Rest epoch/engagement/tracking reconciliation
- World resource/revision/result/capability와 checkpoint/event/server projection
- Rust Any-profile cross-tile fixture와 occupied pair/third actor regression
- Focused/broad/static/canonical/Complete-High audit

## 명시적 비범위

- Relocation fallback/automatic command conversion 또는 occupied-tile UI 구현
- FieldWork assignment/content/batch semantics
- Manual targeting/focus 또는 새 live skill
- Player walking/pathfinding/reservation/pending swap
- Unity runtime/contract/UI/tests
- Swap 전용 cost/cooldown data와 승인되지 않은 compatibility

## 미결정 정책

현재 알려진 사용자-visible 미결정 항목은 없다. Existing Core transaction이 양쪽 effect를 intermediate state 없이
표현할 수 없거나 command identity/version에 승인되지 않은 선택이 필요하면 영향 변경을 중단하고
`policy_decision_required`로 전환한다.

## 구현 계획

1. Phase 6 relocation/effect/occupancy 코드를 batch-safe하게 재사용할 seam과 server/DTO 경로를 조사한다.
2. Swap 전후 engagement/effect/tracking/facing과 실패 원자성 baseline을 확보한다.
3. 두 source와 두 destination을 모두 검증하는 Core command를 구현한다.
4. Intermediate occupancy 없이 body/facing/effect/cooldown을 batch commit하고 한 번 refresh한다.
5. World cost-once/revision-once와 command/result/event/capability/checkpoint/server를 cutover한다.
6. Pause, third occupancy, busy/CC/resource/cooldown/affinity와 Any cross-tile direct regressions를 추가한다.
7. Canonical/legacy/dirty/broad gate와 Complete/High audit을 수행한다.

## 완료 조건

- `DR-14` exact two-employee command, facing, cost-once/cooldown-both 의미가 구현된다.
- 모든 양쪽 source/destination rejection과 전체 transaction immutability가 direct 증명된다.
- Pair exclusion과 third player actor occupancy, no fallback/no intermediate state가 direct 증명된다.
- 양쪽 identity/transient state, engagement/tracking과 Ground/Platform effect가 final positions에서 보존/재계산된다.
- Pause/checkpoint/root/event/server/current policy가 strict current shape와 일치한다.
- Core/server/RON/canonical/static/broad gate와 Complete/High audit이 통과한다.

## DTO migration

완료. Explicit swap command/result/event와 action capability를 같은 release에 추가했다. Existing per-unit
relocation cooldown/eligibility를 재사용하며 Unity는 비범위다.

## Data/live RON migration

없음. Phase 6 required `tactical_relocation(cost, cooldown_ms)`를 유일한 tuning authority로 재사용한다.

## Save/checkpoint migration

불필요. Active battle는 durable save 밖이며 기존 tactical cooldown/checkpoint shape를 그대로 재사용한다.

## 필수 focused 검증

- Success, explicit facings, same unit/non-employee/Attack/HardCC/busy/resource/cooldown rejection와 immutability
- Pause, cost once, both employee cooldown, new runtime redeploy reset ownership
- Pair occupancy exclusion, third employee/defense-object rejection, zone/static/affinity 양방향 검사
- Same instances HP/buff/resonance/stance/action cooldown/released delivery 보존
- Both engagement release/reacquire, range/spatial/tracking and no intermediate duplicate event/state
- Ground↔Platform Any effects, block and Rest cadence batch transition
- DTO/event/checkpoint/root/server/current policy exact mapping와 no duplicate tuning

## 필수 broad 검증

- `cargo test -p game_core`
- `cargo test -p game_server`
- `cargo test -p game_core --test ron_loading`
- `cargo fmt --all -- --check`

## 중단 조건

- 승인되지 않은 source/destination/facing/activity/tuning/version 선택이 필요함
- 양쪽 effect/occupancy를 intermediate state 없이 안전하게 commit할 수 없음
- 필수 Core/server/live data/test runner에 접근할 수 없음
- 선행 Phase 또는 user dirty와 안전하게 병합할 수 없음

## Compatibility

승인된 compatibility는 없다. Strict current command/state/event만 지원한다.

## 완료 감사 Evidence Matrix

| ID | 완료 조건 | 정책 근거 | Runtime/data/contract 근거 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- |
| P7-01 | Explicit swap/cost-once/cooldown-both | `DR-14` | player/Core command, shared relocation policy/state | paused world transaction + Core cooldown rejection | pass |
| P7-02 | Two-source/two-destination preflight | `DR-14` | pair occupancy + both profile/source gates | same/busy/CC/static/third actor/resource immutable cases | pass |
| P7-03 | Atomic spatial/engagement/tracking | `DR-14` | batch helper, release-both, one refresh/event | identity/state and pair success; Phase 6 delivery regressions | pass |
| P7-04 | Batch tile-effect transition | `DR-14` | shared effect/Rest epoch update | Any Ground↔Platform direct swap | pass |
| P7-05 | DTO/event/server/version | Master Phase 7 gate | result/presentation/action/server, event v30 | world DTO/source ID and strict server serde | pass |
| P7-06 | Canonical/legacy/dirty/broad | Master common gate | four canonical contracts, strict current-only shape | Core 898/server 55/RON 24/check/fmt/diff/search | pass |

## 현재 진행 상태

- Phase 6 completion audit 확인: 완료
- Child path/dirty 확인과 문서 초기화: 완료
- Production discovery와 baseline: 완료
- Core/world/DTO/server implementation: 완료
- Focused/strict/static/broad verification: 완료
- Completion audit: [Complete / High](COMPLETION_AUDIT.md)
