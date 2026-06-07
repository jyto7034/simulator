# Abnormality Attempt Retreat Goal

이 goal은 전투 노드를 이상현상으로 취급하고, 최대 3회 진입 시도, 퇴각, 재진입, 실패/성공/소멸에 따른 노드 소비 정책을 core lifecycle에 구현하는 작업이다.

관련 정책은 `docs/core_policy_decisions_2026_06.md`의 `이상현상 시도와 퇴각`을 따른다.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/abnormality_attempt_retreat/PLAN.md
docs/goals/abnormality_attempt_retreat/EXPERIMENTS.md
docs/goals/abnormality_attempt_retreat/EXPERIMENT_NOTES.md
```

## Engineering Principles

- 코드 수정은 장기적인 방향으로 한다.
- 처음 구조 판단이 어려우면 trial and error를 수행하고 결과를 기록한다.
- 레거시 “후퇴하면 노드 실패 소비” 정책은 compatibility layer로 보존하지 않는다.
- 문서를 무조건 신뢰하지 않고 실제 map/node/battle lifecycle 코드를 읽는다.
- 과거 정책을 고정하는 테스트는 삭제 또는 최신 focused test로 교체한다.
- 사용자와 의논하여 정해야 할 정책이 발견되면 goal을 종료한다.

## Goal Execution Contract

- source of truth 우선순위는 실제 runtime code, live RON/data, Unity-facing snapshot/command 계약, 최신 정책 문서 순서로 본다.
- 문서와 코드가 충돌하면 코드를 먼저 읽고, 최신 정책과 코드의 차이를 `EXPERIMENT_NOTES.md`에 기록한 뒤 수정 방향을 정한다.
- compatibility layer, fallback path, dual schema는 기본적으로 만들지 않는다. 정말 필요하면 이유, 제거 예정 조건, 테스트 범위를 `PLAN.md`에 기록하고 사용자 확인을 받는다.
- 테스트는 내부 구현 모양보다 사용자-visible behavior, Unity-facing DTO, data validation, live RON loading, 실제 gameplay flow를 고정한다.
- 새 정책과 충돌하는 테스트는 기대값만 바꾸지 않는다. 그 테스트가 무엇을 보호하던 것인지 확인한 뒤 삭제하거나 최신 정책 테스트로 교체한다.
- ignored test로 레거시를 보존하지 않는다.
- goal 범위를 벗어난 개선안은 바로 구현하지 말고 `EXPERIMENT_NOTES.md`에 후속 후보로 기록한다. 현재 goal 완료에 필수이거나 blast radius를 줄이는 경우에만 적용한다.
- Unity-facing DTO shape, live RON schema, 저장 데이터 migration, UX 의미 변화, 밸런스 기준, 기존 콘텐츠 삭제/대체, 실패/보상/소비 시점 변화처럼 사용자 결정이 필요한 정책이 발견되면 임의로 결정하지 않고 goal을 종료하고 질문 목록을 보고한다.
- 작은 변경 단위마다 빠른 focused test 또는 `cargo check`를 먼저 돌리고, 마지막에 넓은 테스트를 돌린다.
- 테스트 실패는 `EXPERIMENTS.md`에 실패 원인, 수정 내용, 재검증 결과를 기록한다.
- 완료 시 변경 요약, 제거한 레거시, 새로 고정한 계약, 갱신한 테스트, 남은 위험, 실행한 검증 명령을 보고한다.

## Canonical Unity Docs

Unity-facing 계약/구현 문서의 canonical 위치는 `F:\unity projects\ark\docs`다. Linux/WSL 경로로는 `/mnt/f/unity projects/ark/docs`다.

아래 파일은 외부 canonical 문서를 기준으로 확인하고 갱신한다.

- `F:\unity projects\ark\docs\unity_core_contract.md`
- `F:\unity projects\ark\docs\unity_client_implementation_goal.md`

이 저장소 안의 같은 이름 문서는 stale copy일 수 있다.

## Source Of Truth

우선 읽을 파일:

- `src/game/world.rs`: command dispatch.
- `src/game/world/state.rs`: run/map/node/battle state.
- `src/game/world/node_flow.rs`: node selection/confirm/complete.
- `src/game/world/combat.rs`: battle start/end/retreat handling.
- `src/game/world/snapshot.rs`: Unity-facing state snapshot.
- `src/game/managers/action_scheduler.rs`: `allowed_actions`.
- `src/game/world/helpers.rs`: action validation.
- `src/game/behavior.rs`: `RetreatBattle`, result DTO.
- `src/game/battle/core/mod.rs`: battle end state.
- `src/game/battle/timeline.rs`: `BattleEnd`.
- `src/game/world/tests.rs`: retreat/node consumption tests.
- `docs/core_policy_decisions_2026_06.md`.
- `F:\unity projects\ark\docs\unity_core_contract.md`.

필요하면 `../game_server`의 `/game` actor mapping을 확인한다.

## Objective

완료 후 다음이 가능해야 한다.

1. 전투 노드는 `ConfirmEnterNode` 시 즉시 소비되지 않는다.
2. 같은 이상현상은 최대 3번까지 진입 시도할 수 있다.
3. `BattleEnd` 전까지 후퇴할 수 있다.
4. 모든 아군이 전투불능이어도 `BattleEnd` 전이면 후퇴할 수 있다.
5. 후퇴하면 attempt 1회를 소모하고, 남은 attempt가 있으면 Safezone/NodeConfirm으로 복귀한다.
6. 퇴각 없는 실패는 노드를 즉시 소비한다.
7. 3회 시도 소진 시 이상현상이 사라지고 노드가 소비된다.
8. 성공 시 노드가 소비되고 보상 처리가 진행된다.

## In Scope

- abnormality attempt state 추가.
- node confirm/in battle snapshot에 남은 attempt와 retreat 가능 여부 노출.
- retreat command 처리 변경.
- battle end 전후 allowed action 정리.
- 재진입 시 battle state 초기화.
- attempt 소진 시 node consumption 처리.
- 기존 retreat tests 교체.
- Unity 계약 문서 갱신.

## Out Of Scope

- consumable modifier duration 감소. 이는 `consumable_modifier_reentry_duration_goal.md`에서 처리한다.
- threat warning false rumor의 `Disproved` 전환. 이 goal은 attempt/re-entry 정책만 다루고 warning 표시 lifecycle은 별도 goal에서 처리한다.
- 전투 밸런스 수치 조정.
- Unity UI 구현.

## Implementation Plan

1. 현재 node consumption 시점을 읽는다.
2. 현재 retreat test `retreat_from_live_defense_battle_consumes_node...`가 어떤 정책을 고정하는지 확인한다.
3. 전투 노드 attempt state를 어디에 저장할지 결정한다.
4. `ConfirmEnterNode`가 attempt를 시작하되 node를 즉시 소비하지 않게 바꾼다.
5. `RetreatBattle`이 `BattleEnd` 전까지 허용되는지 확인한다.
6. retreat 시 battle state를 종료하고 Safezone/NodeConfirm으로 복귀한다.
7. remaining attempts가 0이면 node를 소비한다.
8. 실패/성공/소진의 node result 처리와 reward/no reward 정책을 정리한다.
9. snapshot/allowed_actions를 갱신한다.
10. 구형 retreat tests를 최신 정책 테스트로 교체한다.

## Test Requirements

- `ConfirmEnterNode`만으로 전투 노드가 소비되지 않는다.
- retreat 1회 후 같은 node에 재진입할 수 있다.
- retreat 후 remaining attempts가 감소한다.
- 세 번째 attempt 사용 후 이상현상이 사라지고 node가 소비된다.
- `BattleEnd` 이후에는 retreat가 거부된다.
- 모든 아군이 전투불능이어도 `BattleEnd` 전이면 retreat가 허용된다.
- 퇴각 없는 실패는 node를 소비한다.
- 성공은 node를 소비하고 보상을 지급한다.
- 구형 “retreat consumes node” 테스트는 삭제/교체한다.

검증 후보:

```text
cargo test -p game_core retreat -- --nocapture
cargo test -p game_core node_flow -- --nocapture
cargo test -p game_core live_defense -- --nocapture
cargo test -p game_core -- --nocapture
cargo check -p game_core
cargo check -p game_server
```

## Completion Conditions

- 이상현상 attempt lifecycle이 core state에 구현된다.
- retreat/node consumption tests가 최신 정책을 고정한다.
- Unity-facing snapshot/allowed action 계약이 문서와 일치한다.
- 과거 retreat 소비 정책 테스트가 남아 있지 않다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- attempt state를 map node에 저장할지 selected event/session에 저장할지 정책 결정이 필요하다.
- 퇴각 후 정확히 어떤 state로 돌아갈지 코드만으로 판단하기 어렵다.
- 전투불능과 BattleEnd 확정 사이의 상태 순서가 큰 재설계를 요구한다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
