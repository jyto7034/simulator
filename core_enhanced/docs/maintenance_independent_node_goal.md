# Maintenance Independent Node Goal

이 goal은 현재 `SupportNodeType::Maintenance`로 구현된 Maintenance를 독립적인 맵 노드로 분리하는 계획서다.

현재 core 기준으로 Maintenance 기능 자체는 구현되어 있지만, 노드 모델은 아래처럼 Support 하위 타입에 묶여 있다.

```text
MapNodeCategory::Support
MapNodePayload::Support { support_type: SupportNodeType::Maintenance, ... }
```

새 정책은 Maintenance를 Medical/Rest와 같은 지원 선택지가 아니라, 장비/스킬 파편을 분쇄, 강화, 개화하는 독립 정비 노드로 취급하는 것이다.

## Goal Mode Working Method

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/maintenance_independent_node/PLAN.md
docs/goals/maintenance_independent_node/EXPERIMENTS.md
docs/goals/maintenance_independent_node/EXPERIMENT_NOTES.md
```

각 파일의 역할:

- `PLAN.md`: 구현 순서, 범위, 완료 조건, 중단 조건, 검증 명령을 기록한다.
- `EXPERIMENTS.md`: 시도/실패/성공 결과, 테스트 실패 원인, 수정 내용, 재검증 결과를 기록한다.
- `EXPERIMENT_NOTES.md`: 작업 중 판단, source-of-truth 충돌, 정책 질문, 후속 후보를 기록한다.

## Objective

Maintenance를 Support 하위 타입에서 분리해 독립적인 노드/이벤트 계약으로 만든다.

최종 목표:

```text
MapNodeCategory::Maintenance
MapNodePayload::Maintenance
selected_event.type == "maintenance"
```

Support는 Medical/Rest 중심의 지원 노드로 유지하고, Maintenance 작업 기능은 독립 Maintenance 노드에서만 노출한다.

## Design Boundary

Maintenance는 노드 완료 시 단일 효과를 적용하는 지원 이벤트가 아니다.

Maintenance의 핵심 성격:

- 노드 안에서 여러 번 작업을 수행한다.
- 주요 작업은 스킬 파편/장비의 분쇄, 강화, 개화다.
- 장비/스킬 파편 장착 교체는 정비 중 편의를 위한 부가 기능이다.
- 작업마다 preview, 비용, 획득 재료, 경고, 실행 전 확인이 필요하다.

따라서 Maintenance는 `SupportNodeMode::Known | LimitedChoice | FullChoice` 선택 구조에서 벗어나야 한다.

## Source Of Truth

문서를 먼저 믿지 말고 실제 runtime code를 우선 확인한다.

먼저 읽을 core/runtime code:

- `src/game/map/types.rs`: `MapNodeCategory`, `MapNodePayload`, `SupportNodeType`.
- `src/game/map/session.rs`: `NodeSessionKind`, `NodeSession`.
- `src/game/world/map_content.rs`: map node enter routing.
- `src/game/world/support.rs`: Support/Medical/Rest/Maintenance snapshot and completion behavior.
- `src/game/world/maintenance.rs`: Maintenance operation runtime.
- `src/game/world/helpers.rs`: allowed action scheduling and validation dispatch.
- `src/game/world/snapshot.rs`: Unity-facing `selected_event` shape.
- `src/game/world/admin.rs`: admin fixture node entry.
- `src/game/world/tests.rs`: support/maintenance gameplay tests.
- `src/game/behavior.rs`: `PlayerBehavior`, `BehaviorResult`, Maintenance DTO.
- `../game_server/src/game/player_game_actor/messages.rs`: Unity WebSocket request DTO.
- `../game_server/src/game/player_game_actor/state.rs`: result payload mapping.
- `../game_resources/data/map/node_definitions.ron`: live map node definitions.

Canonical Unity docs location:

```text
/mnt/f/unity projects/ark/docs
```

Update external Unity-facing contracts there after runtime code is changed.

## In Scope

1차 구현 범위:

- `MapNodeCategory::Maintenance` 추가.
- `MapNodePayload::Maintenance` 추가.
- `NodeSessionKind::Maintenance` 추가.
- map node enter flow에서 Maintenance category를 독립 라우팅한다.
- `ActiveNodeContent`에 Maintenance 전용 세션 또는 적절한 독립 표현을 추가한다.
- `selected_event.type == "maintenance"` snapshot을 제공한다.
- 기존 `maintenance_options` DTO와 preview 생성 로직을 독립 Maintenance event에서 재사용하거나 이동한다.
- Maintenance allowed actions를 독립 노드 기준으로 허용한다.
- `SupportNodeType::Maintenance`를 제거하거나 더 이상 runtime/live data에서 사용하지 않게 한다.
- 기존 Maintenance operation commands는 독립 Maintenance 노드에서만 실행되게 한다.
- admin fixture command를 `admin_enter_maintenance` 또는 동등한 독립 Maintenance 진입 명령으로 갱신한다.
- live RON map node definitions에서 Maintenance 노드를 독립 category/payload로 마이그레이션한다.
- Unity-facing docs에 `selected_event.type == "maintenance"`와 request/response 계약을 반영한다.
- Python/admin probe가 독립 Maintenance 노드로 fixture를 만들고 `maintenance_options`를 검증하게 갱신한다.

## Out Of Scope

- Maintenance 작업 경제 밸런스 변경.
- 장비/스킬 파편 강화/분쇄/개화 규칙 변경.
- Unity 화면 구현 자체.
- 저장 데이터 migration. 현재 런 저장 파일을 유지해야 하는 요구가 생기면 사용자와 의논한다.
- Medical/Rest 정책 변경.
- Support 선택 모드 전체 제거. 단, Maintenance는 Support 선택지에서 빠진다.
- 전투/보상/상점/본사 연락 노드 정책 변경.

## Proposed Runtime Shape

권장 core shape:

```rust
pub enum MapNodeCategory {
    Start,
    Combat,
    Support,
    Maintenance,
    HeadquartersContact,
    Shop,
    Boss,
    Reward,
}
```

```rust
pub enum MapNodePayload {
    None,
    Support { ... },
    Maintenance,
    HeadquartersContact { ... },
    Encounter { ... },
    Shop { ... },
    Reward { ... },
}
```

권장 selected event:

```json
{
  "type": "maintenance",
  "node_id": "uuid",
  "maintenance_options": {
    "items": [],
    "materials": []
  }
}
```

Support selected event에는 더 이상 `support_type == "Maintenance"`가 나오지 않아야 한다.

## Implementation Plan

1. 현재 Maintenance runtime surface를 읽고 실제 진입/완료/allowed action/snapshot 흐름을 기록한다.
2. `MapNodeCategory`, `MapNodePayload`, `NodeSessionKind`에 Maintenance를 추가한다.
3. `ActiveNodeContent`에 Maintenance 독립 세션을 추가할지, 기존 Support session을 재사용하지 않는 별도 구조를 만들지 판단한다.
   - 권장: `MaintenanceSessionState { node_id }`처럼 작고 명시적인 타입을 둔다.
4. `world/map_content.rs`에서 Maintenance category를 독립 진입 처리한다.
5. `world/support.rs`에서 Maintenance preview 생성 로직을 독립 Maintenance 경로로 이동하거나 명확한 helper로 분리한다.
6. `is_in_maintenance_support_node` 같은 Support 의존 함수를 Maintenance 독립 판정으로 교체한다.
7. Maintenance operation validation이 독립 Maintenance 노드에서만 통과하도록 수정한다.
8. `SupportNodeType::Maintenance` 제거 가능성을 검토한다.
   - 제거가 가능하면 과감히 제거한다.
   - live data 또는 public DTO migration 때문에 즉시 제거가 어렵다면, 제거 조건과 이유를 `EXPERIMENT_NOTES.md`에 기록하고 사용자 확인을 받는다.
9. Admin command를 갱신한다.
   - 기존 `admin_enter_support { support_type: "Maintenance" }`는 제거하거나 실패하게 한다.
   - 새 `admin_enter_maintenance`를 추가하는 방향을 우선 검토한다.
10. live RON/data를 갱신한다.
11. Unity external docs와 Python probe를 갱신한다.
12. 테스트를 갱신/추가한다.
13. focused tests, cargo check, 가능하면 live WebSocket probe를 실행한다.

## Test Requirements

테스트는 아래를 고정한다.

- `MapNodeCategory::Maintenance` 노드에 진입하면 `selected_event.type == "maintenance"`가 된다.
- Maintenance selected event에 `maintenance_options`가 존재한다.
- Maintenance allowed actions가 독립 Maintenance 노드에서 허용된다.
- Support selected event에는 더 이상 Maintenance가 선택지로 나오지 않는다.
- `SupportNodeType::Maintenance`가 제거되거나 live/runtime에서 사용 불가 상태임을 테스트한다.
- Maintenance 작업 command는 Maintenance 노드에서 성공하고, Medical/Rest Support 노드에서는 실패한다.
- 장비 분쇄/강화, 스킬 파편 분쇄/강화/개화 기존 기능이 독립 Maintenance 노드에서도 유지된다.
- admin fixture로 Maintenance에 진입할 수 있다.
- Python probe가 독립 Maintenance fixture로 `maintenance_options`를 검증한다.
- live RON loading이 독립 Maintenance 노드 정의를 통과한다.

권장 검증 명령:

```text
cargo test -p game_core maintenance -- --nocapture
cargo test -p game_core support -- --nocapture
cargo test -p game_core --test ron_loading -- --nocapture
cargo test -p game_server maintenance -- --nocapture
cargo check -p game_core
cargo check -p game_server
```

가능하면 live probe:

```text
ENABLE_ADMIN_COMMANDS=true ADMIN_COMMAND_TOKEN=dev cargo run -p game_server
ADMIN_COMMAND_TOKEN=dev WS_PORT=8081 python3 "/mnt/f/unity projects/ark/docs/unity_admin_debug_command_probe.py"
```

## Completion Conditions

- Maintenance가 Support 하위 타입이 아니라 독립 map node category/payload로 표현된다.
- `selected_event.type == "maintenance"` 계약이 runtime snapshot과 Unity docs에 반영된다.
- Maintenance operations는 독립 Maintenance 노드에서만 실행된다.
- Support node choices에서 Maintenance가 제거되거나 runtime에서 사용되지 않는다.
- live RON/data가 새 Maintenance 노드 구조로 갱신되어 로딩 테스트를 통과한다.
- admin fixture/probe가 독립 Maintenance 노드를 사용한다.
- 기존 장비/스킬 파편 정비 기능이 새 노드에서도 유지된다.
- Focused tests와 cargo check가 통과한다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

아래 사항이 발견되면 임의로 결정하지 말고 goal을 종료한 뒤 사용자에게 질문한다.

- 기존 저장 데이터 migration이 필요해지는 경우.
- Unity가 당장 `support_type == "Maintenance"` 계약을 유지해야 한다는 외부 제약이 확인되는 경우.
- Maintenance가 Support 선택지로도 남아야 한다는 정책 요구가 생기는 경우.
- `SupportNodeMode::LimitedChoice | FullChoice`에서 Maintenance를 계속 선택 가능하게 해야 하는 요구가 생기는 경우.
- live RON schema를 한 번에 바꾸기 어려워 compatibility layer가 필요해지는 경우.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Goal Command

```text
/goal Maintenance를 Support 하위 타입에서 분리해 독립적인 맵 노드로 구현한다.

목표 문서:
- docs/maintenance_independent_node_goal.md

Goal working directory:
- docs/goals/maintenance_independent_node/

시작 시 반드시 아래 작업 기억장치를 만들고 계속 갱신하라.
- docs/goals/maintenance_independent_node/PLAN.md
- docs/goals/maintenance_independent_node/EXPERIMENTS.md
- docs/goals/maintenance_independent_node/EXPERIMENT_NOTES.md

코드 수정은 장기적인 방향으로 하라. 임시방편, 최소한의 수정, compatibility layer, fallback path, dual schema, 특정 테스트만 통과시키는 패치를 피하라.
문서를 무조건 신뢰하지 말고 실제 runtime code, live RON/data, Unity-facing snapshot/command 계약, 최신 정책 문서 순서로 source of truth를 확인하라.
Maintenance는 Medical/Rest와 같은 Support 선택지가 아니라 독립 정비 노드다. `selected_event.type == "maintenance"` 계약을 목표로 한다.
Unity-facing 계약 문서는 외부 canonical 위치인 `/mnt/f/unity projects/ark/docs`에 갱신하라.
git restore, git reset 등 git file modifier 명령은 수행하지 않는다.
git의 과거 내역 코드를 현재 파일에 cp하지 않는다.
파일을 과거 상태로 돌려야 한다고 판단하면 이유를 설명하고 사용자 허락을 먼저 구한다.
사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
```
