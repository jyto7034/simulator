# DefenseRoute 전술 지휘 Phase 2 — Experiment Notes

Status: complete

## 초기 구조 판단

- Master Phase 2의 승인 범위를 그대로 표현하는
  `defense_route_tactical_command_phase_02_hard_cc_silence_capability` 이름을 사용했다.
- Phase 2는 `DR-13`의 중앙 control capability와 현재 action consumer만 소유한다.
- Phase 3의 tactical resource authority와 Phase 5~9의 stance/movement/work consumer를 선행하지 않는다.

## 초기 dirty worktree

Phase 2 초기화 직전 status:

```text
 M core_enhanced/docs/README.md
 M core_enhanced/docs/core_runtime_contract.md
 M core_enhanced/docs/engineering_workflow.md
 M core_enhanced/docs/game_rulebook.md
 M core_enhanced/docs/goal_completion_review_guide.md
 M core_enhanced/docs/skills/abnormality_skill_design_notes.ko.md
 M core_enhanced/src/game/battle/core/basic_attack.rs
 M core_enhanced/src/game/battle/core/build.rs
 M core_enhanced/src/game/battle/core/mod.rs
 M core_enhanced/src/game/battle/core/sim.rs
 M core_enhanced/src/game/battle/core/skill_runtime/area.rs
 M core_enhanced/src/game/battle/core/types.rs
 M core_enhanced/src/game/world/combat.rs
 M core_enhanced/src/game/world/state.rs
 M core_enhanced/src/game/world/tests/combat.rs
 M game_server/docs/unity_bridge_protocol.md
 M game_server/src/game/player_game_actor/handlers.rs
?? core_enhanced/docs/decisions/defense_route_tactical_command_policy_2026-08-06.md
?? core_enhanced/docs/goal_authoring_and_execution_guide.md
?? core_enhanced/docs/goals/defense_route_tactical_command_master/
?? core_enhanced/docs/goals/defense_route_tactical_command_phase_00_policy_baseline/
?? core_enhanced/docs/goals/defense_route_tactical_command_phase_01_withdraw_redeploy_lifecycle/
?? core_enhanced/docs/tmp/
```

- Phase 0/1의 production/canonical/test/Goal 변경은 verified predecessor output으로 보존한다.
- README/workflow/audit guide/abnormality skill 문서는 기존 사용자 변경으로 보존한다.
- Production 수정 전 대상 파일의 Phase 2 시작 diff를 확인해 기존 변경과 겹치는 부분을 구분한다.

## 경로 충돌

- Phase 2 후보 디렉터리와 필수 문서 3개는 초기화 전에 모두 없었다.
- 생성한 경로는 기존 Goal과 충돌하지 않는다.

## Source-of-truth 판단

- 목표 정책은 target decision record `DR-13`이다.
- Current authority는 `BattleCore::buffs`의 active buff instance와 metadata의
  `BuffKind::control_capability()`다. `UnitControlState`와 checkpoint field는 duplicate storage가 아니라
  이 authority의 derived projection이다.
- Phase 1의 release/detached delivery contract는 verified predecessor constraint다.
- Live RON schema와 event-log version migration은 필요하지 않았다. Official active unit checkpoint만 strict
  required `control` field로 같은-release cutover했다.

## 외부 소비자와 비범위

- game_server는 접근 가능한 Core consumer로 검증한다.
- Unity runtime/contract/tests는 사용자 승인 비범위라 접근하지 않는다.
- Future stance, tactical movement/swap, Rest/DefenseFocus와 FieldWork consumer는 해당 Phase가 소유한다.

## 발견한 기존 결함과 닫은 경계

- 서로 다른 Hard CC가 exclusive replacement되어 overlap/max-expiry를 위반했다. Instance별 유지/자연 expiry로
  바꾸고 validator도 같은 모델을 사용한다.
- Hard CC pending cast가 cancel이 아니라 delay됐고 basic windup의 stale resolve와 focus/windup owned lock이
  남았다. Pending cast start seq와 basic attack epoch/deadline으로 release 전 action만 취소한다.
- Block engagement가 Hard CC를 무시했다. Blocker와 blocked enemy 양쪽의 derived Hard CC를 검사해 즉시
  release하고 마지막 expiry 뒤 current state에서 재계산한다.
- Active checkpoint에 control state가 없었다. Battle time과 active buffs에서 exact max expiry/reason을
  파생하며 old missing shape는 거부한다.

## 결정한 lifecycle 세부

- Policy release time은 남은 Hard CC `expires_at_ms`의 최댓값이다. Event queue는 같은 timestamp에 새 action을
  역삽입하지 않도록 automatic retry만 `max_expiry + 1ms`에 예약한다.
- Hard CC cancellation은 auto basic windup과 pending auto/manual cast/focus에만 적용한다. Triggered basic
  effect, cast가 release한 delayed step/projectile/area/buff는 기존 snapshot/runtime을 계속 사용한다.
- Auto cast는 cancellation 뒤 pending intent를 보존하고 Manual cast는 새 command를 기다린다.
- Owned lock은 canceled action deadline과 exact 일치할 때만 줄여 다른 source가 만든 더 긴 lock을 보존한다.
- Silence는 new-skill-only다. Basic windup과 started skill, block/resonance/movement를 건드리지 않는다.
- Hard CC와 Silence overlap의 projection/manual rejection은 `HardCrowdControl`이 우선한다.

## Migration/version 판단

- `game_resources/data/buffs/base.ron`은 수정하지 않았다.
- `BATTLE_EVENT_LOG_VERSION`은 27을 유지한다. Existing `SkillCastInterrupted`, `BuffApplied`, `BuffExpired`로
  lifecycle을 완전히 표현한다.
- Active battle은 run checkpoint/save payload에 포함되지 않으므로 persisted capability migration이 없다.
- Unit checkpoint DTO는 Core/game_server atomic deployment 전제의 strict required field이며 default/alias가 없다.

## 최종 검증 요약

- Focused: Hard CC 15, Silence 6, control status 11, server transport/reason 각 1 pass
- Broad: Core 866, server 55, production RON 23 pass
- Static: direct status consumer 없음, live data diff 없음, event version 27, fmt/diff check pass
- Completion audit: `Complete / High`

## 다음 안전한 작업

Master를 Phase 2 complete로 바꾸고 Phase 3 tactical resource authority child만 active로 전환한다.
