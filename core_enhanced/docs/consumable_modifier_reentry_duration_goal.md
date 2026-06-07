# Consumable Modifier Re-entry Duration Goal

이 goal은 소비 아이템 modifier가 이상현상 재진입 동안 유지되고, duration이 진입 시도마다 감소하지 않도록 정리하는 작업이다.

선행 권장 goal: `docs/abnormality_attempt_retreat_goal.md`.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/consumable_modifier_reentry_duration/PLAN.md
docs/goals/consumable_modifier_reentry_duration/EXPERIMENTS.md
docs/goals/consumable_modifier_reentry_duration/EXPERIMENT_NOTES.md
```

## Engineering Principles

- 코드 수정은 장기적인 방향으로 한다.
- trial and error 결과는 `EXPERIMENTS.md`에 남긴다.
- 이전 소비 아이템 duration 정책이 새 이상현상 정책과 충돌하면 과감히 교체한다.
- 문서를 무조건 신뢰하지 않고 실제 consumable lifecycle 코드를 읽는다.
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

- `src/game/data/consumable_data.rs`: consumable metadata/effect/duration.
- `src/game/employee.rs`: `active_consumable_modifier` 적용/감소.
- `src/game/world/maintenance.rs`: `use_consumable_item` 처리.
- `src/game/world/combat.rs`: combat result/failure lifecycle.
- `src/game/world/state.rs`: world state와 employee roster.
- `src/game/world/snapshot.rs`: active modifier snapshot.
- `src/game/behavior.rs`: `UseConsumableItem`, result DTO.
- `src/game/managers/action_scheduler.rs`: allowed actions.
- `docs/core_policy_decisions_2026_06.md`.
- `docs/consumable_item_goal.md`: 기존 consumable 정책.

## Objective

완료 후 다음이 가능해야 한다.

1. `use_consumable_item` 성공 시점에 item이 즉시 소비된다.
2. 사용된 item은 node cancel, retreat, re-entry, overwrite에도 환불되지 않는다.
3. 같은 이상현상 재진입 동안 `active_consumable_modifier`가 유지된다.
4. `NextCombatNode`/`CombatNodes(n)` duration은 진입 시도마다 감소하지 않는다.
5. 전투/보스 노드가 해결될 때만 duration이 감소한다.
6. 성공, 퇴각 없는 실패, 3회 시도 소진은 node 해결로 간주한다.

## In Scope

- `use_consumable_item` 소비 시점 테스트 보강.
- modifier overwrite 환불 없음 테스트.
- retreat/re-entry 중 modifier 유지.
- duration 감소 시점 변경.
- `NextCombatNode`가 같은 이상현상 재진입 중 유지되는지 보장.
- Unity snapshot 계약 확인.

## Out Of Scope

- 새 consumable effect 추가.
- `DefenseMitigation` runtime 구현.
- consumable UI 구현.
- abnormality attempt lifecycle 자체 구현.

## Implementation Plan

1. 현재 `UseConsumableItem` 성공 시 inventory 제거가 이미 되는지 확인한다.
2. 현재 duration 감소가 어느 lifecycle에서 발생하는지 확인한다.
3. 전투 진입 시 감소하는 로직이 있다면 node resolved 시점으로 옮긴다.
4. abnormality attempt state와 연결해 retreat/re-entry는 duration 감소를 발생시키지 않게 한다.
5. overwrite 시 기존 modifier 제거와 no refund 정책을 테스트로 고정한다.
6. snapshot이 modifier 상태를 정확히 보여주는지 확인한다.

## Test Requirements

- drag/drop command 성공 시 inventory item이 제거된다.
- node cancel 후에도 item이 환불되지 않는다.
- retreat 후 same abnormality re-entry에서 modifier가 유지된다.
- success 후 `NextCombatNode` modifier가 제거된다.
- 퇴각 없는 failure 후 duration이 감소한다.
- 3회 attempt 소진 후 duration이 감소한다.
- modifier overwrite는 기존 item/effect를 환불하지 않는다.

검증 후보:

```text
cargo test -p game_core consumable -- --nocapture
cargo test -p game_core retreat -- --nocapture
cargo test -p game_core node_flow -- --nocapture
cargo check -p game_core
```

## Completion Conditions

- consumable modifier duration 정책이 이상현상 re-entry와 일치한다.
- focused tests가 소비/환불 없음/유지/감소 시점을 검증한다.
- 문서와 Unity contract가 구현과 일치한다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- node resolved 시점을 code에서 명확히 분리하기 어렵다.
- `NextCombatNode` 의미를 node attempt 기준으로 바꿔야 하는 압력이 생긴다.
- death prevent 같은 일회성 effect의 소비 시점과 duration 감소가 충돌한다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
