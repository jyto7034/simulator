# DefenseRoute 전술 지휘 Phase 3 — Experiment Notes

Status: active

## 초기 구조 판단

- Master Phase 3의 승인 범위를 그대로 표현하는
  `defense_route_tactical_command_phase_03_tactical_resource_authority` 이름을 사용했다.
- Phase 3은 `DR-02`의 resource authority, existing deploy/redeploy consumer, policy/effect/DTO strict naming만
  소유한다.
- Phase 4~9의 weapon/stance/relocation/swap/manual-target/FieldWork command를 선행하지 않는다.

## 초기 dirty worktree

Phase 3 초기화 직전 status는 Phase 0~2의 verified production/canonical/test/Goal 변경과 다음 기존 사용자
변경을 함께 포함한다.

```text
 M core_enhanced/docs/README.md
 M core_enhanced/docs/core_runtime_contract.md
 M core_enhanced/docs/engineering_workflow.md
 M core_enhanced/docs/game_rulebook.md
 M core_enhanced/docs/goal_completion_review_guide.md
 M core_enhanced/docs/skills/abnormality_skill_design_notes.ko.md
 M core_enhanced/src/game/battle/buffs.rs
 M core_enhanced/src/game/battle/core/basic_attack.rs
 M core_enhanced/src/game/battle/core/build.rs
 M core_enhanced/src/game/battle/core/commands.rs
 M core_enhanced/src/game/battle/core/mod.rs
 M core_enhanced/src/game/battle/core/movement/blocking.rs
 M core_enhanced/src/game/battle/core/movement/engine.rs
 M core_enhanced/src/game/battle/core/movement/planner.rs
 M core_enhanced/src/game/battle/core/sim.rs
 M core_enhanced/src/game/battle/core/skill_runtime/area.rs
 M core_enhanced/src/game/battle/core/targeting.rs
 M core_enhanced/src/game/battle/core/types.rs
 M core_enhanced/src/game/battle/enums.rs
 M core_enhanced/src/game/battle/validation/buffs.rs
 M core_enhanced/src/game/battle/validation/validator.rs
 M core_enhanced/src/game/behavior.rs
 M core_enhanced/src/game/world/combat.rs
 M core_enhanced/src/game/world/state.rs
 M core_enhanced/src/game/world/tests/combat.rs
 M core_enhanced/src/game/world/tests/mod.rs
 M game_server/docs/unity_bridge_protocol.md
 M game_server/src/game/player_game_actor/handlers.rs
 M game_server/src/game/player_game_actor/messages.rs
?? core_enhanced/docs/decisions/defense_route_tactical_command_policy_2026-08-06.md
?? core_enhanced/docs/goal_authoring_and_execution_guide.md
?? core_enhanced/docs/goals/defense_route_tactical_command_master/
?? core_enhanced/docs/goals/defense_route_tactical_command_phase_00_policy_baseline/
?? core_enhanced/docs/goals/defense_route_tactical_command_phase_01_withdraw_redeploy_lifecycle/
?? core_enhanced/docs/goals/defense_route_tactical_command_phase_02_hard_cc_silence_capability/
?? core_enhanced/docs/tmp/
```

- Phase 0~2 output을 verified predecessor로 보존한다.
- README/workflow/audit guide/abnormality skill 문서는 기존 사용자 변경으로 보존한다.
- Production 수정 전 대상 파일의 Phase 3 시작 diff를 확인해 기존 변경과 겹치는 부분을 구분한다.

## 경로 충돌

- Phase 3 후보 디렉터리와 필수 문서 3개는 초기화 전에 모두 없었다.
- 생성한 경로는 기존 Goal과 충돌하지 않는다.

## Source-of-truth 판단

- 목표 정책은 target decision record `DR-02`다.
- Current resource arithmetic/lifecycle은 `LiveBattleDeploymentState`와 live deployment policy일 가능성이 높지만
  production 조사 전 exact producer/consumer 책임을 확정하지 않는다.
- Phase 1의 deploy/redeploy/withdraw transaction과 Phase 2의 active battle/checkpoint strict projection은
  verified predecessor constraint다.
- DTO/RON/save/version migration 책임은 current schema와 모든 reader 조사 전까지 discovery pending이다.

## Production discovery 결론

- Resource authority는 `LiveBattleDeploymentState.current_cost/max_cost/cost_per_second/last_cost_update_ms`에
  결합되어 있다.
- Deployment lifecycle authority는 같은 struct의 base/redeploy price, cooldown, salt, deployed mapping과
  redeploy locks다.
- Strict split 뒤 resource current/max/recovery clock은 `TacticalResourceState`, deploy/redeploy price tuning은
  `TacticalResourcePolicy`, cooldown/salt/mapping은 `LiveBattleDeploymentPolicy/State`가 소유한다.
- Phase 6/7의 relocation/swap cost tuning은 실제 consumer와 함께 추가한다. Phase 3에서 dead policy/DTO
  placeholder를 만들지 않는다.
- Root snapshot과 live checkpoint는 현재 같은 combined deployment DTO를 사용한다. 둘 다 별도 required
  `tactical_resource`를 같은 release에서 projection해야 한다.
- game_server는 Core checkpoint DTO를 그대로 side-message로 운반하므로 Core type cutover와 server fixture
  update 외 별도 adapter가 없다.
- Active battle session은 persisted run checkpoint/save에 들어가지 않는다. Reconnect는 same in-memory session,
  Retreat/battle end는 `active_battle = None`이므로 save migration/version increment는 불필요하다.
- 기존 regen은 whole-second cadence를 유지하지만 large elapsed의 `u64 -> u32 as`와 at-max cursor 의미를 새
  state에서 명시적 saturating arithmetic으로 강화한다.

## 외부 소비자와 비범위

- game_server는 접근 가능한 Core consumer로 검증한다.
- Unity runtime/contract/tests는 사용자 승인 비범위라 접근하지 않는다.
- Relocation/swap/stance/Rest/DefenseFocus/FieldWork consumer는 각 future Phase가 소유한다.

## 남은 위험

- Phase 3 범위의 알려진 correctness gap은 없다.
- Future relocation/swap consumer가 생길 때 현재 pool과 payment boundary를 재사용하고 별도 resource state를
  만들지 않아야 한다.
- Unity consumer는 승인 비범위라 strict Core/server DTO의 외부 presentation adoption은 별도 후속 책임이다.

## 다음 안전한 작업

1. Phase 3 `Complete / High` audit을 Master에 연결한다.
2. Master Phase 4만 ready로 전환한다.
3. Phase 4 child가 활성화되기 전에는 weapon tactical profile production 작업을 시작하지 않는다.
