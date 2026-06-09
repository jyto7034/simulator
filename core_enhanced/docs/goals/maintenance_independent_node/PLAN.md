# Maintenance Independent Node Plan

## Objective

Maintenance를 `SupportNodeType::Maintenance`에서 분리해 독립적인 map node category/payload와 Unity-facing selected event로 구현한다.

## Current Plan

1. [done] 현재 Maintenance runtime surface를 읽는다.
   - map category/payload
   - support session
   - maintenance operation commands
   - allowed actions
   - selected event snapshot
   - admin fixture/probe
2. [done] 독립 node model을 설계한다.
   - `MapNodeCategory::Maintenance`
   - `MapNodePayload::Maintenance`
   - `NodeSessionKind::Maintenance`
   - `MaintenanceSessionState`
3. [done] Maintenance 진입/완료/snapshot/allowed action을 독립 경로로 옮긴다.
4. [done] Support에서 Maintenance 선택지를 제거한다.
5. [done] Admin fixture와 Unity/Python probe 계약을 새 Maintenance 노드로 갱신한다.
6. [done] live RON/data를 새 schema로 갱신한다.
7. [done] 테스트를 추가/갱신한다.
8. [done] focused tests, cargo check, live probe를 실행한다.

## In Scope

- 독립 Maintenance node category/payload.
- 독립 Maintenance selected event.
- 기존 Maintenance operations 유지.
- Support choices에서 Maintenance 제거.
- Admin fixture/probe 갱신.
- Unity external docs 갱신.
- live RON/data 갱신.

## Out Of Scope

- Maintenance 경제 밸런스 변경.
- Unity 화면 구현.
- 저장 데이터 migration.
- Medical/Rest 정책 변경.
- 전투/보상/상점 정책 변경.

## Completion Conditions

- Maintenance가 Support 하위 타입이 아닌 독립 노드다.
- `selected_event.type == "maintenance"`가 runtime snapshot과 Unity docs에 반영된다.
- Maintenance operation commands는 독립 Maintenance 노드에서만 허용된다.
- Support choices/runtime에서 Maintenance가 제거된다.
- live RON/data와 admin probe가 새 Maintenance 노드를 사용한다.
- Focused tests와 cargo check가 통과한다.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

- 저장 데이터 migration이 필요하다.
- Unity가 기존 `support_type == "Maintenance"` 계약을 유지해야 한다.
- Maintenance를 Support 선택지로도 유지해야 한다.
- compatibility layer나 dual schema가 필요하다.
- 사용자와 의논하여 정해야 할 정책이 있다.

## Verification Commands

```text
cargo test -p game_core maintenance -- --nocapture
cargo test -p game_core support -- --nocapture
cargo test -p game_core --test ron_loading -- --nocapture
cargo test -p game_server maintenance -- --nocapture
cargo check -p game_core
cargo check -p game_server
```

Live probe:

```text
ENABLE_ADMIN_COMMANDS=true ADMIN_COMMAND_TOKEN=dev cargo run -p game_server
ADMIN_COMMAND_TOKEN=dev WS_PORT=8081 python3 "/mnt/f/unity projects/ark/docs/unity_admin_debug_command_probe.py"
```
