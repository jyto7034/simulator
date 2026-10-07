# DefenseRoute 전술 지휘 Phase 9 — 현장 작업과 최종 교차 통합

Status: complete

Implementation Complete: true

Goal Type: phase

Execution Mode: implement_and_verify

Production Mutation Authorized: yes — 이 Phase의 `DR-17`과 명시된 교차 통합 범위에 한정

Parent Master: [defense_route_tactical_command_master](../defense_route_tactical_command_master/PLAN.md)

Predecessor Audit:
[Phase 8 Complete / High](../defense_route_tactical_command_phase_08_manual_skill_targeting/COMPLETION_AUDIT.md)

Phase: 9

Goal Name: `defense_route_tactical_command_phase_09_field_work_integration`

Target Repository: `/mnt/f/work/simulator`

Start Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`

## 목표

별도 setup `FieldWorkObjectDef`, stable object/work-cell identity, object-owned Preserve progress와 typed completion
effect를 구현한다. Authored work-cell 최종 점유에서 `Defense + FieldWorking` assignment를 자동 reconcile하고
battle-time 기여, exactly-once completion, checkpoint/resync/server projection을 배치·회수·사망·전술 이동·교대와
원자 통합한다. 첫 live repair object를 제공한 뒤 Phase 0~9 전체 교차 감사를 닫는다.

## 승인된 정책과 근거

- Primary policy: `DR-17`
- Cross-policy integration: `DR-03`, `DR-06`, `DR-08`, `DR-09`, `DR-13`, `DR-14`, `DR-15`, `DR-16`
- Target source:
  [DefenseRoute 전술 지휘 목표 정책 결정](../../decisions/defense_route_tactical_command_policy_2026-08-06.md#dr-17--현장-작업)
- Master boundary:
  [Phase 9](../defense_route_tactical_command_master/PLAN.md#phase-9--현장-작업과-최종-교차-통합)
- Predecessor:
  [Phase 8 Complete / High](../defense_route_tactical_command_phase_08_manual_skill_targeting/COMPLETION_AUDIT.md)

## 확정 정책

- Setup은 tactical resource/static object/tile kind와 별도인 `FieldWorkObjectDef`를 사용한다. Stable object ID,
  presentation kind, position, 하나 이상의 stable work cell, positive required work와 typed completion effect가 필수다.
- Assignment command와 별도 비용은 없다. Active Player unit이 incomplete object의 authored work cell을 최종 점유하면
  같은 mutation에서 자동 assignment되고 직접 배치·재배치는 `Defense + FieldWorking`으로 시작한다.
- Final occupancy mutation은 assignment를 batch reconcile한다. 이탈한 worker는 일반 Defense/tile effect로 복귀하고,
  진입한 적격 unit은 worker가 된다. Relocation과 swap 양쪽 역할 및 자발적 withdraw는 허용한다.
- FieldWorking은 새 기본 공격/automatic·manual skill, block, stance toggle, DefenseFocus와 Rest를 막는다. Worker는
  damage를 계속 받고 enemy overlap/attack은 assignment 또는 progress를 끊지 않는다.
- 각 active eligible worker는 battle time 1ms당 object progress 1ms를 기여한다. 여러 work cell은 합산하며 progress는
  required work에서 clamp한다. Worker 0에서도 progress는 Preserve하고 Reset enum/분기는 두지 않는다.
- Hard CC는 assignment/progress를 보존하고 capability에서 worker contribution만 정지한다. Pause는 clock을 정지한다.
  어느 쪽도 해제 후 catch-up하지 않는다.
- 최초 required 도달 update에서 한 번만 Completed가 되고 typed effect가 한 번 실행된다. Work cell은 underlying tile로
  돌아가며 남은 worker assignment 종료와 일반 Defense/tile effect 복귀를 같은 mutation에서 수행한다.
- Assignment start/stop과 completion만 semantic event다. Checkpoint/resync는 current progress/completion/worker mapping을
  stable object/work-cell/unit ordering으로 보존한다.
- 첫 live object는 `suppress_freischutz.black_box_emergency_repair`, `EmergencyRepairTerminal`, object와 단일 Ground
  work cell `(2, 6)`, `required_work_ms: 10_000`, `RepairProtectedUnit { flat_hp: 100 }`이다. Full HP에서도 완료·소모한다.

## 범위

- Strict field-work setup/live schema, validation and first live object authoring
- Runtime object/progress/work-cell assignment state and real `Ready`/`FieldWorking` activity
- Battle-time contribution scheduling, stale completion no-op and exactly-once typed repair
- Deploy/redeploy/withdraw/death/relocation/swap/Hard CC/Pause/stance/action/tile-effect integration
- Setup/checkpoint/resync/event/current game_server contract and deterministic stable ordering
- Canonical documents, Phase 9 Complete/High audit and master cross-phase completion audit

## 명시적 비범위

- Assign/Cancel work command or separate work cost
- Reset interruption mode, generic scripting/status/effect framework or additional live work content
- Player pathfinding, reservation, pending movement or swap fallback
- Tactical resource/static object/tile-kind state reuse
- Unity runtime/contract/UI/tests

## 구현 계획

1. Current setup/scenario/live data, BattleCore time/event/state, activity/capability and all position lifecycle seams을 조사한다.
2. Narrow strict FieldWork definition/runtime/checkpoint carriers and validation을 implement하고 first live object를 author한다.
3. Final-occupancy assignment batch reconcile과 `Ready`/`FieldWorking` gates/effects를 모든 position/lifecycle mutation에 연결한다.
4. Eligible worker contribution과 generation-safe completion event, Preserve/Hard CC/Pause no-catch-up, typed repair exactly-once를 구현한다.
5. Semantic event, checkpoint/resync/state and game_server current strict transport를 cutover한다.
6. Required focused/static/broad/canonical gates와 Phase 9 Complete/High audit을 수행한다.
7. `DR-01`~`DR-17` traceability, final strict shapes, cross-policy integration, dirty/out-of-scope 보존을 master completion audit한다.

## 완료 조건

- Invalid/missing/duplicate field-work definition and unstable object/work-cell mappings reject deterministically.
- Direct deploy/redeploy, leave/withdraw/death, relocation and two-role swap produce exact atomic assignment/activity/tile effects.
- Single/multi-worker progress, Preserve resume, enemy overlap/damage, Hard CC/Pause no-catch-up and clamp are direct-tested.
- Completion/effect/event are exactly once including stale event and full-HP repair; completed cells no longer assign.
- Checkpoint/reconnect/setup/server preserve stable object/work-cell/unit state and deterministic ordering.
- First live object is exact and no generic/reset/additional content or compatibility contract exists.
- Core/server/RON/canonical/static/broad gates, Phase 9 audit and final master audit pass.

## DTO/data/save migration

Discovery completed. The immutable battle setup snapshot is strict version `2`, the append-only battle event log is strict version
`31`, and checkpoint/update/resync use the current Core DTO directly through game_server. Active battle is not part of the durable
run payload, so no save migration exists. No compatibility reader, alias, dual schema or speculative durable state was added.

## 필수 focused 검증

- required/positive/unique/stable definition and exact first live object
- direct deploy/redeploy assignment, leave/withdraw/death, relocation and swap handoff atomicity
- FieldWorking basic attack/skill/block/stance/DefenseFocus/Rest gates with withdraw/relocation/swap allowed
- single/multi-worker time contribution, Preserve, enemy overlap/damage, Hard CC and Pause no-catch-up
- exact completion, stale event no-op, full-HP consumption and completed underlying tile restoration
- checkpoint/reconnect/setup/event/server serialized shape and stable ordering

## 필수 broad 검증

- `cargo test -p game_core`
- `cargo test -p game_server`
- `cargo test -p game_core --test ron_loading`
- `cargo check --workspace --all-targets`
- `cargo fmt --all -- --check`
- final `DR-01`~`DR-17` cross-phase integration/static audit

## 중단 조건

- Approved object/progress/assignment/completion semantics require a new user-visible policy or tuning choice
- Required current cutover needs compatibility or durable migration not authorized here
- Core/server/live data/test runner is inaccessible
- Predecessor or unrelated user dirty cannot be preserved safely

## Compatibility

승인된 compatibility는 없다. Strict current shape only.

## 완료 감사 Evidence Matrix

| ID | 완료 조건 | 정책 근거 | Runtime/data/contract 근거 | 검증 근거 | 판정 |
| --- | --- | --- | --- | --- | --- |
| P9-01 | Strict setup/live object | `DR-17` | PvE FieldWork schema/validation + exact Freischutz object | invalid schema and 256-seed three-template live audit | pass |
| P9-02 | Atomic assignment/activity lifecycle | `DR-17`, cross policies | final-occupancy reconcile + real activity | deploy/redeploy/withdraw/death/relocate/swap/platform tests | pass |
| P9-03 | Battle-time progress/CC/Pause | `DR-17`, `DR-13`, `DR-15` | object settlement + derived contributor count | single/multi/zero-worker/Hard CC/Pause no-catch-up | pass |
| P9-04 | Exactly-once completion/effect | `DR-17` | generation-safe completion + typed repair | stale event/full HP/clamp/underlying tile tests | pass |
| P9-05 | Checkpoint/event/server ordering | `DR-17` | setup v2/checkpoint/event v31/Core DTO server mapping | live recovery and serialized setup/update/event tests | pass |
| P9-06 | Canonical/legacy/dirty/broad/master | Master final gate | canonical rules/runtime/data aligned | Core 910/server 56/RON 24/check/fmt/diff/static | pass |

## 현재 진행 상태

- Phase 8 completion audit 확인: 완료
- Exact DR-17/Phase 9 policy extraction: 완료
- Child path/문서 초기화와 active 전환: 완료
- Runtime/live data/transport discovery: 완료
- Production implementation: 완료
- Focused/broad/canonical audit: 완료
- Phase completion audit: [Complete / High](COMPLETION_AUDIT.md)
- Master completion audit: 별도 Master final audit에서 수행
