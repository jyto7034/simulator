# DefenseRoute 전술 지휘 Phase 0 — Experiment Notes

Status: complete

## 초기 구조 판단

- 사용자의 Master `PLAN.md` 수행 요청을 Phase 0 activation 승인으로 해석했다.
- 저장소 phase naming 관례와 Master의 경계를 그대로 표현해 Goal 이름을
  `defense_route_tactical_command_phase_00_policy_baseline`으로 정했다.
- 이 Phase는 정책/source-of-truth 분리, baseline과 `DR-01`/`DR-15`만 소유한다.
- Phase 1~9 production 구현은 선행하지 않는다.

## 초기 dirty worktree

Phase Goal 초기화 직전 `git status --short`로 확인한 항목:

```text
 M core_enhanced/docs/README.md
 M core_enhanced/docs/engineering_workflow.md
 M core_enhanced/docs/goal_completion_review_guide.md
 M core_enhanced/docs/skills/abnormality_skill_design_notes.ko.md
?? core_enhanced/docs/goal_authoring_and_execution_guide.md
?? core_enhanced/docs/goals/defense_route_tactical_command_master/
?? core_enhanced/docs/tmp/
```

- 위 항목은 기존 사용자 또는 Master 초기화 변경으로 보존한다.
- `core_enhanced/docs/tmp/`의 DefenseRoute 임시 조사 문서는 Phase 0 입력이지만 다른 임시 파일은
  수정하거나 정리하지 않는다.
- Production 수정 전 대상 파일의 현재 diff와 관련 baseline을 별도로 확인한다.

## 경로 충돌

- 초기화 전 Phase Goal 디렉터리와 필수 문서 3개는 모두 존재하지 않았다.
- 새 Goal 경로는 기존 이름과 충돌하지 않는다.

## Source-of-truth 판단

- 승인된 목표 정책은
  `core_enhanced/docs/decisions/defense_route_tactical_command_policy_2026-08-06.md`가 소유한다.
- 임시 조사 문서는 2026-08-06 historical evidence로 동결했고 목표 정책 source가 아니다.
- 현재 실제 동작은 Core/game_server code와 production live data에서 확인한다.
- 현재 canonical은 구현된 현재 계약만 설명해야 한다.
- 새 decision record는 목표 정책을 소유하되 미래 정책을 현재 구현으로 주장하지 않는다.

## Phase별 현재 baseline과 canonical 갱신 위치

| Phase | 현재 authoritative seam과 gap | 구현 시 갱신할 canonical |
| --- | --- | --- |
| 0 | `WinCondition`/`compute_winner`와 playback/revision/actor serialization은 존재. Pause 중 실제 mutation sequence test gap은 이 Phase에서 보강 | `game_rulebook.md`, `core_runtime_contract.md`, `game_server/docs/unity_bridge_protocol.md` |
| 1 | Core withdraw/redeploy, HP lock과 delivery cleanup seam은 존재하나 올림 비용, unreleased instant attack과 released GroundFixed 보존은 미완성 | `game_rulebook.md`, `core_runtime_contract.md` |
| 2 | Stun/Freeze/Silence와 hard-CC expiry helper는 존재하지만 모든 command/activity가 소비하는 중앙 capability와 목표 중첩/취소 계약은 없음 | `game_rulebook.md`, `core_runtime_contract.md` |
| 3 | `LiveBattleDeploymentState`가 current/max/natural recovery와 deploy mapping을 함께 소유하고 RON/DTO 이름도 `live_deployment`/cost/stabilization을 사용 | `game_rulebook.md`, `core_runtime_contract.md`, `data_loading_contract.md`, server transport canonical |
| 4 | 29개 weapon이 공격 profile/base block을 명시하지만 affinity/DefenseFocus/Rest와 starter training sword, weapon-required gate는 없음 | `game_rulebook.md`, `core_runtime_contract.md`, `data_loading_contract.md` |
| 5 | `EngagementStance`, DefenseFocus, Rest production symbol과 state/event/profile이 없음 | `game_rulebook.md`, `core_runtime_contract.md`, `data_loading_contract.md` |
| 6 | Deploy zone/affinity primitive와 RuntimeUnit position은 있으나 relocation command, command-level allied occupancy와 actual tile effect transition이 없음 | `game_rulebook.md`, `core_runtime_contract.md`, server transport canonical |
| 7 | `SwapUnitPositions` command/state/event가 없음 | `game_rulebook.md`, `core_runtime_contract.md`, server transport canonical |
| 8 | Optional `SkillCastTarget::Unit/Tile`과 cast target preservation은 있으나 `SkillCastSelection`, strict live selection과 tactical focus/preview authority가 없음 | `game_rulebook.md`, `skill_target_contract.md`, `data_loading_contract.md`, server transport canonical |
| 9 | Setup은 `tactical_points`와 빈 `static_objects`를 투영하지만 FieldWork def/state/assignment/progress/event가 없음 | `game_rulebook.md`, `core_runtime_contract.md`, `data_loading_contract.md`, server transport canonical |

## Version과 migration baseline

| 계약 | 현재 version/shape | 후속 owner |
| --- | --- | --- |
| Live RON manifest | schema `1` | Schema 변경 Phase가 strict cutover 판단 |
| Core active-run payload | `ActiveRunCorePayloadV2`, version `2` | Phase 3/4가 같은 release면 최종 tactical-resource+weapon shape로 한 번 cutover 가능 |
| game_server active-run envelope | version `3` | Core payload가 opaque라 envelope shape가 실제로 바뀔 때만 별도 bump |
| Battle setup snapshot | setup version `1` | Phase 6/9의 setup field가 같은 release면 최종 shape로 한 번 bump 가능 |
| Battle event log | version `27` | 새 semantic variant를 추가하는 각 배포 단위가 fixture와 함께 bump |
| Battle checkpoint/update | explicit schema version field 없음, `battle_state_revision`이 mutation authority | Shape 변경 Phase가 exact serializer/server mapping과 reconnect fixture를 함께 갱신 |

미래 Phase field를 현재 version에 미리 추가하지 않는다. 정확한 version number와 같은-release 결합 여부는
해당 Phase 시작 시 current tree와 배포 단위를 다시 확인한다.

## DR-01 현재 evidence

- `core_enhanced/src/game/battle/scenario.rs::WinCondition`
- `core_enhanced/src/game/battle/core/sim.rs::compute_winner`
- `core_enhanced/docs/game_rulebook.md`의 `전투 모드`
- Focused baseline: `protect_unit_until` 2 passed,
  `battle_setup_snapshot_exposes_survival_timer_without_encirclement_variant` 1 passed

전술 command 사용 여부는 winner 계산 입력이 아니며 기존 protected-unit/required-group/survival 의미와
decision record가 일치한다. Runtime correction은 필요하지 않았다.

## DR-15 현재 evidence와 correction

- Core clock: `ActiveBattleSession::simulation_delta_for_tick`은 paused면 delta를 만들지 않는다.
- Core transaction: `handle_pause_battle`, `handle_resume_battle`, `handle_set_battle_speed`와 일반 battle
  handlers는 commit 가능 여부 확인 뒤 revision을 한 번 증가시킨다.
- Capability: playback paused는 deploy/withdraw/skill 자체를 비활성화하지 않는다.
- Server: `PlayerGameActor`가 mailbox에서 request를 직렬 처리하며 stale error 뒤 current resync를 보낸다.
- Canonical: rulebook/runtime/server protocol에 현재 clock/serial/revision 의미를 동기화했다.
- Test gap correction: Pause 중 deploy→invalid deploy→withdraw→invalid withdraw, manual skill start와 stored
  speed 변경/Resume, 동일 revision 두 mutation의 first-commit/second-stale actor 순서를 추가했다.

Production runtime 동작은 바꾸지 않았고 현재 contract를 직접 보호하는 regression test와 canonical만
보강했다.

## 외부 소비자와 비범위

- Unity 문서, runtime, contract와 tests는 사용자 승인 비범위이며 접근하지 않는다.
- Core/server 완료에 필요한 접근 가능한 소비자만 Phase 0 baseline에 포함한다.

## 적용하지 않은 대안

### Master 문서에서 Phase 0을 직접 실행

적용하지 않는다. Master는 orchestration-only이며 실제 조사·변경·검증은 활성 Phase Goal이
소유해야 한다.

### 미래 Phase canonical과 schema를 미리 작성

적용하지 않는다. 구현되지 않은 목표를 현재 계약으로 오인하게 만들고 placeholder migration을
발생시킨다.

## 생략한 검증과 근거

- Unity 검증은 승인 비범위다.
- Phase 1~9 runtime behavior는 아직 구현되지 않아 Phase 0 완료 evidence로 실행하지 않는다.
- Repository 전체 test는 dependency 근거가 없는 범위를 포함하므로 Core/server package와 live RON suite를
  broad gate로 사용한다.

## 남은 위험

- Unity 제외로 Core/server contract의 실제 presentation 소비는 이 Master에서 증명하지 않는다.
- 미래 Phase command는 구현될 때마다 `DR-15` Pause consumer test를 자체 gate로 추가해야 한다.
- Phase 3/4와 Phase 6/9의 version bump 결합 여부는 실제 배포 단위가 확정될 때 재판정해야 한다.

## 다음 안전한 작업

1. Master registry에서 Phase 0 audit evidence를 연결하고 Phase 1만 ready로 전환한다.
2. Phase 1 child Goal 이름/경로를 확정해 필수 문서 3개를 초기화한다.
3. Phase 1 production 조사 전에 현재 withdrawal/redeploy baseline과 대상 diff를 다시 확인한다.
