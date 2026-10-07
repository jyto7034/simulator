# DefenseRoute 전술 지휘 Phase 1 — Experiment Notes

Status: complete

## 초기 구조 판단

- 사용자의 Master 수행 요청과 Phase 0의 동일 naming convention에 따라
  `defense_route_tactical_command_phase_01_withdraw_redeploy_lifecycle` 이름을 사용했다.
- Phase 1은 `DR-03`의 withdraw/redeploy lifecycle만 소유한다.
- Phase 2의 중앙 Hard CC capability나 Phase 3의 tactical resource rename은 선행하지 않는다.

## 초기 dirty worktree

Phase 1 초기화 직전 status:

```text
 M core_enhanced/docs/README.md
 M core_enhanced/docs/core_runtime_contract.md
 M core_enhanced/docs/engineering_workflow.md
 M core_enhanced/docs/game_rulebook.md
 M core_enhanced/docs/goal_completion_review_guide.md
 M core_enhanced/docs/skills/abnormality_skill_design_notes.ko.md
 M core_enhanced/src/game/world/tests/combat.rs
 M game_server/docs/unity_bridge_protocol.md
 M game_server/src/game/player_game_actor/handlers.rs
?? core_enhanced/docs/decisions/defense_route_tactical_command_policy_2026-08-06.md
?? core_enhanced/docs/goal_authoring_and_execution_guide.md
?? core_enhanced/docs/goals/defense_route_tactical_command_master/
?? core_enhanced/docs/goals/defense_route_tactical_command_phase_00_policy_baseline/
?? core_enhanced/docs/tmp/
```

- Phase 0의 canonical/test/decision/Goal 변경은 verified predecessor output으로 보존한다.
- README/workflow/audit guide/abnormality skill 문서는 기존 사용자 변경으로 보존한다.
- Production 수정 전 대상 파일의 Phase 1 시작 diff를 확인해 Phase 0과 겹치는 부분을 구분한다.

## 경로 충돌

- Phase 1 후보 디렉터리와 필수 문서 3개는 초기화 전에 모두 없었다.
- 생성한 경로는 기존 Goal과 충돌하지 않는다.

## Source-of-truth 판단

- 목표 정책은 target decision record `DR-03`이다.
- 실제 authority는 BattleCore action/delivery lifecycle과 World deployment/redeploy transaction이다.
- Run policy는 기존 30초/90초/150%를 유지하며 tactical resource rename은 Phase 3 소유다.
- Serialized shape 변경은 필요하지 않았다. `AreaRuntime`의 source snapshot/facing은 private in-memory
  state이고 existing checkpoint, event, setup, save와 server envelope shape를 유지한다.

## 외부 소비자와 비범위

- game_server는 접근 가능한 Core consumer로 검증한다.
- Unity runtime/contract/tests는 사용자 승인 비범위라 접근하지 않는다.

## 구현 뒤 구조 판단

- `AreaRuntime`은 직렬화되지 않는 private runtime이므로 GroundFixed 생성 시 source snapshot과 facing을
  보관해도 checkpoint/save migration이 생기지 않는다.
- GroundFixed tick의 roll identity는 area ID와 tick time을 source context에 섞어 source 철수 뒤에도
  결정론적이면서 tick별로 독립되게 유지한다.
- Basic/skill projectile은 launch 시점 source snapshot을 이미 소유하므로 withdraw cleanup에서 보존하는
  것이 기존 delivery authority와 맞는다.
- `SkillAreaTracking`의 `GroundFixed`, `FollowCaster`, `FollowTarget` 구분이 autonomous/source-bound
  cleanup의 기존 authoring seam이다. 새 option은 추가하지 않았다.
- Active cast key가 제거된 뒤 도착하는 queued `SkillStep`은 stale event로 no-op해야 release 전 action
  cancellation이 완결된다.
- World의 자발적 철수와 전투불능 reconcile은 `LiveBattleDeploymentState::redeploy_cost()`를 공유해
  ceiling과 overflow 의미를 한 곳에서 소유한다.

## Migration과 consumer 판단

- DTO, production RON, checkpoint/save, event/setup, server wire version migration: 없음.
- Core checkpoint와 game_server generic mapping은 existing redeploy lock/candidate shape를 그대로 전달한다.
- 접근 가능한 consumer인 game_server 전체 54 tests를 통과했다.
- Unity runtime/contract/tests는 사용자 승인 비범위이며 검증한 것으로 간주하지 않는다.

## Dirty worktree 보존

- Phase 1 production 변경은 battle Core의 withdraw/delivery runtime, world redeploy helper와 관련 regression,
  두 current canonical에 한정했다.
- Phase 0의 world test/canonical/server 변경과 시작 시 존재하던 README/workflow/audit guide/skill notes 및
  untracked 문서를 되돌리거나 정리하지 않았다.
- 관련 없는 untracked file 삭제, repository-wide mechanical rewrite와 compatibility layer를 만들지 않았다.

## 남은 위험

- 이후 stance/FieldWork가 새 source-bound effect를 추가하면 동일한 release ownership matrix를 그 Phase에서
  다시 감사해야 한다.
- Future delivery schema가 serialized checkpoint로 확장될 경우 source snapshot migration은 그 변경을
  소유하는 Phase에서 별도로 결정해야 한다.
- 위 항목은 현재 `DR-03` 완료의 material gap이 아니다.

## 다음 안전한 작업

1. Phase 1을 Master registry에서 `Complete / High`로 닫는다.
2. Phase 2 child Goal 필수 문서를 먼저 초기화한다.
3. 그 뒤에만 `DR-13` 중앙 Hard CC/Silence capability를 조사한다.
