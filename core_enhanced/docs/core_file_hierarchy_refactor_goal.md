# Core File Hierarchy Refactor Goal

이 goal은 `core_enhanced`의 파일 계층을 장기적으로 읽기 쉽고 변경하기 쉬운 구조로 정리하기 위한 리팩토링 계획서다.

목적은 동작을 바꾸는 것이 아니라, 현재 구현된 게임 흐름과 Unity-facing 계약을 유지하면서 책임 경계를 더 선명하게 만드는 것이다.

## Objective

`src/game` 루트와 `world`, `combat_preview`, 테스트/디버그 산출물 계층을 정리해, 한 정책을 수정할 때 열어야 하는 파일 수와 source-of-truth 혼선을 줄인다.

## Goal Mode Working Method

goal 시작 시 아래 작업 기억장치를 만든 뒤 계속 갱신한다.

```text
docs/goals/core_file_hierarchy_refactor/PLAN.md
docs/goals/core_file_hierarchy_refactor/EXPERIMENTS.md
docs/goals/core_file_hierarchy_refactor/EXPERIMENT_NOTES.md
```

- `PLAN.md`: 계획, 범위, 완료 조건, 중단 조건, 검증 명령.
- `EXPERIMENTS.md`: 시도/실패/성공 결과, 테스트 실패 원인, 수정 내용, 재검증 결과.
- `EXPERIMENT_NOTES.md`: 작업 중 판단, source-of-truth 충돌, 정책 질문, 후속 후보.

처음에는 어떤 구조가 장기적인 방향인지 판단하기 어려울 수 있다. 작은 trial and error를 통해 확인하고, 실패한 접근과 이유를 `EXPERIMENTS.md`에 남겨 같은 실수를 반복하지 않는다.

## Source Of Truth

문서를 무조건 신뢰하지 말고 아래 순서로 확인한다.

1. 실제 runtime code
2. live RON/data
3. Unity-facing snapshot/command 계약
4. 최신 정책 문서

먼저 읽을 파일/문서:

- `docs/refactor_preparation_plan.md`
- `docs/code_documentation_sync_guidelines.md`
- `src/game/mod.rs`
- `src/game/world.rs`
- `src/game/world/*.rs`
- `src/game/combat_preview.rs`
- `src/game/combat_preview/*.rs`
- `src/game/combat_*.rs`
- `src/game/data/mod.rs`
- `src/game/events/combat.rs`
- `tests/`
- `tests_bak/`
- `debug_event_log_exports/`
- `timeline_exports/`
- `docs/game_rulebook.md`
- 외부 canonical Unity 계약 문서: `/mnt/f/unity projects/ark/docs`

## Current Findings

현재 파일 계층에서 확인한 주요 정리 후보:

- `src/game` 루트의 `combat_*` 파일이 많아져 전투 preview/setup 책임이 루트에 흩어져 있다.
- `src/game/combat_preview.rs`가 매우 크고, 이미 `combat_preview/` 디렉토리가 존재해 파일/디렉토리 경계가 어색하다.
- `src/game/world/admin.rs`가 admin command enum, catalog DTO, fixture entry, grant logic, tests를 모두 포함해 커지고 있다.
- `src/game/world/tests.rs`가 매우 커서 노드/장비/전투/스냅샷 테스트가 한 파일에 섞여 있다.
- `debug_event_log_exports/`, `timeline_exports/`, `tests_bak/`, `src/old/`, `logs/`, `tmp/` 같은 디렉토리의 공식성/산출물 여부가 불명확하다.

## Engineering Principles

코드 수정은 장기적인 방향으로 한다.

피해야 할 것:

- 임시방편.
- 최소한의 수정.
- 특정 테스트만 통과시키는 패치.
- compatibility layer.
- fallback path.
- dual schema.
- 기존 파일을 옮기면서 새 wrapper/adaptor만 추가하는 방식.
- 테스트가 새 파일 구조만 고정하고 사용자-visible behavior를 검증하지 않는 방식.

레거시는 과감하게 제거한다. 레거시 동작을 살리기 위한 compatibility layer, fallback path, dual schema는 기본적으로 만들지 않는다. 정말 필요하면 이유와 제거 조건을 `PLAN.md`에 기록하고 사용자 확인을 받는다.

## In Scope

1. `combat_preview` 계층 정리.
   - `combat_preview.rs`를 디렉토리 모듈로 승격하거나, 현재 `combat_preview/`와 책임을 명확히 분리한다.
   - DTO, template, threat warning, validation, build/planning 책임을 작은 파일로 분리한다.

2. `src/game/combat_*` 루트 파일 정리.
   - preview/build 관련 파일은 `combat_preview/` 또는 새 `combat_setup/` 계층으로 이동한다.
   - 실제 battle scenario handoff 책임과 preview-only 책임을 구분한다.

3. `world/admin.rs` 분리.
   - 예시 후보:
     - `world/admin/mod.rs`
     - `world/admin/commands.rs`
     - `world/admin/catalog.rs`
     - `world/admin/fixtures.rs`
     - `world/admin/grants.rs`
     - `world/admin/tests.rs`
   - admin command 계약과 result payload는 유지한다.

4. `world/tests.rs` 분리.
   - 예시 후보:
     - `world/tests/support.rs`
     - `world/tests/maintenance.rs`
     - `world/tests/combat.rs`
     - `world/tests/map_flow.rs`
     - `world/tests/equipment.rs`
     - `world/tests/snapshots_and_start.rs`
     - `world/tests/node_sessions.rs`
   - 테스트가 보호하는 behavior는 유지한다.

5. 디버그/산출물/백업 디렉토리 분류.
   - `tests_bak/`, `src/old/`, `debug_event_log_exports/`, `timeline_exports/`, `logs/`, `tmp/`가 공식 fixture인지 단순 부산물인지 판정한다.
   - 단순 부산물은 제거 또는 repo 밖 위치로 이동하는 방안을 기록한다.
   - golden fixture라면 `tests/fixtures/` 같은 명확한 위치로 이동하는 방안을 기록한다.

6. 문서 갱신.
   - 파일 계층과 source-of-truth가 바뀌면 `docs/refactor_preparation_plan.md` 또는 관련 문서를 갱신한다.
   - Unity-facing 계약이 바뀌지 않아야 한다. 만약 바뀐다면 stop condition이다.

## Out Of Scope

- gameplay rule 변경.
- Unity-facing DTO shape 변경.
- WebSocket command/result 계약 변경.
- live RON schema 변경.
- 저장 데이터 migration.
- 밸런스 수치 변경.
- 전투 런타임 정책 변경.
- Movement/Rapier 동작 변경.
- Skill execution/targeting 정책 변경.
- Maintenance/Admin 기능 추가.
- 단순 미관을 위한 파일명 변경.

## Implementation Plan

1. 현재 파일 계층과 의존 관계를 다시 조사한다.
   - `rg`, `cargo metadata`, `cargo test --no-run` 등으로 이동 영향 범위를 확인한다.
   - public module re-export와 server import 경계를 기록한다.

2. 작업을 작은 phase로 나눈다.
   - Phase A: `world/admin.rs` 분리.
   - Phase B: `world/tests.rs` 분리.
   - Phase C: `combat_preview.rs`와 `combat_preview/` 계층 정리.
   - Phase D: 루트 `combat_*` 파일 계층화.
   - Phase E: 디버그/산출물/백업 디렉토리 정책 정리.

3. 각 phase마다 focused test를 먼저 실행한다.
   - behavior 변경 없이 compile/test가 통과하는지 확인한다.
   - 실패 원인과 수정 내용을 `EXPERIMENTS.md`에 기록한다.

4. public 계약을 확인한다.
   - `PlayerBehavior`, `BehaviorResult`, `selected_event`, server mapping, Unity docs가 바뀌지 않았는지 확인한다.
   - 바뀌어야만 하는 지점이 발견되면 임의로 진행하지 않고 goal을 종료한다.

5. 문서와 README를 갱신한다.
   - 새 계층이 source-of-truth 경계를 바꾼다면 문서에 반영한다.
   - 완료된 goal 기록은 현재 정책 문서로 필요한 내용만 옮긴 뒤 정리한다.

## Test Requirements

테스트는 내부 파일 구조가 아니라 사용자-visible behavior와 계약을 고정한다.

필수 검증 후보:

```text
cargo check -p game_core
cargo check -p game_server
cargo test -p game_core admin -- --nocapture
cargo test -p game_server admin -- --nocapture
cargo test -p game_core support -- --nocapture
cargo test -p game_core maintenance -- --nocapture
cargo test -p game_core --test ron_loading -- --nocapture
cargo test -p game_core
cargo test -p game_server
```

필요 시:

```text
ADMIN_COMMAND_TOKEN=dev WS_PORT=18083 python3 "/mnt/f/unity projects/ark/docs/unity_admin_debug_command_probe.py"
```

작은 변경 단위마다 빠른 focused test 또는 `cargo check`를 돌리고, 마지막에 넓은 테스트를 돌린다.

## Completion Conditions

- `src/game` 루트의 전투 preview/setup 관련 책임이 더 명확한 하위 계층으로 정리된다.
- `combat_preview` 계층이 단일 파일 비대화에서 벗어나고, DTO/build/template/validation 책임이 읽기 쉬운 위치에 놓인다.
- `world/admin` 책임이 command/catalog/fixture/grant/test 등으로 분리된다.
- `world` 테스트가 주제별 파일로 나뉘어 새 테스트 추가 위치가 명확하다.
- 디버그/산출물/백업 디렉토리의 공식성 여부가 정리된다.
- public gameplay behavior, Unity-facing DTO, WebSocket command/result 계약이 바뀌지 않는다.
- 필요한 문서가 갱신된다.
- focused tests, `cargo check`, 넓은 테스트가 통과한다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

아래 상황이 발견되면 임의로 결정하지 말고 goal을 종료하고 질문 목록을 보고한다.

- Unity-facing DTO shape 변경이 필요하다.
- WebSocket command/result 계약 변경이 필요하다.
- live RON schema 변경이 필요하다.
- 저장 데이터 migration이 필요하다.
- 테스트/디버그 산출물 삭제가 기존 검증 흐름을 깨뜨릴 수 있다.
- `tests_bak/`, `timeline_exports/`, `debug_event_log_exports/`, `src/old/` 중 무엇을 공식 fixture로 볼지 정책 판단이 필요하다.
- module 이동이 public API 또는 외부 crate import를 바꿔야 한다.
- 리팩토링 중 gameplay behavior 변경이 필요해 보인다.
- compatibility layer나 dual schema가 필요해 보인다.

## Goal Command

```text
/goal docs/core_file_hierarchy_refactor_goal.md를 기준으로, core_enhanced의 파일 계층을 장기적으로 읽기 쉽고 안전하게 정리하라. goal 시작 시 docs/goals/core_file_hierarchy_refactor/PLAN.md, EXPERIMENTS.md, EXPERIMENT_NOTES.md를 만들고 계속 갱신하라. 먼저 docs/refactor_preparation_plan.md, docs/code_documentation_sync_guidelines.md, src/game/mod.rs, src/game/world.rs, src/game/world/*.rs, src/game/combat_preview.rs, src/game/combat_preview/*.rs, src/game/combat_*.rs, src/game/data/mod.rs, src/game/events/combat.rs, tests/, tests_bak/, debug_event_log_exports/, timeline_exports/를 꼼꼼히 읽고 현재 파일 책임과 source-of-truth 경계를 기록하라. 코드 수정은 장기적인 방향으로 하고 임시방편, 최소한의 수정, compatibility layer, fallback path, dual schema, 특정 테스트만 통과시키는 패치를 피하라. 문서를 무조건 신뢰하지 말고 실제 runtime code, live RON/data, Unity-facing snapshot/command 계약, 최신 정책 문서 순서로 source of truth를 확인하라. 레거시는 과감하게 제거하되, 삭제 대상이 live fixture인지 단순 산출물인지 확인하라. 테스트는 내부 파일 구조보다 사용자-visible behavior, Unity-facing DTO, data validation, live RON loading, 실제 gameplay flow를 고정하라. 작업은 작은 phase로 나누어 world/admin 분리, world/tests 분리, combat_preview 계층 정리, 루트 combat_* 파일 계층화, 디버그/산출물/백업 디렉토리 정책 정리 순으로 진행하라. Unity-facing DTO shape, WebSocket command/result 계약, live RON schema, 저장 데이터 migration, UX 의미 변화, 밸런스 기준, 기존 콘텐츠 삭제/대체, 실패/보상/소비 시점 변화처럼 사용자 결정이 필요한 정책이 발견되면 임의로 결정하지 말고 goal을 종료하고 질문 목록을 보고하라. 작은 변경 단위마다 빠른 focused test 또는 cargo check를 돌리고, 마지막에 cargo check -p game_core, cargo check -p game_server, cargo test -p game_core, cargo test -p game_server를 수행하라. 테스트 실패는 EXPERIMENTS.md에 실패 원인, 수정 내용, 재검증 결과를 기록하라. 완료 시 변경 요약, 제거한 레거시, 새로 고정한 계약, 갱신한 테스트, 남은 위험, 실행한 검증 명령을 보고하라. 완료 조건에는 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다를 포함하라.
```
