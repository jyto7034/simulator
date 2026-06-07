# Unity Snapshot Contract Goal

이 goal은 Unity 클라이언트가 전투 필드와 직원 배치 UI를 추론 없이 구현할 수 있도록 `game_core` snapshot 계약을 보강하는 작업 지침서다.

현재 Unity는 `combat_preview.valid_tiles`, `deployment_zones`, `spawn_zones`, `routes`, `spawn_waves`, `obstacles`를 받을 수 있다. 하지만 3D tile-block battlefield를 만들기에는 다음 정보가 부족하다.

- 각 tile이 `Ground`, `Platform`, `Obstacle` 중 무엇인지 한 번에 읽을 수 있는 canonical tile list.
- 직원이 `Ground`, `Platform`, `Any` 중 어디에 배치 가능한지 나타내는 Unity-facing field.

이 정보가 없으면 Unity는 `deployment_zones`에서 platform을 추론하거나, `valid_tiles`를 전부 ground로 간주해야 한다. 이는 장기적으로 잘못된 방향이다. core가 전장/배치 규칙의 source of truth가 되어야 한다.

## Goal Mode Working Method

이 문서는 `C:\Users\blast\Desktop\using-codex-goals-effectively-ko.md`의 goal 운용 원칙을 따른다.

작업 중 필요하면 다음 파일을 만들고 갱신한다.

```text
docs/goals/unity_snapshot_contract/PLAN.md
docs/goals/unity_snapshot_contract/EXPERIMENTS.md
docs/goals/unity_snapshot_contract/EXPERIMENT_NOTES.md
```

이 goal은 scope가 작으므로 위 파일이 반드시 필요하지는 않다. 단, 예상보다 작업이 길어지거나 테스트 실패 원인 분류가 필요해지면 즉시 작성한다.

## Source Of Truth

우선 읽을 파일:

- `src/game/world/snapshot.rs`: Unity-facing run snapshot JSON 생성.
- `src/game/combat_preview.rs`: `CombatPreview`, `DeploymentZone`, battlefield template parsing.
- `src/game/battle/types.rs`: `DeploymentAffinity`, `UnitCombatProfile`.
- `src/game/world/combat.rs`: deployment zone kind와 affinity 검증.
- `docs/unity_core_contract.md`: Unity 클라이언트 계약 문서.
- `tests/ron_loading.rs`, `src/game/world/tests.rs`, `src/game/combat_preview.rs` tests: live contract 회귀 테스트 후보.

문서보다 코드가 최종 기준이다. 하지만 Unity-facing 계약 변경은 반드시 문서와 테스트에 같이 반영한다.

## Objective

Unity가 다음을 임의 추론하지 않아도 되도록 snapshot 계약을 보강한다.

1. 각 전투 tile의 표시/판정용 kind.
2. 각 직원의 배치 허용 타입.
3. 기존 `valid_tiles`, `deployment_zones`, `obstacles`와 새 canonical field 사이의 관계.

## Required Contract Changes

### CombatPreview Tiles

`CombatPreview`에 Unity 렌더링용 canonical tile list를 추가한다.

권장 JSON shape:

```json
{
  "tiles": [
    { "position": { "x": 0, "y": 0 }, "kind": "Ground" },
    { "position": { "x": 1, "y": 0 }, "kind": "Platform" },
    { "position": { "x": 2, "y": 0 }, "kind": "Obstacle" }
  ]
}
```

권장 Rust 타입:

```rust
pub enum BattlefieldTileKind {
    Ground,
    Platform,
    Obstacle,
}

pub struct BattlefieldTile {
    pub position: Position,
    pub kind: BattlefieldTileKind,
}
```

계약 규칙:

- `tiles`는 `valid_tiles` 안에 실제로 존재하는 tile만 포함한다.
- `Void`는 `tiles`에 포함하지 않는다.
- `Obstacle`은 `valid_tiles` 안에 있지만 이동/배치 불가인 blocker tile이다.
- `Platform`은 enemy movement에는 blocker로 취급되는 배치용 높은 타일이다.
- `Ground`는 일반 지형이다.
- `obstacles` 배열은 당장 제거하지 않는다. 기존 core 로직과 호환을 위해 유지하되, `tiles.kind == Obstacle`과 일관되어야 한다.
- `deployment_zones.kind`는 배치 가능한 zone overlay의 kind다. tile kind의 source of truth로 사용하지 않는다.
- Unity는 `deployment_zones`에서 platform tile을 추론하지 않는다.

초기 구현 정책:

- 현재 ASCII template parser가 ground/platform/obstacle을 이미 구분한다면 그 정보를 `tiles`로 보존한다.
- parser에 명시 tile kind가 없고 `deployment_zones.kind`만 있다면, Unity 계약을 위해 더 명확한 parser/source 구조를 추가한다.
- 장기적으로 `valid_tiles + obstacles + deployment_zones`에서 매번 tile kind를 재구성하는 helper를 흩뿌리지 않는다. `CombatPreview` 생성 시 canonical `tiles`를 한 번 구성한다.

### Employee Deployment Affinity

roster snapshot의 `combat_profile`에 `deployment_affinity`를 노출한다.

권장 JSON shape:

```json
{
  "combat_profile": {
    "deployment_affinity": "GroundOnly"
  }
}
```

또는 기존 serde snake_case를 따른다면:

```json
{
  "combat_profile": {
    "deployment_affinity": "ground_only"
  }
}
```

구현 시 현재 Unity 계약의 enum casing 관례를 확인하고 하나로 정한다. 이미 `DeploymentAffinity`는 `#[serde(rename_all = "snake_case")]`이므로 직접 serialize하면 `ground_only`, `platform_only`, `any`가 된다. 새 field는 이 serde 결과를 그대로 사용하는 것을 우선한다.

계약 규칙:

- Unity는 이 값을 보고 배치 UI를 미리 보조할 수 있다.
- 최종 배치 성공/실패는 항상 core command validation이 결정한다.
- 직원 기본값은 현재 `GroundOnly`다.
- 원거리/힐러 등 플랫폼 전용 직원이 필요하면 직원 데이터/프로필에서 `PlatformOnly`를 설정할 수 있게 확장한다. 단, 이번 goal은 snapshot 노출이 우선이며 신규 직업 밸런스 설계는 범위 밖이다.

## In Scope

- `CombatPreview`/`BattlefieldInstance`에 canonical `tiles` 추가.
- ASCII battlefield template parsing 또는 preview generation에서 tile kind를 보존/생성.
- `roster.employees[*].combat_profile.deployment_affinity` snapshot 노출.
- `docs/unity_core_contract.md` 갱신.
- focused tests 추가/수정.
- 기존 field를 제거하지 않고 새 명시 계약을 추가한다.

## Out Of Scope

- Unity 클라이언트 구현.
- 전투 맵 최종 아트/프리팹 제작.
- 직원 직업/역할 밸런스 설계.
- 모든 직원 데이터의 플랫폼 전용 재분류.
- `valid_tiles`, `deployment_zones`, `obstacles` 제거.
- Boss 전투 정책.

## Implementation Plan

1. 현재 ASCII battlefield template parser가 어떤 문자로 ground/platform/obstacle/spawn/deploy/route를 표현하는지 확인한다.
2. `BattlefieldTileKind`, `BattlefieldTile` 타입을 추가한다.
3. `ParsedBattlefieldTemplate`, `BattlefieldInstance`, `CombatPreview`에 `tiles`를 추가한다.
4. template parsing 단계에서 `tiles`를 생성한다.
5. `obstacles`와 `tiles.kind == Obstacle`의 일관성을 검증한다.
6. `CombatPreview::try_generate_for_node`가 `tiles`를 snapshot까지 전달하게 한다.
7. `get_employee_roster_snapshot_json`의 `combat_profile`에 `deployment_affinity`를 추가한다.
8. `docs/unity_core_contract.md`의 CombatPreview와 Roster sections를 갱신한다.
9. focused tests로 JSON shape와 계약 불변식을 검증한다.

## Test Requirements

최소 테스트:

- CombatPreview JSON에 `tiles`가 포함된다.
- `tiles`는 `valid_tiles` 밖 좌표를 포함하지 않는다.
- `obstacles`의 모든 좌표는 `tiles.kind == Obstacle`로 표현된다.
- platform deployment zone fixture가 있다면 platform tile이 `tiles.kind == Platform`으로 표현된다.
- roster snapshot의 각 직원 `combat_profile.deployment_affinity`가 노출된다.
- Unity 계약 문서의 예시와 실제 JSON casing이 일치한다.

권장 focused test 후보:

- `combat_preview` module test: ASCII template parsing to tile kinds.
- `world::tests::map_flow::combat_node_preview_exposes_basic_briefing` 확장 또는 새 테스트.
- `world::tests::snapshots_and_start::employee_roster_snapshot_exposes_status_loadout_and_roster_slot` 확장 또는 새 테스트.

검증 순서:

```text
cargo test -p game_core combat_preview -- --nocapture
cargo test -p game_core employee_roster_snapshot -- --nocapture
cargo test -p game_core -- --nocapture
cargo check -p game_core
```

필요하면 `game_server` mapping도 확인한다. snapshot JSON은 core에서 생성되므로 보통 별도 server 변경은 필요 없지만, 서버 테스트가 snapshot shape를 고정한다면 갱신한다.

## Stop Conditions

다음 경우 임의 확정하지 말고 goal을 종료하고 질문 목록을 보고한다.

- ASCII template 문법에서 ground/platform/obstacle을 구분할 수 없다.
- platform tile을 `deployment_zones.kind`에서 역추론해야만 하는 구조다.
- `Obstacle`을 valid tile로 유지할지, valid tile 밖 blocker로 볼지 정책 충돌이 있다.
- enum casing을 바꾸면 Unity-facing 기존 계약과 충돌한다.
- 직원별 `deployment_affinity`를 데이터로 설정할 위치가 없어 신규 정책 결정이 필요하다.

## Completion Criteria

1. Unity가 tile kind를 추론하지 않아도 되는 `combat_preview.tiles`가 snapshot에 포함된다.
2. Unity가 직원 배치 가능 타입을 추론하지 않아도 되는 `combat_profile.deployment_affinity`가 snapshot에 포함된다.
3. `valid_tiles`, `deployment_zones`, `obstacles`, `tiles`의 관계가 코드와 문서에서 일관된다.
4. `docs/unity_core_contract.md`가 최신 JSON shape와 casing을 설명한다.
5. focused tests와 `cargo test -p game_core -- --nocapture`, `cargo check -p game_core`가 통과한다.
6. 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Goal Command

```text
/goal F:\work\simulator\core_enhanced\docs\unity_snapshot_contract_goal.md를 기준으로, Unity 클라이언트가 전투 필드와 직원 배치 UI를 추론 없이 구현할 수 있도록 game_core snapshot 계약을 보강하라. 먼저 src/game/world/snapshot.rs, src/game/combat_preview.rs, src/game/battle/types.rs, src/game/world/combat.rs, docs/unity_core_contract.md를 읽고 현재 CombatPreview/roster snapshot shape를 확인하라. CombatPreview에는 Unity 렌더링용 canonical tiles 배열을 추가하라. 각 tile은 position과 kind를 가지며 kind는 Ground/Platform/Obstacle을 표현하고, Void는 tiles에 포함하지 않는다. 기존 valid_tiles, deployment_zones, obstacles는 제거하지 말고 유지하되, tiles와의 관계를 명확히 검증하라. Unity가 deployment_zones에서 Platform을 추론하지 않아도 되게 하라. roster.employees[*].combat_profile에는 deployment_affinity를 노출하라. DeploymentAffinity는 기존 serde snake_case가 있으면 그 결과를 우선 사용하고, 문서 예시와 실제 JSON casing을 일치시켜라. 직원 직업/밸런스 재설계, Unity 구현, 기존 field 제거는 범위 밖이다. focused tests로 CombatPreview tiles, obstacle/tile consistency, platform tile fixture, roster deployment_affinity snapshot 노출을 검증하고 docs/unity_core_contract.md를 갱신하라. 문서를 무조건 신뢰하지 말고 실제 코드 기준으로 더 나은 개선안이 있으면 근거를 기록하고 적용하라. platform tile을 source data에서 구분할 수 없거나, Obstacle/valid tile 정책 충돌, enum casing 충돌, 직원별 deployment_affinity 데이터 위치처럼 사용자와 의논해야 할 정책이 있으면 goal을 종료하고 질문 목록을 보고하라. 검증은 관련 focused test 후 cargo test -p game_core -- --nocapture, cargo check -p game_core 순서로 수행하라. 레거시는 compatibility layer로 감싸지 말고 제거하되, 기존 Unity-facing 필드는 이유 없이 삭제하지 마라.
```
