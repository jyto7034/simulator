# DefenseRoute 전술 지휘 Phase 6 — 전술 이동과 점유

Status: complete

Implementation Complete: true

Goal Type: phase

Execution Mode: implement_and_verify

Production Mutation Authorized: yes — 이 Phase의 `DR-04`, `DR-05`, `DR-12` 범위에 한정

Parent Master: [defense_route_tactical_command_master](../defense_route_tactical_command_master/PLAN.md)

Predecessor Audit:
[Phase 5 Complete / High](../defense_route_tactical_command_phase_05_stance_tile_effects/COMPLETION_AUDIT.md)

Phase: 6

Goal Name: `defense_route_tactical_command_phase_06_relocation_occupancy`

Target Repository: `/mnt/f/work/simulator`

Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## 목표

Defense 상태 active employee의 server-authoritative 즉시 `TacticalRelocateUnit`을 비용 5와 employee별
10,000ms cooldown으로 구현한다. 공통 destination 검증으로 최초/재배치와 relocation의 logical allied
occupancy를 닫고, position/facing/engagement/range/area 포함 관계 및 Ground/Platform derived effect를
중간 중복 없이 원자 전환한다.

## 승인된 정책과 근거

- Primary policies: `DR-04`, `DR-05`, `DR-12`
- Target source:
  [DefenseRoute 전술 지휘 목표 정책 결정](../../decisions/defense_route_tactical_command_policy_2026-08-06.md#dr-04--전술-순간이동)
- Master boundary:
  [Phase 6](../defense_route_tactical_command_master/PLAN.md#phase-6--전술-이동-점유와-tile-effect-원자-전환)
- Predecessor stance contract:
  [Phase 5 Complete / High](../defense_route_tactical_command_phase_05_stance_tile_effects/COMPLETION_AUDIT.md)

## 확정 정책

- Relocation은 같은 active runtime instance의 position/facing을 즉시 바꾸는 별도 command다. 철수/재배치,
  보행/path/segment/reservation/arrival state가 아니다.
- Source는 Defense여야 하고 Hard CC, unreleased basic windup과 skill cast/focus가 막는다. Silence와 block은
  막지 않는다. FieldWork integration은 Phase 9가 소유한다.
- Cost 5, employee별 battle-time cooldown 10,000ms. `DeployCostReduction`은 적용하지 않는다.
- Same position+facing은 no-op reject, same position facing-only는 동일 cost/cooldown으로 허용한다.
- Destination은 player deployment zone, weapon affinity, valid/static tile와 logical allied occupancy를 모두
  검사한다. Enemy overlap은 허용하고 path reachability는 검사하지 않는다.
- Active player employee와 defense-object actor가 slot을 점유한다. Source 자신의 current tile은 제외한다.
- 최초/모든 재배치도 occupied destination을 `position_occupied`로 거부한다.
- Occupied relocation은 자동 push/swap fallback 없이 거부한다. Phase 7 swap command를 선행하지 않는다.
- HP/buff/resonance/stance/identity/action cooldown은 보존한다. 기존 engagement는 해제 후 새 위치에서
  deterministic 재계산하고 target/range/aura 포함 관계를 새 position에서 평가한다.
- Ground↔Platform은 Any weapon만 허용한다. Defense 유지 상태에서 old Focus/Rest 제거와 new effect 적용을
  position mutation과 원자 commit한다. Platform effective block 0, Ground 복귀는 stance/profile로 재계산한다.
- Source 대상 delivery는 relocation으로 취소하지 않는다. Homing/fixed/GroundFixed/FollowCaster/FollowTarget의
  기존 tracking 의미를 유지한다.
- 모든 preflight 성공 뒤 position/facing/resource/cooldown/effect/event/revision을 한 번에 commit한다.

## 유지할 불변식

- Phase 5 stored stance는 Attack/Defense뿐이며 relocation은 Defense를 보존하고 stance cooldown을 만들지 않는다.
- 실패는 position/facing/engagement/effect/resource/cooldown/event/revision 무변경이다.
- Pause는 battle clock만 멈추며 frozen time에서 relocation eligibility를 검사한다.
- Tactical resource는 Phase 3 global authority 하나를 사용한다.
- Phase 1 released delivery, Phase 2 control capability, Phase 4 weapon authority를 보존한다.
- Unity runtime/contract/tests와 Phase 7~9 placeholder는 비범위다.

## 범위

- Relocation state/cooldown/policy/command/result/event/capability/checkpoint/root/server projection
- Common logical destination and occupancy validation for deploy/redeploy/relocate
- Atomic world position/facing, tactical tile/effect, block/target/spatial reconciliation
- Any-profile Rust fixture Ground↔Platform engine contract
- Strict live policy, event/checkpoint version ownership, canonical/focused/broad/audit

## 명시적 비범위

- Swap command/transaction과 occupied-tile UI fallback
- FieldWork assignment/content
- Player walking/pathfinding/reservation/pending relocation
- Any first-live weapon content
- Manual targeting redesign
- Unity runtime/contract/UI/tests
- 승인되지 않은 compatibility

## 미결정 정책

현재 알려진 사용자-visible 미결정 항목은 없다. Existing battlefield spatial mutation이 같은-position
facing-only 또는 area/aura reconciliation을 표현할 수 없거나 version 선택이 필요하면 영향 변경을 중단하고
`policy_decision_required`로 전환한다.

## 구현 계획

1. Position mutation/spatial registry/target/block/action-busy/deployment-zone/occupancy/DTO path를 조사한다.
2. 대상 diff와 focused baseline을 확보한다.
3. Required relocation policy, employee cooldown state와 Core command preflight를 구현한다.
4. Common destination/occupancy validation을 deploy/redeploy/relocate에 적용한다.
5. Position/facing/spatial/engagement/target를 원자 reconcile한다.
6. Ground↔Platform derived effect와 Rest epoch를 원자 전환한다.
7. Checkpoint/root/event/server/version과 strict data를 cutover한다.
8. Focused/broad/static/canonical과 Complete/High audit을 수행한다.

## 완료 조건

- `DR-04`, `DR-05`, `DR-12` exact command/cost/cooldown/occupancy/effect 의미가 구현된다.
- Defense/busy/CC/resource/cooldown/destination/affinity/no-op rejection과 atomic immutability가 direct 증명된다.
- Deploy/redeploy occupancy와 relocation enemy-overlap/source-self/facing-only 규칙이 direct 증명된다.
- Same instance transient state와 released delivery tracking이 보존된다.
- Engagement/target/range/spatial과 Ground↔Platform Focus/Rest 전환이 deterministic direct regression을 가진다.
- Checkpoint/reconnect/root/server/live policy/version이 strict current shape와 일치한다.
- Core/server/RON/canonical/static/broad gate와 Complete/High audit이 통과한다.

## DTO migration

필요함. Relocation command/result/event와 per-unit cooldown/eligibility projection을 같은 release에 추가한다.
Unity는 비범위다.

## Data/live RON migration

필요함. Run policy에 relocation cost/cooldown의 required ownership을 추가하며 alias/default는 없다.

## Save/checkpoint migration

Discovery pending. Active battle는 durable save 밖이지만 checkpoint/event/setup projection 변화와 실제 version
owner를 조사해 strict cutover한다.

## 필수 focused 검증

- Success/facing-only/no-op, Attack/HardCC/busy/resource/cooldown rejection와 immutability
- Pause clock, same employee cooldown, redeploy reset/ownership
- Valid/static/zone/affinity/ally occupied/enemy overlap/source-self destination
- Deploy/redeploy occupied rejection와 race/latest-authority revalidation
- Identity/HP/buff/resonance/stance/action cooldown/released delivery preservation
- Engagement release/reacquire, target/range/spatial and tracking policies
- Ground↔Platform Any effect atomicity, block and Rest cadence reset
- DTO/event/checkpoint/root/server/data/version exact mapping

## 필수 broad 검증

- `cargo test -p game_core`
- `cargo test -p game_server`
- `cargo test -p game_core --test ron_loading`
- `cargo fmt --all -- --check`

## 중단 조건

- 승인되지 않은 occupancy/spatial/tracking/version/tuning 선택이 필요함
- Phase 5 effect state를 안전하게 원자 전환할 수 없음
- 필수 Core/server/live data/test runner에 접근할 수 없음
- 선행 Phase 또는 user dirty와 안전하게 병합할 수 없음

## Compatibility

승인된 compatibility는 없다. Strict current command/state/data/version만 지원한다.

## 완료 감사 Evidence Matrix

| ID | 완료 조건 | 정책 근거 | Runtime/data/contract 근거 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- |
| P6-01 | Relocation command/cost/cooldown | `DR-04` | Core/world command, tactical side state, cost 5/cooldown 10000 | focused success/facing/no-op/gates/Pause/resource tests | pass |
| P6-02 | Common occupancy/destination | `DR-05` | Core logical slot validator used by deploy/redeploy/relocate | employee/defense-object/empty/enemy/source/static/zone/affinity tests | pass |
| P6-03 | Atomic spatial/engagement/tracking | `DR-04` | same UnitBody mutation, scoped block release/refresh, live tracking | identity/state, engagement reacquire, homing/fixed/area regressions | pass |
| P6-04 | Ground/Platform effect transition | `DR-12` | Any-profile tile transaction and Rest epoch | Ground↔Platform block/focus/rest and facing cadence tests | pass |
| P6-05 | DTO/server/data/version | Master Phase 6 gate | required DTO/event v29/server/request/policy | world wire, server serde, strict policy/RON tests | pass |
| P6-06 | Canonical/legacy/dirty/broad | Master common gate | four canonical docs, no Phase 7~9 placeholder | Core 895, server 55, RON 24, check/fmt/diff | pass |

## 현재 진행 상태

- Phase 5 completion audit 확인: 완료
- Runtime/data/server discovery와 baseline: 완료
- Production/DTO/data/canonical 구현: 완료
- Focused/broad/static/dirty/legacy 감사: 완료
- Completion audit: [Complete / High](COMPLETION_AUDIT.md)
