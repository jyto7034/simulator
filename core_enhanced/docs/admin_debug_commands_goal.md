# Admin Debug Commands Goal

이 goal은 Unity 통신 계약, 비전투 노드 UI, 아이템/정비/치료 기능을 빠르게 검증하기 위한 관리자/디버그 명령어를 구현하는 계획서다.

관리자 명령은 게임 규칙이 아니다. 플레이어가 도달할 수 없는 상태를 빠르게 만드는 테스트 fixture builder이며, prod gameplay command와 분리되어야 한다.

## Goal Mode Working Method

작업자는 goal 시작 시 아래 작업 기억장치를 만들고 계속 갱신한다.

```text
docs/goals/admin_debug_commands/PLAN.md
docs/goals/admin_debug_commands/EXPERIMENTS.md
docs/goals/admin_debug_commands/EXPERIMENT_NOTES.md
```

각 파일의 역할:

- `PLAN.md`: 구현 순서, 범위, 완료 조건, 중단 조건, 검증 명령을 기록한다.
- `EXPERIMENTS.md`: 시도/실패/성공 결과, 테스트 실패 원인, 수정 내용, 재검증 결과를 기록한다.
- `EXPERIMENT_NOTES.md`: 작업 중 판단, source-of-truth 충돌, 정책 질문, 후속 후보를 기록한다.

이 파일들은 goal 실행 중의 작업 기억장치다. 완료 후 유지해야 할 계약은 외부 Unity docs와 core 정책 문서에 옮긴다.

## Objective

`game_server` WebSocket에서 dev/test 전용 관리자 명령을 제공해 Unity와 Python probe가 원하는 게임 상태를 빠르게 만들고, 비전투 노드/아이템/정비/치료 계약을 반복 검증할 수 있게 한다.

## Design Boundary

관리자 명령은 게임 설계와 상반될 수 있다. 예를 들어 현재 노드를 강제로 바꾸거나 아이템을 즉시 지급하는 행동은 정상 플레이 규칙이 아니다.

따라서 이 goal의 핵심 원칙은 다음이다.

- 관리자 명령은 gameplay command가 아니라 test fixture command다.
- 관리자 명령은 prod 빌드/운영 환경에서 접근 불가해야 한다.
- 관리자 명령은 일반 `PlayerBehavior`와 섞지 않는다.
- 관리자 명령이 만든 상태도 이후 일반 command가 처리할 수 있는 유효한 runtime state여야 한다.
- 관리자 명령은 저장 데이터/밸런스/보상 설계를 우회하기 위한 플레이어 기능으로 문서화하지 않는다.
- 설계 침범이 큰 명령은 1차 구현 범위에서 제외하거나 사용자 확인 후 진행한다.

## Source Of Truth

먼저 읽을 runtime code:

- `../game_server/src/game/player_game_actor/messages.rs`: WebSocket client message enum.
- `../game_server/src/game/player_game_actor/session.rs`: auth, command handling, response ordering.
- `../game_server/src/game/player_game_actor/state.rs`: command result payload mapping.
- `../game_server/src/game/player_game_actor/handlers.rs`: actor message handling.
- `src/game/behavior.rs`: gameplay `PlayerBehavior`, `BehaviorResult`, DTO.
- `src/game/world.rs`: command dispatch.
- `src/game/world/map_content.rs`: node enter behavior.
- `src/game/world/support.rs`: Support/Medical/Rest/Maintenance behavior.
- `src/game/world/maintenance.rs`: Maintenance operations.
- `src/game/world/shop.rs`: Shop behavior.
- `src/game/world/reward.rs`: Reward behavior.
- `src/game/world/headquarters.rs`: HeadquartersContact behavior.
- `src/game/world/snapshot.rs`: Unity-facing snapshot shape.
- `src/game/resources/selection.rs`: active node session state.
- `src/game/resources/inventory.rs`: owned item/material DTO and inventory mutation.
- `src/game/skill_fragment.rs`: skill fragment inventory/progress/dust.
- `src/game/data/mod.rs`, `src/game/data/*_data.rs`: live data lookup and item definitions.
- `../game_resources/data/**/*.ron`: live data IDs.

Canonical Unity docs location:

```text
F:\unity projects\ark\docs
/mnt/f/unity projects/ark/docs
```

특히 관리자 명령 계약은 외부 Unity docs에 새 문서로 남긴다.

## In Scope

1차 구현 범위:

- Dev/test 전용 admin WebSocket message namespace 추가.
- Admin 접근 게이트 추가.
- Admin command result와 snapshot response ordering 확정.
- 상태 조회/덤프 명령.
- 비전투 노드 진입 fixture 명령.
- 아이템/재료/파편/자원 지급 fixture 명령.
- 직원 치료 테스트 상태 fixture 명령.
- Python probe가 바로 사용할 수 있는 관리자 명령 계약 문서와 예시 스크립트 작성.

권장 1차 admin commands:

```text
admin_dump_state
admin_dump_selected_event
admin_dump_inventory
admin_dump_roster
admin_dump_allowed_actions
admin_enter_support
admin_enter_shop
admin_enter_reward
admin_enter_headquarters_contact
admin_grant_equipment
admin_grant_consumable
admin_grant_artifact
admin_grant_skill_fragment
admin_grant_equipment_material
admin_grant_fragment_dust
admin_set_employee_hp
admin_set_employee_trauma
admin_set_enkephalin
```

2차 후보:

```text
admin_reveal_all_nodes
admin_force_node_kind
admin_complete_node
admin_skip_node
admin_add_employee
admin_set_skill_fragment_progress
admin_equip_item
admin_equip_skill_fragment
admin_make_medical_target
admin_seed_run
```

전투 조작 후보는 독립 goal 또는 사용자 확인 후 진행한다.

```text
admin_enter_combat
admin_advance_battle
admin_finish_battle
admin_spawn_enemy
admin_set_enemy_hp
admin_grant_deploy_cost
```

## Out Of Scope

- 플레이어가 사용할 수 있는 치트/콘솔 UI.
- 운영 서버에서 접근 가능한 관리자 기능.
- 저장 데이터 migration.
- 정상 gameplay reward/balance 정책 변경.
- 전투 타임라인/시뮬레이션 직접 조작.
- 과거 정책을 살리기 위한 compatibility layer.
- 랜덤 맵 생성 규칙 자체 변경.

## Admin Access Policy

다음 중 하나 이상의 접근 게이트를 구현한다.

- `RUN_MODE != production`에서만 admin message를 허용한다.
- `ENABLE_ADMIN_COMMANDS=true` 같은 명시 env flag가 있어야 허용한다.
- admin token 또는 dev secret을 요구한다.

권장 정책:

```text
RUN_MODE != production
AND ENABLE_ADMIN_COMMANDS=true
AND admin_token matches configured dev token when token is configured
```

운영 환경에서 admin command가 들어오면 서버는 상태를 변경하지 않고 `error`를 반환해야 한다.

## Contract Direction

일반 gameplay command와 섞지 않는다.

권장 top-level client message:

```json
{
  "type": "admin_command",
  "request_id": "admin-001",
  "admin": {
    "type": "admin_enter_maintenance"
  }
}
```

성공 시 일반 command와 같은 순서를 따른다.

```text
admin_result
state_snapshot
```

실패 시:

```text
error
```

관리자 명령의 `result_type`은 gameplay `BehaviorResult`와 혼동되지 않도록 `Admin...` prefix를 붙인다.

예:

```json
{
  "type": "admin_result",
  "request_id": "admin-001",
  "ok": true,
  "result_type": "AdminEnteredMaintenance",
  "payload": {
    "node_id": "..."
  }
}
```

## Implementation Plan

1. 현재 server message/session/actor 구조를 읽는다.
   - `PlayerGameClientMessage`에 새 top-level admin branch를 넣을지, 별도 route를 만들지 판단한다.
   - 장기적으로 더 안전한 구조를 선택하고 이유를 `EXPERIMENT_NOTES.md`에 기록한다.
2. Admin request/result 타입을 정의한다.
   - 일반 `PlayerBehaviorRequest`와 분리한다.
   - admin DTO는 `game_server`에만 둘지, `game_core`에 command/result 타입을 둘지 검토한다.
3. Admin 접근 게이트를 구현한다.
   - prod 금지 테스트를 추가한다.
   - dev flag 미설정 시 거부 테스트를 추가한다.
4. Admin command executor를 구현한다.
   - 가능한 한 기존 `GameCore` public/internal helper를 사용하되, 상태 무결성을 깨는 직접 필드 조작은 최소화한다.
   - 필요한 fixture helper는 의도가 드러나는 이름으로 별도 모듈에 둔다.
5. 1차 fixture command를 구현한다.
   - 상태 dump.
   - Support/Shop/Reward/HeadquartersContact 진입.
   - item/material/fragment/resource 지급.
   - employee HP/trauma 설정.
6. Admin command 후 snapshot 방출을 보장한다.
   - 일반 command와 동일하게 Unity가 snapshot을 최종 truth로 쓸 수 있어야 한다.
7. Python probe를 작성한다.
   - 외부 Unity docs에 저장한다.
   - 비전투 노드 계약 검증용 fixture setup에 admin commands를 사용한다.
8. 문서를 갱신한다.
   - `/mnt/f/unity projects/ark/docs/unity_admin_debug_command_contract.md`
   - 필요 시 `/mnt/f/unity projects/ark/docs/unity_noncombat_node_ws_contract.md`
9. 테스트를 실행한다.
   - focused tests.
   - `cargo check`.
   - 가능한 경우 live WebSocket probe.

## Test Requirements

테스트는 아래를 고정한다.

- production/admin disabled 상태에서 admin command가 거부된다.
- admin command는 일반 `PlayerBehaviorRequest`로 deserialize되지 않는다.
- admin command 성공 시 `admin_result` 후 `state_snapshot`이 전송된다.
- `admin_enter_support` 후 `selected_event.type == "support"`가 된다.
- `admin_enter_maintenance` 후 `selected_event.type == "maintenance"`가 되고 `maintenance_options`가 snapshot에 존재한다.
- `admin_enter_shop` 후 `selected_event.type == "shop"`이 된다.
- `admin_enter_reward` 후 `selected_event.type == "reward"`가 된다.
- `admin_enter_headquarters_contact` 후 `selected_event.type == "headquarters_contact"`가 된다.
- grant 명령은 live data ID가 없으면 실패하고 상태를 바꾸지 않는다.
- employee HP/trauma fixture 명령은 존재하는 직원만 수정한다.
- admin command 이후 일반 gameplay command가 같은 상태를 정상 처리할 수 있다.

권장 검증 명령:

```text
cargo test -p game_core admin
cargo test -p game_server admin
cargo check -p game_core
cargo check -p game_server
```

가능하면 live server probe:

```text
cargo run -p game_server
python3 /mnt/f/unity projects/ark/docs/unity_admin_debug_command_probe.py
```

## Completion Conditions

- `admin_command` namespace가 일반 gameplay command와 분리되어 있다.
- prod/admin disabled 상태에서 admin command가 상태를 변경하지 않는다.
- 1차 admin commands가 구현되어 비전투 노드/아이템/치료 테스트 fixture를 빠르게 만들 수 있다.
- admin command 결과와 snapshot ordering이 문서화되어 있다.
- 외부 Unity docs에 관리자 명령 계약 문서와 Python probe가 있다.
- 기존 WebSocket smoke probe와 non-combat node probe가 admin fixture를 활용할 수 있다.
- 테스트가 admin command의 접근 제어, 상태 생성, snapshot 계약을 고정한다.
- 레거시/임시 fallback path 없이 장기적으로 유지 가능한 구조다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

아래 사항이 발견되면 임의로 결정하지 말고 goal을 종료한 뒤 사용자에게 질문한다.

- admin command를 prod에서 일부 허용해야 한다는 요구가 생긴 경우.
- admin command가 저장 데이터 migration 또는 영구 save format 변경을 요구하는 경우.
- 전투 결과, 보상, 실패/성공 판정을 우회하는 명령이 1차 범위에 필요해진 경우.
- live RON schema를 바꾸지 않으면 fixture를 만들 수 없는 경우.
- Unity-facing DTO shape를 변경해야 하는 경우.
- 정상 gameplay command와 admin command를 같은 enum/schema에 합쳐야 한다는 압력이 생긴 경우.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Goal Command

```text
/goal 관리자/디버그 명령어를 dev/test 전용 WebSocket 계약으로 구현해 Unity와 Python probe가 비전투 노드, 아이템, 정비, 치료 상태를 빠르게 만들고 검증할 수 있게 한다.

목표 문서:
- docs/admin_debug_commands_goal.md

Goal working directory:
- docs/goals/admin_debug_commands/

시작 시 반드시 아래 작업 기억장치를 만들고 계속 갱신하라.
- docs/goals/admin_debug_commands/PLAN.md
- docs/goals/admin_debug_commands/EXPERIMENTS.md
- docs/goals/admin_debug_commands/EXPERIMENT_NOTES.md

코드 수정은 장기적인 방향으로 하라. 임시방편, 최소한의 수정, compatibility layer, fallback path, dual schema, 특정 테스트만 통과시키는 패치를 피하라.
문서를 무조건 신뢰하지 말고 실제 runtime code, live RON/data, Unity-facing snapshot/command 계약, 최신 정책 문서 순서로 source of truth를 확인하라.
관리자 명령은 게임 규칙이 아니라 dev/test fixture builder다. 일반 gameplay command와 섞지 말고, prod/admin disabled 상태에서는 반드시 거부되게 하라.
Unity-facing 계약 문서는 외부 canonical 위치인 `/mnt/f/unity projects/ark/docs`에 갱신하라.
git restore, git reset 등 git file modifier 명령은 수행하지 않는다.
git의 과거 내역 코드를 현재 파일에 cp하지 않는다.
파일을 과거 상태로 돌려야 한다고 판단하면 이유를 설명하고 사용자 허락을 먼저 구한다.
사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.
```
