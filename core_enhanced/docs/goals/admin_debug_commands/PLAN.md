# Admin Debug Commands Plan

## Objective

Dev/test 전용 관리자 명령을 구현해 Unity와 Python probe가 비전투 노드, 아이템, 정비, 치료 상태를 빠르게 만들고 WebSocket 계약을 반복 검증할 수 있게 한다.

## Current Plan

1. Server/client message flow를 읽는다. 완료.
   - `PlayerGameClientMessage`
   - session command handling
   - actor handler
   - result/snapshot emission
2. Admin namespace 구조를 선택한다. 완료.
   - top-level `admin_command`.
   - 일반 `PlayerBehaviorRequest`와 분리한다.
3. 접근 제어를 구현한다. 완료.
   - production 금지.
   - explicit env flag 필요.
   - 필요 시 admin token 필요.
4. Admin executor를 구현한다. 완료.
   - 상태 dump.
   - 비전투 node fixture enter.
   - item/material/fragment/resource grant.
   - employee HP/trauma fixture.
5. Admin command 후 snapshot emission을 일반 command와 같은 방식으로 보장한다. 완료.
6. 테스트를 추가한다. 완료.
7. 외부 Unity docs에 admin command 계약과 Python probe를 남긴다. 완료.

## Implemented Shape

- Core command type: `game_core::game::world::AdminCommand`.
- Core executor: `GameCore::execute_admin_command`.
- WebSocket client request: top-level `admin_command`.
- WebSocket server response: `admin_result`, followed by the normal `state_snapshot`.
- Server gate:
  - `RUN_MODE=production` rejects admin commands unconditionally.
  - `ENABLE_ADMIN_COMMANDS` must be truthy.
  - `ADMIN_COMMAND_TOKEN` requires matching request `token` when configured.
- Fixture node entry creates valid admin-only map nodes and normal node sessions so ordinary follow-up commands can continue from the state.

## In Scope

- `admin_dump_state`
- `admin_dump_selected_event`
- `admin_dump_inventory`
- `admin_dump_roster`
- `admin_dump_allowed_actions`
- `admin_enter_support`
- `admin_enter_shop`
- `admin_enter_reward`
- `admin_enter_headquarters_contact`
- `admin_grant_equipment`
- `admin_grant_consumable`
- `admin_grant_artifact`
- `admin_grant_skill_fragment`
- `admin_grant_equipment_material`
- `admin_grant_fragment_dust`
- `admin_set_employee_hp`
- `admin_set_employee_trauma`
- `admin_set_enkephalin`

## Out Of Scope

- 전투 직접 조작.
- prod cheat command.
- save migration.
- reward/balance policy 변경.
- 정상 gameplay command schema 변경.

## Completion Conditions

- Admin command namespace가 gameplay command와 분리되어 있다.
- Admin command는 prod/admin disabled 상태에서 거부된다.
- 1차 fixture commands가 구현되어 비전투 노드/아이템/치료 테스트 상태를 만들 수 있다.
- Admin result 후 snapshot이 방출된다.
- 외부 Unity docs에 admin command 계약과 Python probe가 있다.
- Focused tests와 cargo check가 통과한다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- Admin command를 prod에서 허용해야 하는 요구가 생긴다.
- 전투 결과/보상/실패 판정 우회가 1차 구현에 필요해진다.
- Unity-facing DTO shape 변경이 필요하다.
- 저장 데이터 migration이 필요하다.
- 정상 gameplay command와 admin command를 합쳐야 하는 구조 압력이 생긴다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Verification Commands

```text
cargo test -p game_core admin
cargo test -p game_server admin
cargo check -p game_core
cargo check -p game_server
```

Live probe:

```text
ENABLE_ADMIN_COMMANDS=true ADMIN_COMMAND_TOKEN=dev cargo run -p game_server
python3 /mnt/f/unity projects/ark/docs/unity_admin_debug_command_probe.py
```
