# Unity Core Contract

이 문서는 Unity 클라이언트가 `game_server`의 `/game` WebSocket을 통해 `game_core`와 연동할 때 알아야 하는 최소 계약을 정리한다.

목표는 Unity가 Rust 내부 구조를 몰라도 다음을 안정적으로 처리하는 것이다.

- 현재 게임 상태를 렌더링한다.
- `allowed_actions`를 기준으로 가능한 버튼/입력을 연다.
- 명령을 보낸 뒤 `CommandResult`와 최신 `StateSnapshot`을 반영한다.
- DefenseRoute 실시간 전투에서는 tick delta와 snapshot을 함께 소비한다.

## Source Of Truth

계약을 수정할 때 우선 확인해야 하는 파일은 다음이다.

- `../game_server/src/game/player_game_actor/messages.rs`: WebSocket client/server message envelope.
- `../game_server/src/game/player_game_actor/state.rs`: `BehaviorResult` -> JSON payload mapping.
- `src/game/behavior.rs`: `PlayerBehavior`, `BehaviorResult`, live battle DTO.
- `src/game/world/snapshot.rs`: 전체 run snapshot JSON.
- `src/game/combat_preview.rs`: 전투 노드 preview, 전장 tile/route/wave/deployment zone.
- `src/game/battle/timeline.rs`: 전투 timeline event contract.

문서보다 코드가 항상 최종 기준이다. 이 문서와 코드가 충돌하면 코드를 읽고 문서를 갱신한다.

## WebSocket Endpoint

```text
GET /game
```

클라이언트는 텍스트 JSON frame만 사용한다. Binary frame은 지원하지 않는다.

인증은 현재 mock 수준이다. `player_id`가 nil UUID가 아니면 통과한다.

```json
{
  "type": "auth",
  "player_id": "00000000-0000-0000-0000-000000000001",
  "token": null
}
```

인증 성공 시 서버는 순서대로 보낸다.

```json
{ "type": "authed", "player_id": "..." }
```

```json
{
  "type": "state_snapshot",
  "state": {}
}
```

## Client Messages

최상위 client message는 `type`을 `snake_case`로 보낸다.

### Auth

```json
{
  "type": "auth",
  "player_id": "uuid",
  "token": null
}
```

### Command

```json
{
  "type": "command",
  "request_id": "client-generated-id",
  "behavior": {
    "type": "start_new_game"
  }
}
```

`request_id`는 Unity가 만든 문자열이다. 서버 응답의 `CommandResult.request_id`와 매칭한다.

### Ping / Quit

```json
{ "type": "ping" }
```

```json
{ "type": "quit" }
```

## Server Messages

서버 message도 최상위 `type`은 `snake_case`다.

### StateSnapshot

```json
{
  "type": "state_snapshot",
  "state": {}
}
```

Unity는 이 snapshot을 가장 신뢰해야 한다. 명령 결과 payload는 즉시 UI 반응에 쓰고, 최종 화면 상태는 뒤따라오는 snapshot으로 동기화한다.

### CommandResult

```json
{
  "type": "command_result",
  "request_id": "client-generated-id",
  "ok": true,
  "result_type": "NodePreview",
  "payload": {}
}
```

`result_type`은 Rust `BehaviorResult` variant 이름 그대로 PascalCase다.

현재 서버는 성공 시에만 `command_result`를 보낸다. 실패는 `ok=false` command result가 아니라 별도 `error` message로 온다.

### Error

```json
{
  "type": "error",
  "request_id": "client-generated-id",
  "code": "invalid_action",
  "message": "Action is not allowed in the current state"
}
```

`request_id`는 command 처리 중 발생한 오류에만 포함된다. Auth 실패, 잘못된 message format, 서버 tick 오류처럼 특정 command와 매칭되지 않는 오류에서는 `request_id` 필드가 생략될 수 있다.

주요 error code:

- `not_authenticated`: auth 전에 command를 보냄.
- `invalid_message_format`: JSON shape 또는 enum 값이 틀림.
- `stale_session`: 같은 player의 이전 socket에서 명령을 보냄.
- `invalid_action`: 현재 `game_state`에서 허용되지 않는 행동.
- `insufficient_resources`: 자원 부족.
- `position_occupied`: 배치 위치 점유.
- `unit_not_found`: 직원 또는 유닛을 찾을 수 없음.
- `not_implemented`: 아직 core에 없는 기능 호출.

### Notification

현재 주요 notification은 실시간 전투 tick이다.

```json
{
  "type": "notification",
  "notification_type": "battle_delta",
  "payload": {
    "result_type": "BattleAdvanced",
    "payload": {}
  }
}
```

서버는 `battle_delta` 뒤에 최신 `state_snapshot`도 push한다.

## Snapshot Root Shape

`state_snapshot.state`의 최상위 구조는 다음과 같다.

```json
{
  "game_state": "viewing_map",
  "game_state_context": {},
  "allowed_actions": ["SelectMapNode"],
  "run_progression": {},
  "map_progression": {},
  "map": {},
  "current_node_session": null,
  "selected_event": null,
  "roster": {},
  "roster_order": {},
  "inventory": {},
  "resources": {}
}
```

Unity UI는 `game_state_context.type`과 `allowed_actions`를 기준으로 화면/버튼을 열어야 한다.

`allowed_actions`는 `ActionKind` variant 이름 그대로 PascalCase다. 예: `StartNewGame`, `SelectMapNode`, `DeployUnit`.

`game_state`는 `game_state_context.type`과 같은 snake_case 상태 문자열이다. 화면 분기는 `game_state_context.type`을 우선 사용한다.

`selected_event`라는 key 이름은 과거 명칭이지만 client-facing 계약으로 유지된다. 의미상 현재 활성 노드 콘텐츠다.

## Game State Context

`game_state_context.type`은 Unity 화면 라우팅의 1차 기준이다.

```text
not_started
selecting_starter_employees
viewing_map
node_confirm
in_node
in_shop
in_reward
in_reward_claimed
combat_result
in_battle
game_over
run_complete
run_failed
```

주요 context:

```json
{
  "type": "selecting_starter_employees",
  "required_count": 3,
  "candidates": []
}
```

```json
{
  "type": "node_confirm",
  "node_id": {},
  "kind_id": "combat_low",
  "category": "Combat",
  "combat_preview": {}
}
```

```json
{
  "type": "in_battle",
  "battle_uuid": "uuid",
  "node_type": "Defense",
  "mission_variant": "Defense",
  "encounter_id": "defend_black_box_relay",
  "combat_preview": {},
  "last_pushed_timeline_seq": 10,
  "deployment": {},
  "playback": {
    "paused": false,
    "speed": "X1"
  }
}
```

## Common Types

### UUID

UUID는 JSON string이다.

### Position

```json
{ "x": 3, "y": 4 }
```

전투 preview의 tile 좌표는 정수 grid다. Unity에서 world 좌표로 변환하는 것은 클라이언트 책임이다.

### FacingDirection

`snake_case` 문자열이다.

```text
up
right
down
left
```

### MapNodeCategory

PascalCase 문자열이다.

```text
Start
Combat
Support
HeadquartersContact
Shop
Boss
Reward
```

### BattlePlaybackSpeed

```text
X0_5
X1
X2
X3
```

## Core Flow

### 1. Start

```json
{
  "type": "command",
  "request_id": "start-1",
  "behavior": {
    "type": "start_new_game"
  }
}
```

성공 result:

```text
StartNewGame
```

payload:

```json
{
  "candidates": [],
  "required_count": 3
}
```

### 2. Select Starter Employees

```json
{
  "type": "command",
  "request_id": "starter-1",
  "behavior": {
    "type": "select_starter_employees",
    "candidate_ids": ["candidate_a", "candidate_b", "candidate_c"]
  }
}
```

성공 result:

```text
StarterEmployeesSelected
```

payload:

```json
{
  "selected_candidate_ids": [],
  "employee_uuids": [],
  "map": {}
}
```

### 3. Map

현재 맵 요청:

```json
{
  "type": "command",
  "request_id": "map-1",
  "behavior": {
    "type": "request_map_data"
  }
}
```

노드 선택:

```json
{
  "type": "command",
  "request_id": "node-1",
  "behavior": {
    "type": "select_map_node",
    "node_id": "..."
  }
}
```

성공 result:

```text
NodePreview
```

전투 노드면 `combat_preview`가 포함된다.

`NodePreview` 이후 상태는 `node_confirm`이다. 이 상태는 별도 잠금 화면이 아니라 Node Map의 preview-ready 상태다. Unity는 Node Map을 유지한 채 preview 패널을 갱신하고, `ConfirmEnterNode`가 `allowed_actions`에 있을 때만 진입 버튼을 활성화한다.

`node_confirm` 상태에서도 다른 available node를 클릭할 수 있다. 이때 Unity는 다시 `select_map_node`를 보내고, core는 기존 preview/session을 새 노드의 `NodePreview`로 교체한다. `ConfirmEnterNode`는 항상 마지막으로 preview된 노드에 진입한다.

노드 진입:

```json
{
  "type": "command",
  "request_id": "enter-1",
  "behavior": {
    "type": "confirm_enter_node"
  }
}
```

선택 취소:

```json
{
  "type": "command",
  "request_id": "cancel-node-1",
  "behavior": {
    "type": "cancel_selected_node"
  }
}
```

노드 처리 완료:

```json
{
  "type": "command",
  "request_id": "complete-node-1",
  "behavior": {
    "type": "complete_node"
  }
}
```

## Combat Preview Contract

전투 노드의 `combat_preview`는 Unity가 전장 UI를 만들기 위한 계약이다.

```json
{
  "node_id": "...",
  "encounter_id": "defend_black_box_relay",
  "battlefield_template_id": "defense_route_small_a",
  "node_type": "Defense",
  "mission_variant": "Defense",
  "mission_risk": "Controlled",
  "archetype": "Corridor",
  "size_class": "Small",
  "width": 10,
  "height": 8,
  "tiles": [],
  "valid_tiles": [],
  "deployment_zones": [],
  "spawn_zones": [],
  "routes": [],
  "spawn_waves": [],
  "obstacles": [],
  "enemy_briefing": [],
  "threat_warnings": []
}
```

Unity 규칙:

- `width`, `height`: grid bounds.
- `tiles`: Unity 전장 조립의 기준이 되는 canonical tile list. 각 tile은 `position`과 `kind`를 가진다.
- `valid_tiles`: 이동/배치/전투가 가능한 전체 tile.
- `obstacles`: 사용 가능한 tile 중 장애물이 있는 tile.
- `deployment_zones`: 배치 가능한 tile 묶음.
- `spawn_zones`: 적 출현 후보/표시 구역.
- `routes`: DefenseRoute 적 이동 경로.
- `spawn_waves`: 시간 기반 적 wave.
- `enemy_briefing`: UI 요약용 적 정보.
- `threat_warnings`: 노드 미리보기 경고문용 항목. Unity는 세부 적 스탯 숫자를 노출하지 않고 이 항목을 사람이 읽을 문구로 변환한다.

`tiles`가 Unity 렌더링의 source of truth다. `valid_tiles`, `deployment_zones`, `obstacles`는 기존 흐름과 검증용 계약으로 유지하지만, Unity는 Platform/Obstacle 여부를 이 필드들에서 추론하지 않는다.

### BattlefieldTile

```json
{
  "position": { "x": 1, "y": 2 },
  "kind": "Ground"
}
```

`kind`:

```text
Ground
Platform
Obstacle
```

- `Ground`: 근거리/지상 배치와 적 이동의 기본 지형.
- `Platform`: 원거리/힐러 등 platform 배치용 지형. 적 이동 경로로 쓰지 않는다.
- `Obstacle`: Unity가 3D 장애물 cube로 그릴 지형. `obstacles` 배열과 같은 좌표 집합이어야 한다.
- `Void`: 별도 kind로 내려오지 않는다. `tiles`에 없는 좌표가 void다.

### DeploymentZone

```json
{
  "id": "ground_a",
  "label": "Ground A",
  "kind": "Ground",
  "cells": [{ "x": 1, "y": 2 }]
}
```

`kind`:

```text
Ground
Platform
```

직원의 배치 허용 타입(`roster.employees[*].combat_profile.deployment_affinity`)과 zone kind 검증은 core가 수행한다. Unity는 잘못된 타일을 미리 막아주면 좋지만, 최종 권한은 core에 있다.

### BattlefieldRoute

```json
{
  "id": "lane_a",
  "start": { "x": 0, "y": 3 },
  "end": { "x": 9, "y": 3 },
  "cells": []
}
```

DefenseRoute에서 적은 route를 따라 이동한다.

### SpawnWave

```json
{
  "id": "wave_1",
  "time_ms": 10000,
  "spawn_zone_ids": ["entry_a"],
  "route_id": "lane_a",
  "enemy_entries": [],
  "required_for_victory": true
}
```

전투 노드 미리보기는 제한 자원 소비로 업그레이드되지 않는다. Unity는 기본 브리핑으로 받은 `spawn_zones`, `routes`, `spawn_waves`, `enemy_briefing`, `threat_warnings`를 표시하고, 정확한 기믹/웨이브 학습은 live battle 관찰과 전투 기록 UI로 보완한다.

`threat_warnings` shape:

```json
{
  "tag": "high_magic_resist_enemy_possible",
  "status": "Unverified",
  "source": "Briefing"
}
```

- `status`: `Unverified`, `Disproved`.
- `source`: `Briefing`, `Rumor`.
- `Disproved` 경고는 같은 이상현상 재진입 preview에서 취소선 처리한다.
- `Rumor` 경고는 20% 확률로 최대 1개 등장하는 루머성 오경고다. core는 실제 resolved spawn wave 경고에 없는 사전 정의 후보 중 seed 기반으로 선택하며, Unity는 즉석 생성/랜덤화를 하지 않는다.
- 플레이어가 전투에 진입한 뒤 퇴각하면 같은 이상현상 재진입 preview에서 false rumor가 `Disproved`로 내려올 수 있다.
- 실제 경고는 `Unverified`로 유지한다.

초기 `threat_warnings[*].tag` 후보:

| tag | 표시 예시 |
| --- | --- |
| `armored_enemy_possible` | 장갑형 적 출현 가능 |
| `high_magic_resist_enemy_possible` | 마법 저항이 높은 적 출현 가능 |
| `air_enemy_possible` | 공중 적 출현 가능 |
| `hard_to_block_enemy_possible` | 저지하기 어려운 적 출현 가능 |
| `shielded_enemy_possible` | 보호막 적 출현 가능 |
| `regenerating_enemy_possible` | 재생 적 출현 가능 |
| `fast_breakthrough_enemy_possible` | 빠른 돌파 적 출현 가능 |

경고 항목은 core/preview가 내려주는 표시 계약이다. Unity는 적 스탯을 재계산해 경고를 만들지 않는다.

## DefenseRoute Live Battle

DefenseRoute 전투는 공식 전투 모드이며, 실시간 command + server tick delta 방식이다.

현재 live battle로 바로 시작되는 조합은 다음이다.

```text
node_type=Defense, mission_variant=Defense
node_type=Defense, mission_variant=Encirclement
```

`Defense/SplitRoom`은 추후 구현 대상이며 live 조우에 배정하지 않는다. `Boss/Boss`는 TODO이며 향후 `DefenseRoute` 파생 전투로 설계할 가능성이 높다. 공식 전투 노드는 사전 배치 자동전투나 replay-only 흐름을 사용하지 않는다.

### Enter

전투 노드에서 `confirm_enter_node`가 성공하면 state가 `in_battle`이 된다.

초기 배치가 자동으로 되지 않는다. Unity는 `combat_preview.deployment_zones`를 표시하고, 플레이어가 직접 배치 명령을 보내야 한다.

Unity는 전투 시작 후 live deployment 명령으로 직원을 배치한다. 사전 배치 자동전투 payload나 `move_unit` 요청은 공식 계약에 없다.

### Deploy Unit

```json
{
  "type": "command",
  "request_id": "deploy-1",
  "behavior": {
    "type": "deploy_unit",
    "employee_uuid": "uuid",
    "position": { "x": 3, "y": 4 },
    "facing": "right"
  }
}
```

성공 result:

```text
BattleUnitDeployed
```

payload 핵심:

```json
{
  "battle_uuid": "uuid",
  "encounter_id": "defend_black_box_relay",
  "node_type": "Defense",
  "mission_variant": "Defense",
  "playback": { "paused": false, "speed": "X1" },
  "employee_uuid": "uuid",
  "unit_instance_id": "uuid",
  "timeline_delta": [],
  "last_timeline_seq": 12,
  "deployment": {}
}
```

### Withdraw Unit

```json
{
  "type": "command",
  "request_id": "withdraw-1",
  "behavior": {
    "type": "withdraw_unit",
    "employee_uuid": "uuid"
  }
}
```

후퇴한 직원은 즉시 사라지고 `redeploying_units`에 들어간다.

### Activate Skill

```json
{
  "type": "command",
  "request_id": "skill-1",
  "behavior": {
    "type": "activate_skill",
    "employee_uuid": "uuid",
    "skill_id": "fragment_one_sin_penitence",
    "target": {
      "type": "Tile",
      "position": { "x": 5, "y": 3 }
    }
  }
}
```

또는 unit target:

```json
{
  "type": "Unit",
  "unit_instance_id": "uuid"
}
```

주의:

- `SkillCastTarget`의 `type`은 현재 `Tile`, `Unit`처럼 PascalCase다.
- Manual skill은 `activate_skill` 명령으로만 발동된다.
- Auto/Triggered skill은 core의 기존 자동 흐름을 따른다.

### Request Battle State

```json
{
  "type": "command",
  "request_id": "battle-state-1",
  "behavior": {
    "type": "request_battle_state",
    "since_seq": 10
  }
}
```

`since_seq`를 생략하면 가능한 전체 delta를 받는다.

성공 result:

```text
BattleState
```

### Pause / Resume / Speed

```json
{
  "type": "command",
  "request_id": "pause-1",
  "behavior": {
    "type": "pause_battle"
  }
}
```

```json
{
  "type": "command",
  "request_id": "resume-1",
  "behavior": {
    "type": "resume_battle"
  }
}
```

```json
{
  "type": "command",
  "request_id": "speed-1",
  "behavior": {
    "type": "set_battle_speed",
    "speed": "X0_5"
  }
}
```

성공 result:

```text
BattlePlaybackChanged
```

정지 중에는 서버가 빈 `battle_delta`를 계속 보내지 않는다.

### Retreat Battle

```json
{
  "type": "command",
  "request_id": "retreat-1",
  "behavior": {
    "type": "retreat_battle"
  }
}
```

비보스 실시간 전투에서 후퇴한다. 후퇴는 현재 이상현상 진입 시도 1회를 소모하지만, 남은 시도 횟수가 있으면 노드를 즉시 소비하지 않는다. 플레이어는 Safezone/NodeConfirm으로 돌아가 로드아웃, 소비 아이템, 출전 직원, 배치 계획을 조정한 뒤 같은 이상현상에 재진입할 수 있다.

후퇴 가능 시점은 `BattleEnd`가 발생하기 전까지다. 모든 아군 유닛이 전투불능 상태여도 `BattleEnd`가 아직 확정되지 않았다면 후퇴할 수 있다. 일시정지 중, 스킬 시전 중, 웨이브 진행 중에도 후퇴할 수 있지만, `BattleEnd` 이후 결과 처리 상태에서는 후퇴할 수 없다.

같은 이상현상은 최대 3번까지 시도할 수 있다. 퇴각 없이 임무 실패 조건이 확정되거나, 3번의 시도를 모두 사용하면 이상현상은 사라지고 노드가 소비된다. 런 실패는 별도 조건으로만 발생한다.

## Live Battle Deployment DTO

`deployment` shape:

```json
{
  "battle_time_ms": 12000,
  "current_cost": 18,
  "max_cost": 30,
  "base_deploy_cost": 10,
  "cost_per_second": 1,
  "unit_deploy_costs": [
    {
      "employee_uuid": "uuid",
      "base_deploy_cost": 10,
      "effective_deploy_cost": 8
    }
  ],
  "deployed_units": [
    {
      "employee_uuid": "uuid",
      "unit_instance_id": "uuid",
      "facing": "right"
    }
  ],
  "redeploying_units": [
    {
      "employee_uuid": "uuid",
      "ready_at_ms": 30000,
      "deploy_cost": 15
    }
  ]
}
```

UI 표현:

- `current_cost`는 세계관상 공간 안정화치다.
- `base_deploy_cost`는 전투 공통 최초 배치 비용이다. 직원별 현재 표시/검증 비용은 `unit_deploy_costs`를 우선한다.
- `unit_deploy_costs[*].base_deploy_cost`는 해당 직원의 현재 배치 기준 비용이다. 일반 배치면 `base_deploy_cost`, 재배치 중이면 `redeploying_units[*].deploy_cost`와 같은 기준값이다.
- `unit_deploy_costs[*].effective_deploy_cost`는 consumable modifier 등 직원별 비용 변경을 적용한 실제 배치 비용이다. Unity는 배치 버튼/카드 비용 표시에서 이 값을 우선 사용한다.
- `redeploying_units.ready_at_ms` 이후 재배치 가능하다.
- `redeploying_units.deploy_cost`는 재배치 기준 비용이다. 실제 재배치 비용 표시는 같은 직원의 `unit_deploy_costs[*].effective_deploy_cost`를 우선한다.

## Battle Delta

서버 tick으로 받는 notification:

```json
{
  "type": "notification",
  "notification_type": "battle_delta",
  "payload": {
    "result_type": "BattleAdvanced",
    "payload": {
      "battle_uuid": "uuid",
      "encounter_id": "defend_black_box_relay",
      "node_type": "Defense",
      "mission_variant": "Defense",
      "playback": { "paused": false, "speed": "X1" },
      "battle_time_ms": 15000,
      "timeline_delta": [],
      "last_timeline_seq": 42,
      "finished": false,
      "deployment": {}
    }
  }
}
```

`timeline_delta`는 이름은 timeline이지만, 현재 공식 의미는 실시간 전투의 battle event log delta다. 전투 전체를 미리 생성한 replay 조각이 아니라, 서버가 이미 처리한 전투 사건 중 아직 클라이언트가 적용하지 않은 항목이다.

Unity는 `timeline_delta`를 `seq` 기준으로 중복 적용하지 않아야 한다.

권장 방식:

- 클라이언트가 마지막 적용한 `last_timeline_seq`를 저장한다.
- reconnect 또는 누락 의심 시 `request_battle_state { since_seq }`를 보낸다.
- snapshot의 `game_state_context.last_pushed_timeline_seq`는 서버가 마지막으로 push한 기준점이다.

## Battle Event Log Entry

기본 shape:

```json
{
  "time_ms": 1000,
  "seq": 12,
  "cause": {
    "cause_type": "root",
    "kind": "system"
  },
  "event": {
    "type": "UnitSpawned"
  }
}
```

이 항목은 `timeline_delta` 배열의 원소다. Rust 타입과 JSON 문서에는 `Timeline` 이름이 남아 있지만, 현재 역할은 DefenseRoute live battle event log다.

`TimelineEvent.type`은 현재 PascalCase다.

Timeline 내부 casing은 필드마다 다르다.

- `TimelineCause.cause_type`: `root`, `parent`.
- `TimelineRootCause.kind`: `init`, `period`, `system`.
- `TimelineEvent.type`: `BattleStart`, `UnitSpawned` 같은 PascalCase.
- `SkillCastTarget.type`: `Tile`, `Unit`.
- `TimelineSkillAreaShape.type`: DefenseRoute 공식 스킬은 `tile_pattern`만 사용한다. `tile_pattern`은 `affected_tiles: [{x, y}]`를 포함하며 Unity는 이 타일 목록을 그대로 범위 경고/효과 표시로 사용한다. `circle`, `line`, `box`, `rectangle`, `cone` 같은 geometric area 이벤트는 공식 Unity-facing 계약에 포함되지 않는다.
- `FacingDirection`: `up`, `right`, `down`, `left`.

Unity가 우선 처리해야 할 주요 event:

- `BattleStart`: 전투 시작.
- `UnitSpawned`: 유닛 표시 생성.
- `MovementSegmentStarted`: 이동 보간 시작.
- `MovementStopped`: 이동 정지.
- `AttackStart`, `AttackResolve`, `AttackMiss`: 기본 공격 표현.
- `BasicAttackProjectileLaunched`, `BasicAttackProjectileImpacted`: 투사체 표현.
- `ManualCastStart`, `ManualCastEnd`: 수동 스킬 시전 표현.
- `AutoCastStart`, `AutoCastEnd`: 자동 스킬 시전 표현.
- `SkillAreaDeclared`: 범위 경고/효과 표시.
- `SkillProjectileLaunched`, `SkillProjectileImpacted`: 스킬 투사체 표시.
- `BuffApplied`, `BuffTick`, `BuffExpired`: 버프 표시.
- `HpChanged`: 체력 UI 갱신.
- `UnitDied`: 유닛 사망 처리.
- `BattleEnd`: 전투 종료.

`HpChanged` 피해 표시 계약:

```json
{
  "type": "HpChanged",
  "source_instance_id": "attacker-unit-id",
  "target_instance_id": "target-unit-id",
  "delta": -14920,
  "hp_before": 30000,
  "hp_after": 15080,
  "reason": "Damage",
  "damage_source": "BasicAttack",
  "damage_type": "Physical",
  "raw_damage": 17500,
  "final_damage": 14920,
  "damage_breakdown": [],
  "critical": true,
  "feedback_tags": ["critical", "mitigated"]
}
```

- `feedback_tags`는 core가 판정한 전투 표시 태그다. Unity는 방어력/마법 저항 수치를 재계산해 `경감됨`, `약점` 등을 추론하지 않는다.
- `damage_type`은 피해 숫자 색상의 source of truth다. UI 표현에서 AD는 `Physical`, AP는 `Magic`에 대응한다. `Physical`은 붉은 계열, `Magic`은 푸른 계열을 기본으로 표시한다. `True`는 고정 피해 색상으로 분리할 수 있다.
- 피해 숫자 상단 라벨은 `critical`, `mitigated`, `fixed_damage`, `immune`만 사용한다. `weakness` 상단 태그는 초기 계약에서 사용하지 않는다.
- 피해 숫자 좌측 라벨은 그 외 보조 태그에 사용한다. 초기 후보는 `piercing`, `shield`, `blocked`, `resisted_status`다.
- 한 번의 피해에 여러 상단 태그가 있으면 Unity는 메인 상단 라벨 1개만 표시한다. 상단 우선순위는 `immune > fixed_damage > mitigated > critical`이다.
- 세부 적 스탯 도감/숫자 공개는 현재 계약에 포함하지 않는다. NodePreview의 `threat_warnings`, 적 외형, 피해 숫자 색상, `feedback_tags`로 적 성향을 전달한다.

초기 `feedback_tags` 후보:

| tag | 표시 위치 | 표시 예시 | core 판정 기준 |
| --- | --- | --- | --- |
| `critical` | 상단 | 치명타! | 피해 결과의 `critical == true` |
| `mitigated` | 상단 | 경감됨! | `raw_damage > 0`, `final_damage > 0`, `final_damage <= raw_damage * 60%` |
| `fixed_damage` | 상단 | 고정 피해! | 주 피해 타입이 `True` |
| `immune` | 상단 | 면역! | `raw_damage > 0`인데 최종 피해가 0이거나, 상태이상이 완전히 막힘 |
| `piercing` | 좌측 | 관통 | 해당 피해 타입의 관통/저항 관통 modifier가 실제 저항값을 낮춤 |
| `shield` | 좌측 | 보호막 | 피해가 실체 HP보다 보호막/방어막에 먼저 적용됨 |
| `blocked` | 좌측 | 상쇄 | 일회성 방어, 패링, 상쇄 효과가 피해 일부 또는 전부를 지움 |
| `resisted_status` | 좌측 | 저항 | 상태이상이 들어갔지만 지속시간/효과가 감소함 |

## Combat Result

전투가 끝나면 Unity는 결과창을 표시하고, 플레이어가 결과 확인을 누르면 `complete_combat_result`를 보낸다. 전투 노드는 즉시 압축 timeline 결과를 반환하지 않고 live battle state와 delta를 통해 진행된다.

`selected_event.compressed_timeline`에도 결과창/전투 기록용 battle event log payload가 포함될 수 있다. 필드명은 기존 Unity 계약 때문에 유지하지만, 의미는 replay source가 아니라 이미 live battle에서 발생한 사건 로그의 압축본이다.

Unity는 gzip+base64를 풀어 전투 기록 UI, 상세 로그, 디버그 확인에 사용한다. 공식 전투 진행은 이 압축 payload가 아니라 live state와 `timeline_delta` push가 담당한다.

`selected_event.type == "combat_battle"` snapshot 핵심:

```json
{
  "type": "combat_battle",
  "abnormality_id": "one_sin",
  "encounter_id": "defense_route_result",
  "node_type": "Defense",
  "mission_variant": "Defense",
  "abnormality_uuid": "uuid",
  "winner": "Player",
  "reward_mode": "ClaimAll",
  "rewards": [],
  "has_timeline": true,
  "compressed_timeline": {}
}
```

`has_timeline`과 `compressed_timeline`은 레거시 필드명이다. 현재 의미는 `has_event_log`, `compressed_event_log`에 가깝다. Unity-facing DTO 이름 변경은 클라이언트 마이그레이션 정책이 필요하므로 아직 수행하지 않는다.

`compressed_timeline`은 `/game` actor가 snapshot을 만들 때 전투 결과가 있으면 삽입한다. core 내부 snapshot 함수만 직접 호출할 때는 이 필드가 없을 수 있다. 결과 확인은 `complete_combat_result`만 사용한다.

결과 확인:

```json
{
  "type": "command",
  "request_id": "complete-combat-result-1",
  "behavior": {
    "type": "complete_combat_result"
  }
}
```

payload:

```json
{
  "enkephalin": 100,
  "inventory_diff": {},
  "outcome": {},
  "completion": {
    "result_type": "NodeCompleted",
    "payload": {}
  }
}
```

보스/액트 종료/런 종료 상황에서는 `completion.result_type`이 `ActComplete`, `RunComplete`, `RunFailed`일 수 있다.

## Safe Nodes and Safezone

Unity 문서와 UI에서 `Rest`, `Maintenance`, `휴식`, `휴식 정비`, `support`라는 표현이 섞이면 노드 효과와 진입 전 준비 장면이 혼동된다. core 계약에서는 아래처럼 구분한다.

`Safe Node`:

- 맵 위에 존재하는 비전투 노드 계열이다.
- 노드에 들어간 뒤 `complete_node` 또는 해당 노드 전용 command로 해결한다.
- 완료하면 노드가 소비된다.
- 현재 Safe Node 계열은 `Medical`, `Rest`, `Maintenance`, `HeadquartersContact`, `Shop`, `Reward`다.
- 이 중 `Medical`, `Rest`, `Maintenance`만 `SupportState`와 `selected_event.type == "support"`를 사용한다.

`Safezone`:

- core의 별도 map node가 아니라 Unity-facing 진입 전 준비 장면/패널 이름이다.
- 노드를 소비하지 않고, Rest/Maintenance 효과를 지급하지 않는다.
- 플레이어가 선택적으로 진입하는 장소가 아니라 게임 흐름상 노드와 노드 사이에서 항상 거치는 준비 구간이다.
- `StartNewGame` 이후에는 직원 선발이 먼저 진행되고, 직원 선발 완료 후 Unity는 Safezone의 `Node Exploration`으로 들어간다.
- 노드를 소비한 뒤에도 기본 복귀 UI는 Safezone의 `Node Exploration`이다.
- Safezone은 `Node Exploration`, `Item Use`, `Loadout` 세 장면으로 표현한다.
- `Node Exploration`은 다음 노드 preview, 맵 탐색, 진입 후보 확인을 담당한다.
- `Item Use`는 인벤토리와 사용 아이템을 보여주는 장면이다. Unity 하단바 좌측 버튼으로 `Node Exploration`과 전환한다.
- `Item Use` 우측 가방 UI는 기본 4x10 정사각형 슬롯 grid로 표현한다. 아이템이 없어도 빈 슬롯을 표시하고, 추후 가방 크기 변경을 고려해 스크롤 가능한 구조로 둔다.
- `Item Use` 가방 패널 내부, 슬롯 위에는 `소모성`, `무기`, `방어구`, `악세서리`, `스킬 파편` 5종 카테고리 버튼을 둔다. 이 카테고리는 Unity 표시/필터 정책이며 서버 command 계약을 추가하지 않는다.
- `Loadout`은 `Item Use` 장면에서 유닛을 long press했을 때 열리는 장면이다. 장비 장착/해제와 스킬 파편 장착/해제를 담당한다.
- `Loadout`의 Unity UX는 drag-and-drop을 기본으로 한다. 우측 가방의 장비/스킬 파편을 중앙의 호환 장착 슬롯으로 드롭하면 장착 command를 보내고, 중앙 장착 슬롯의 장비/스킬 파편을 우측 가방으로 드롭하면 해제 command를 보낸다. 클릭은 장착/해제가 아니라 선택/미리보기 용도다.
- 호환되지 않는 슬롯에 드롭하면 Unity는 command를 보내지 않고 시각 피드백만 표시한다.
- 현재 계약상 Safezone에서 live 연결할 수 있는 준비 행동은 `use_consumable_item`, `equip_item`, `un_equip_item`, `equip_skill_fragment`, `un_equip_skill_fragment`, 인벤토리/노드 preview 확인이다.
- Safezone은 `Maintenance` 노드가 아니다. `restore_equipment`, `dismantle_equipment`, `enhance_equipment`, `upgrade_skill_fragment`, `awaken_skill_fragment`, `dismantle_skill_fragment`는 `Maintenance` 노드 내부에서만 노출한다.
- `Item Use` 장면은 섭취 아이템을 사망자가 아닌 직원에게 드래그해 다음 전투/보스 노드용 active modifier를 적용하는 live command를 가진다. 사망자인 직원은 Unity에서 사용 불가로 처리한다.

## Support Nodes

지원 노드는 `SupportState`와 `selected_event.type == "support"`를 사용한다.

`selected_event.type == "support"` snapshot 핵심:

```json
{
  "type": "support",
  "node_id": "...",
  "support_mode": "Known",
  "support_type": "Medical",
  "choices": [],
  "selected_support_type": null,
  "target_candidates": [],
  "selected_employee_uuid": null,
  "selected_medical_treatment": null,
  "maintenance_options": null
}
```

`SupportState` command result에도 같은 성격의 필드가 포함되며, 여기에 `research_deliveries`가 추가된다.

지원 타입:

```text
Medical
Rest
Maintenance
```

지원 모드:

```text
Known
LimitedChoice
FullChoice
```

명령:

```json
{
  "type": "command",
  "request_id": "support-1",
  "behavior": {
    "type": "choose_support",
    "support_type": "Maintenance"
  }
}
```

```json
{
  "type": "command",
  "request_id": "support-target-1",
  "behavior": {
    "type": "select_support_target",
    "employee_uuid": "uuid"
  }
}
```

```json
{
  "type": "command",
  "request_id": "medical-1",
  "behavior": {
    "type": "select_medical_treatment",
    "treatment": "BalancedCare"
  }
}
```

의료 처치:

```text
EmergencyCare
Counseling
BalancedCare
```

Maintenance에서는 `maintenance_options`를 보고 가능한 작업만 UI에 표시한다.

`maintenance_options` shape:

```json
{
  "restorable_equipment_recipes": [],
  "dismantle_equipment_item_uuids": [],
  "enhance_equipment_item_uuids": [],
  "enhancement_recipes": [],
  "dismantle_recipes": [],
  "dismantle_skill_fragment_ids": []
}
```

지원 노드 효과 적용 시점:

- `Medical`: 필요하면 `select_support_target`, `select_medical_treatment`를 먼저 보낸 뒤 `complete_node`를 보내야 실제 치료가 적용된다.
- `Rest`: `complete_node` 시 전체 휴식 효과가 적용된다.
- `Maintenance`: 정비 작업은 각 command 시 즉시 적용되고, `complete_node`로 노드를 나간다.

## HeadquartersContact

본사 연락 노드는 한 번 방문할 때 아래 행동 중 하나만 선택한다.

```text
RecruitEmployee
RequestEmergencySupplies
OpenHeadquartersShop
```

`selected_event.type == "headquarters_contact"` snapshot 핵심:

```json
{
  "type": "headquarters_contact",
  "node_id": "...",
  "options": ["RecruitEmployee", "RequestEmergencySupplies", "OpenHeadquartersShop"],
  "recruitment_candidates": [],
  "shop_pool_id": "headquarters_basic"
}
```

`HeadquartersContactState` command result에도 같은 성격의 필드가 포함되며, 여기에 `research_deliveries`가 추가된다.

주의:

- `recruit_employee`, `request_emergency_supplies`, `open_headquarters_shop` 중 하나를 실행하면 본사 연락 노드의 행동권을 소비한다.
- `open_headquarters_shop` 성공 후에는 `in_shop` 상태가 되며, 이후 구매/판매/나가기는 일반 `Shop` 명령을 사용한다.
- `recruit_employee`, `request_emergency_supplies`의 성공 result에는 내부 `completion`이 포함된다. `completion.result_type`은 보통 `NodeCompleted`다.

직원 채용:

```json
{
  "type": "command",
  "request_id": "recruit-1",
  "behavior": {
    "type": "recruit_employee",
    "candidate_id": "candidate_id"
  }
}
```

긴급 보급:

```json
{
  "type": "command",
  "request_id": "supplies-1",
  "behavior": {
    "type": "request_emergency_supplies"
  }
}
```

본사 상점 열기:

```json
{
  "type": "command",
  "request_id": "hq-shop-1",
  "behavior": {
    "type": "open_headquarters_shop"
  }
}
```

## Shop

상점 명령은 항상 WebSocket command envelope로 보낸다.

```json
{
  "type": "command",
  "request_id": "buy-1",
  "behavior": {
    "type": "purchase_item",
    "item_uuid": "uuid"
  }
}
```

```json
{
  "type": "command",
  "request_id": "sell-1",
  "behavior": {
    "type": "sell_item",
    "item_uuid": "uuid"
  }
}
```

```json
{
  "type": "command",
  "request_id": "reroll-1",
  "behavior": {
    "type": "reroll_shop"
  }
}
```

```json
{
  "type": "command",
  "request_id": "shop-exit-1",
  "behavior": {
    "type": "exit_shop"
  }
}
```

`selected_event.type == "shop"`에서 `visible_items`, `hidden_items`, `visible_item_uuids`, `hidden_item_uuids`, `can_reroll`을 표시한다.

상점 관련 result:

```text
ShopState
PurchaseItem
SellItem
RerollShop
NodeCompleted
```

`exit_shop`은 현재 노드 완료 처리를 수행하므로 성공 result가 `NodeCompleted`, `ActComplete`, `RunComplete`, `RunFailed` 중 하나일 수 있다.

## Reward

보상 선택:

```json
{
  "type": "command",
  "request_id": "reward-select-1",
  "behavior": {
    "type": "select_reward",
    "reward_id": "uuid"
  }
}
```

수령:

```json
{
  "type": "command",
  "request_id": "reward-claim-1",
  "behavior": {
    "type": "claim_reward"
  }
}
```

나가기:

```json
{
  "type": "command",
  "request_id": "reward-exit-1",
  "behavior": {
    "type": "exit_reward"
  }
}
```

`selected_event.type == "reward"`에서 `mode`, `rewards`, `selected_reward_uuid`, `can_skip`을 표시한다.

Reward result:

```text
RewardState
RewardGranted
NodeCompleted
```

`mode == "ChooseOne"`이면 먼저 `select_reward`로 하나를 선택한 뒤 `claim_reward`를 보내야 한다. `mode == "ClaimAll"`이면 바로 `claim_reward`가 가능하다.

`exit_reward`는 `can_skip == true`이거나 이미 `in_reward_claimed` 상태일 때만 의미가 있다. 성공 시 현재 노드 완료 처리를 수행한다.

## Inventory And Equipment

`inventory` snapshot 핵심:

```json
{
  "equipments": [],
  "equipment_materials": [],
  "artifacts": [],
  "consumables": [],
  "skill_fragments": [],
  "skill_fragment_progress": [],
  "pending_research_deliveries": [],
  "fragment_dust": 0
}
```

### Roster Order Organization

맵 화면과 노드 확인 화면에서는 직원 명단 표시 순서를 바꿀 수 있다. 이것은 전투 대기석이 아니라 런 중 직원 목록 정렬 상태다.

```json
{
  "type": "command",
  "request_id": "roster-order-move-1",
  "behavior": {
    "type": "move_roster_unit",
    "target_unit_uuid": "employee-uuid",
    "dest_slot": 2,
    "swap_with_unit_uuid": null
  }
}
```

성공 result:

```text
MoveRosterUnit
```

payload:

```json
{
  "roster_slots": [
    { "slot": 0, "unit_uuid": "employee-uuid" },
    { "slot": 1, "unit_uuid": null }
  ]
}
```

장비 장착:

장비 장착/해제는 `ViewingMap`, `NodeConfirm`, Safezone 같은 노드 진입 전 준비 구간에서만 UI로 노출한다. `InBattle`, 전투 결과 처리, Support/Maintenance 내부 작업 결과를 보고 즉석으로 바꾸는 흐름은 만들지 않는다.

```json
{
  "type": "command",
  "request_id": "equip-1",
  "behavior": {
    "type": "equip_item",
    "item_uuid": "uuid",
    "target_unit": "employee-uuid"
  }
}
```

장착 해제:

```json
{
  "type": "command",
  "request_id": "unequip-1",
  "behavior": {
    "type": "un_equip_item",
    "item_uuid": "uuid",
    "target_unit": "employee-uuid"
  }
}
```

장비 metadata의 `bound`가 `true`이면 Safezone에서도 해제할 수 없다. Unity는 inventory item과 roster equipped item의 `can_unequip`, `cannot_unequip_reason`을 사용해 버튼을 비활성화한다. 귀속은 owned instance가 아니라 장비 id/metadata 기준이다.

섭취 아이템 사용:

`use_consumable_item`은 `ViewingMap`과 `NodeConfirm`에서만 허용된다. Unity의 Safezone `Item Use` 장면에서 사망자가 아닌 직원에게 드래그앤드롭이 성공하면 즉시 command를 전송한다. 사망자인 직원은 UI에서 사용 불가로 처리한다.

소비 시점은 전투 진입이 아니라 `use_consumable_item` command 성공 시점이다. 성공 시 item은 inventory에서 즉시 제거되고 대상 직원의 `active_consumable_modifier`가 snapshot에 노출된다. 사용된 아이템은 노드 취소, 퇴각, 재진입, modifier 덮어쓰기 상황에서도 환불되지 않는다.

`NextCombatNode`/`CombatNodes(n)` duration은 전투/보스 노드가 해결될 때만 감소한다. 같은 이상현상에서 퇴각 후 재진입하는 것은 새 전투 노드 해결로 보지 않는다. 남은 시도 횟수가 있는 한 적용된 modifier는 같은 이상현상 재진입에도 유지된다. 퇴각 없이 실패하거나, 성공하거나, 3번의 시도를 모두 사용해 이상현상이 사라지면 해당 전투 노드가 해결된 것으로 보고 duration을 감소시킨다.

```json
{
  "type": "command",
  "request_id": "use-consumable-1",
  "behavior": {
    "type": "use_consumable_item",
    "item_uuid": "owned-consumable-uuid",
    "target_employee_uuid": "employee-uuid"
  }
}
```

`ConsumableItemUsed` result:

```json
{
  "item_uuid": "owned-consumable-uuid",
  "target_employee_uuid": "employee-uuid",
  "replaced_modifier": null,
  "applied_modifier": {
    "source_item_uuid": "owned-consumable-uuid",
    "definition_id": "stabilizing_ampoule",
    "name": "Stabilizing Ampoule",
    "tier": "Common",
    "duration_policy": "NextCombatNode",
    "remaining_combat_nodes": 1,
    "effect": { "TraumaMitigation": { "percent": 20 } }
  },
  "inventory_diff": {
    "added": [],
    "updated": [],
    "removed": ["owned-consumable-uuid"],
    "material_stacks": []
  }
}
```

result는 toast/즉시 피드백용이다. 최종 UI 상태는 뒤따르는 `state_snapshot.inventory.consumables`와 `state_snapshot.roster.employees[*].active_consumable_modifier`를 source of truth로 사용한다.

`active_consumable_modifier.effect`는 Rust enum의 serde 기본 표현을 사용한다. Unity는 variant 이름을 직접 읽고, 모르는 variant는 표시만 하고 기능을 추론하지 않는다. 이 modifier는 전투 timeline의 `BuffApplied`와 다른 계약이며, Unity 표시명으로 `buff`라고 부르더라도 WebSocket/snapshot 필드명은 `active_consumable_modifier`, result 필드명은 `replaced_modifier`와 `applied_modifier`를 유지한다.

```json
{ "DeathPrevent": null }
{ "TraumaMitigation": { "percent": 20 } }
{ "RunHpLossMitigation": { "percent": 20 } }
{ "BattleHpSetup": { "bonus_percent": 15 } }
{ "DeployCostReduction": { "percent": 50 } }
{ "DefenseMitigation": { "percent": 20 } }
{ "InitialSkillCharge": { "percent": 40 } }
{ "OffenseBoost": { "attack_bonus_percent": 20 } }
```

- `DeployCostReduction`은 해당 직원의 현재 배치 코스트 감소만 의미한다. 시작 안정화치 증가나 안정화치 회복량 증가로 해석하지 않는다. 재배치 중인 직원에게는 재배치 기준 비용에 같은 감소율을 적용한 값이 `deployment.unit_deploy_costs[*].effective_deploy_cost`로 노출된다.
- `InitialSkillCharge`는 전투 시작/배치 시 해당 직원의 스킬 게이지(`resonance`)를 퍼센트 기반으로 충전한다. 기본 상한은 Common 20%, Uncommon 40%, Rare 70%, Critical/Forbidden 100%다.

Maintenance 장비 작업:

아래 작업은 Safezone의 간단 정비가 아니라 `Maintenance` Safe Node 내부 작업이다. Unity는 현재 snapshot의 `selected_event.type == "support"`, `support_type == "Maintenance"`, `maintenance_options`와 `allowed_actions`를 확인한 경우에만 노출한다.

```json
{
  "type": "command",
  "request_id": "restore-equipment-1",
  "behavior": {
    "type": "restore_equipment",
    "recipe_id": "recipe_id"
  }
}
```

```json
{
  "type": "command",
  "request_id": "dismantle-equipment-1",
  "behavior": {
    "type": "dismantle_equipment",
    "item_uuid": "uuid"
  }
}
```

```json
{
  "type": "command",
  "request_id": "enhance-equipment-1",
  "behavior": {
    "type": "enhance_equipment",
    "item_uuid": "uuid"
  }
}
```

Maintenance 장비 result:

```text
EquipmentRestored
EquipmentDismantled
EquipmentEnhanced
```

## Skill Fragments

직원은 active skill fragment를 통해 수동 스킬을 사용할 수 있다.

Snapshot에는 스킬 표시를 위해 정적 catalog와 전투 중 readiness가 분리되어 내려온다.

정적 catalog:

```json
{
  "skill_catalog": {
    "version": 1,
    "range_source_of_truth": "defense_tile_range",
    "skills": [
      {
        "skill_id": "fragment_one_sin_penitence",
        "display_name": "Penitence Fragment",
        "kind": "Targeted",
        "focus_time_ms": 220,
        "focus_permissions": {},
        "cast_target": {
          "range_units": 3.0,
          "target_policy": { "EnemySingle": { "rule": "LowestHealthEnemy" } },
          "defense_tile_range": {
            "include_anchor_tile": false,
            "rows": [".XXX.", ".XXX.", "..@..", ".....", "....."]
          }
        },
        "steps": [
          {
            "step_id": "fragment_penitence_judgement",
            "delay_ms": 0,
            "range_units": 3.0,
            "target_policy": { "EnemySingle": { "rule": "LowestHealthEnemy" } },
            "defense_tile_range": {
              "include_anchor_tile": false,
              "rows": [".XXX.", ".XXX.", "..@..", ".....", "....."]
            },
            "delivery": "projectile",
            "tile_area": null,
            "effects_count": 1,
            "presentation": {}
          }
        ]
      }
    ]
  }
}
```

Unity는 스킬 이름, focus 시간, 타겟 정책, range preview를 `skill_catalog.skills[*]`에서 읽는다. DefenseRoute의 범위 source of truth는 `defense_tile_range`이며, `range_units`만으로 범위를 추론하지 않는다.

core 내부에서는 `skill_catalog`를 `SkillCatalogDto` 계열 typed DTO에서 직렬화한다. 따라서 Unity 계약의 field name과 serde casing은 해당 DTO가 source of truth다. 문서 예시는 DTO 출력과 일치해야 하며, `skill_catalog`를 ad-hoc JSON key 조립으로 확장하지 않는다.

`delivery: "tile_area"` step은 `tile_area` 상세 정보를 함께 가진다.

```json
{
  "delivery": "tile_area",
  "tile_area": {
    "anchor": "CastTarget",
    "tile_origin": "Anchor",
    "tracking": "GroundFixed",
    "hit_targets": "Enemies",
    "include_caster": false,
    "tick_policy": "EveryTick",
    "duration_ms": 0,
    "tick_interval_ms": null
  }
}
```

`tile_origin: "Caster"`는 기존처럼 시전자 타일에 `defense_tile_range`의 `@`를 놓는다. `tile_origin: "Anchor"`는 `anchor`가 해석한 위치의 타일에 `@`를 놓는다. Unity는 지정 지점 장판 preview에서 이 값을 보고 cast target 타일 기준 범위를 표시해야 한다.

전투 중 readiness:

```json
{
  "game_state_context": {
    "deployment": {
      "deployed_units": [
        {
          "employee_uuid": "uuid",
          "unit_instance_id": 1,
          "facing": "right",
          "skill_readiness": {
            "skill_id": "fragment_one_sin_penitence",
            "activation_mode": "manual",
            "resonance_current": 100,
            "resonance_max": 100,
            "manual_activation_allowed": true,
            "target_required": true,
            "target_available": false,
            "target_block_reason": "no_valid_target"
          }
        }
      ]
    }
  }
}
```

`manual_activation_allowed`는 플레이어가 스킬 버튼을 눌러 수동 스킬 발동 흐름을 시작할 수 있는지를 의미한다. `target_required`, `target_available`, `target_block_reason`은 대상/타일 선택 상태를 별도로 의미한다.

- `manual_activation_allowed=false`: caster 또는 resource 상태상 발동 흐름 자체를 시작할 수 없다.
- `target_required=true`: 스킬 발동에 타일 또는 대상 선택이 필요하다.
- `target_available=false`: 현재 자동 target 검증 기준으로 즉시 사용할 대상이 없다. 이 경우 Unity는 버튼을 비활성화하지 않고 대상 선택 UI를 열 수 있다.
- `target_block_reason`: target이 없거나 현재 target 조건을 만족하지 못하는 이유다. 대표 값은 `no_valid_target`, `invalid_static_data`다.

`manual_activation_allowed=false`일 때 `can_activate_reason`이 포함될 수 있다. 대표 reason은 `no_skill`, `activation_mode_not_manual`, `silenced`, `action_locked`, `resonance_not_full`, `unit_unavailable`, `not_player_unit`, `invalid_static_data`다.

Unity는 수동 스킬 버튼의 활성화 여부를 `skill_readiness.manual_activation_allowed`로 판단한다. 실제 `ActivateSkill` command는 여전히 서버가 최종 검증한다.

장착:

```json
{
  "type": "command",
  "request_id": "fragment-equip-1",
  "behavior": {
    "type": "equip_skill_fragment",
    "employee_uuid": "uuid",
    "fragment_id": "fragment_one_sin_penitence"
  }
}
```

해제:

```json
{
  "type": "command",
  "request_id": "fragment-unequip-1",
  "behavior": {
    "type": "unequip_skill_fragment",
    "employee_uuid": "uuid",
    "fragment_id": "fragment_one_sin_penitence"
  }
}
```

Maintenance 강화:

```json
{
  "type": "command",
  "request_id": "fragment-upgrade-1",
  "behavior": {
    "type": "upgrade_skill_fragment",
    "target_fragment_id": "fragment_a",
    "material_fragment_id": "fragment_b"
  }
}
```

Maintenance 개화:

```json
{
  "type": "command",
  "request_id": "fragment-awaken-1",
  "behavior": {
    "type": "awaken_skill_fragment",
    "target_fragment_id": "fragment_a",
    "material_fragment_ids": ["fragment_b", "fragment_c"]
  }
}
```

Maintenance 분쇄:

```json
{
  "type": "command",
  "request_id": "fragment-dismantle-1",
  "behavior": {
    "type": "dismantle_skill_fragment",
    "fragment_id": "fragment_a"
  }
}
```

Skill fragment result:

```text
SkillFragmentLoadoutUpdated
SkillFragmentUpgraded
SkillFragmentAwakened
SkillFragmentDismantled
```

## Research Deliveries

스킬 파편 연구가 완료되어도 전투 노드에서 즉시 지급되지 않는다. 완료된 연구는 `inventory.pending_research_deliveries`에 쌓이고, Safe Node 진입 시 자동 수령된다.

기본 Safe Node:

```text
Support
HeadquartersContact
Shop
Reward
```

배송 정보가 포함되는 result:

```text
NodeEntered
SupportState
HeadquartersContactState
ShopState
RewardState
```

snapshot의 `inventory.pending_research_deliveries` shape:

```json
[
  {
    "fragment_id": "fragment_one_sin_penitence",
    "count": 1,
    "total_count": 2
  }
]
```

수령 직후 result의 `research_deliveries` shape:

```json
[
  {
    "fragment_id": "fragment_one_sin_penitence",
    "count": 1
  }
]
```

## Roster

`roster.employees`는 Unity 직원 목록의 기준이다.

주요 필드:

- `uuid`: 직원 UUID.
- `name`: 표시 이름.
- `level`, `experience`: 성장 정보.
- `life_state`: 생존 상태.
- `availability`: 출전 가능 상태.
- `available_for_combat`: 전투 배치 가능 여부.
- `trauma`: 트라우마.
- `health.current_hp`, `health.max_hp`: 런 지속 체력.
- `trust`: 신뢰도 표시/대사 분기용 데이터.
- `combat_profile`: 전투 스탯/기본 공격/배치 타입/현재 effective skill.
- `skill_fragments`: 장착/기본/active fragment 정보.
- `active_consumable_modifier`: Safezone Item Use로 적용된 다음 전투/보스 노드용 섭취 modifier. 없으면 `null`.
- `equipped_items`: 장착 장비.
- `roster_slot`: 직원 명단 표시 순서 슬롯.

`combat_profile.deployment_affinity`는 직원의 기본 배치 허용 타입이며 serde 값은 snake_case다.

```text
ground_only
platform_only
any
```

`combat_profile.effective_deployment_affinity`는 장비/파편/상태 효과가 적용된 현재 전투용 배치 허용 타입이다. 현재 대부분의 직원은 `ground_only`지만 Unity는 이 값을 우선 표시/필터링에 사용한다.

## Client Implementation Rules

Unity는 다음 원칙을 따르면 core 변경에 덜 흔들린다.

- 화면 전환은 `game_state_context.type`을 기준으로 한다.
- 버튼 활성화는 `allowed_actions`를 기준으로 한다.
- 세부 UI 데이터는 `selected_event`, `combat_preview`, `inventory`, `roster`에서 읽는다.
- 명령 성공 후에는 `CommandResult.payload`를 즉시 반영하되, 뒤따라오는 `StateSnapshot`으로 최종 동기화한다.
- 전투 timeline은 `seq` 기준으로 idempotent하게 처리한다.
- `Position`은 core grid 좌표이며 Unity world 좌표가 아니다.
- enum 문자열은 위치마다 casing이 다르다. WebSocket message/behavior `type`은 `snake_case`, 대부분의 core enum 값은 PascalCase, `FacingDirection`은 `snake_case`다.

## Minimal Unity Flow Checklist

1. `/game` WebSocket 연결.
2. `auth` 전송.
3. `state_snapshot` 수신 후 `game_state_context.type`으로 화면 결정.
4. `allowed_actions`에 `StartNewGame`이 있으면 start button 활성화.
5. starter 후보 3명 선택 후 `select_starter_employees`.
6. `ViewingMap`에서 Unity는 Safezone의 `Node Exploration`으로 `map`을 표시.
7. 노드 클릭 시 `select_map_node`.
8. `NodePreview` 또는 snapshot의 `node_confirm` context로 preview UI 표시.
9. `confirm_enter_node`.
10. `InBattle`이면 DefenseRoute 실시간 UI, 아니면 해당 node UI 표시.
11. DefenseRoute에서 `deploy_unit`, `activate_skill`, `withdraw_unit`, `pause_battle`, `resume_battle`, `set_battle_speed` 처리.
12. `battle_delta` timeline을 누적 재생.
13. 노드 결과 후 `complete_node` 또는 각 콘텐츠별 exit/claim 명령 처리.
14. 완료 후 기본 복귀 UI는 Safezone의 `Node Exploration`.

## Known Contract Caveats

- `current_cost` 명칭은 코드상 field name이지만, 플레이어-facing 명칭은 공간 안정화치다.
- Boss 전투의 최종 모드는 아직 TODO다. 현재 Unity는 `node_type == "Boss"`를 별도 연출/진입 화면으로 분기하되, 세부 전투 조작은 snapshot/result를 따른다.
- `Event` category는 live map category에서 제거되어 있다. 추후 랜덤 이벤트로 새로 설계될 수 있다.
