# 외부 Probe Registry

이 문서는 실행 가능한 WebSocket/admin probe의 durable routing index다. 정확한 gameplay와 wire
의미는 연결된 외부 Unity 계약이 소유하며 이 파일은 payload를 복사하지 않는다.

## 문서 책임

| 구분 | 내용 |
| --- | --- |
| 문서 등급 | 실행 probe 색인 |
| 소유 범위 | 현재 probe 위치, 공통 환경 변수, server gate, 대표 실행 명령, evidence 선택 규칙 |
| 소유하지 않는 범위 | gameplay 의미, wire schema, payload 예시, runtime 정책, 현재 변경을 입증하는 과거 검증 결과 |
| 함께 확인할 문서 | [engineering_workflow.md](engineering_workflow.md), 외부 Unity probe 계약, 영향받는 gameplay/runtime/data canonical |

## 위치와 공통 환경

대부분의 범용 probe는 외부 Unity 프로젝트에 있고, Event employee-check 전용 실제 transport
probe는 game_server 계약 가까이에 있다.

```text
F:\unity projects\ark\docs\probe
/mnt/f/unity projects/ark/docs/probe
/mnt/f/work/simulator/game_server/docs/probe
```

외부 Unity의 세 script는 모두 `WS_HOST`(기본값 `127.0.0.1`), `WS_PORT`(기본값 `18083`),
`WS_PATH`(기본값 `/game`), `WS_TIMEOUT` 환경 변수를 받는다. Server는
`/mnt/f/work/simulator/game_server`에서 실행하고 probe는 절대 경로를 사용하면 어느 작업
디렉터리에서든 실행할 수 있다.

## 등록된 Probe

| Probe | 목적 | Server gate | 인접 계약 |
| --- | --- | --- | --- |
| `/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py` | 인증된 run-start, map, combat setup/update/resync와 선택적 combat-result broad smoke | 일반 non-production local server. 더 긴 result 구간이 필요할 때만 `WS_CHECK_COMBAT_RESULT=1` | `/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_test_contract.md` |
| `/mnt/f/unity projects/ark/docs/probe/unity_savepoint_checkpoint_probe.py` | 저작된 `standard_branching_a/save_archive` SavePoint fixture로 저장과 checkpoint load를 수행하며 템플릿 identity, capability, load counter 확인 | `ENABLE_ADMIN_COMMANDS=true`, 일치하는 `ADMIN_COMMAND_TOKEN`(probe 기본값 `dev`), production mode 금지 | `/mnt/f/unity projects/ark/docs/probe/unity_noncombat_node_ws_contract.md` |
| `/mnt/f/unity projects/ark/docs/probe/unity_admin_debug_command_probe.py` | 결정론적 non-combat/admin fixture를 만들고 Maintenance, SavePoint, Headquarters, Shop, Reward와 admin catalog/result envelope 확인 | `ENABLE_ADMIN_COMMANDS=true`, 일치하는 `ADMIN_COMMAND_TOKEN`(probe 기본값 `dev`), production mode 금지 | `/mnt/f/unity projects/ark/docs/probe/unity_admin_debug_command_contract.md`, `unity_noncombat_node_ws_contract.md` |
| `/mnt/f/work/simulator/game_server/docs/probe/event_employee_check_ws_probe.py` | live checked Event의 choice별 후보/확률/key, reconnect, committed provenance, exact duplicate와 no-double-grant를 실제 `/game` WebSocket으로 확인 | `ENABLE_ADMIN_COMMANDS=true`, 일치하는 `ADMIN_COMMAND_TOKEN`(기본값 `dev`), production mode 금지. CLI `--host/--port/--admin-token/--player-id/--timeout` 사용 | [game_rulebook.md](game_rulebook.md), [core_runtime_contract.md](core_runtime_contract.md), `/mnt/f/unity projects/ark/docs/unity_core_contract.md` |

## 실행 명령

일반 server와 broad smoke:

```text
APP__SERVER__BIND_ADDRESS=127.0.0.1 APP__SERVER__PORT=18083 cargo run
WS_PORT=18083 python3 "/mnt/f/unity projects/ark/docs/probe/unity_ws_smoke_probe.py"
```

Admin-enabled server와 SavePoint checkpoint probe:

```text
APP__SERVER__BIND_ADDRESS=127.0.0.1 APP__SERVER__PORT=18083 ENABLE_ADMIN_COMMANDS=true ADMIN_COMMAND_TOKEN=dev cargo run
ADMIN_COMMAND_TOKEN=dev WS_PORT=18083 python3 "/mnt/f/unity projects/ark/docs/probe/unity_savepoint_checkpoint_probe.py"
```

Admin-enabled server와 non-combat/admin probe:

```text
APP__SERVER__BIND_ADDRESS=127.0.0.1 APP__SERVER__PORT=18083 ENABLE_ADMIN_COMMANDS=true ADMIN_COMMAND_TOKEN=dev cargo run
ADMIN_COMMAND_TOKEN=dev WS_PORT=18083 python3 "/mnt/f/unity projects/ark/docs/probe/unity_admin_debug_command_probe.py"
```

Admin-enabled server와 Event employee-check probe:

```text
APP__SERVER__BIND_ADDRESS=127.0.0.1 APP__SERVER__PORT=18083 ENABLE_ADMIN_COMMANDS=true ADMIN_COMMAND_TOKEN=dev cargo run
python3 /mnt/f/work/simulator/game_server/docs/probe/event_employee_check_ws_probe.py --port 18083 --admin-token dev --player-id 00000000-0000-0000-0000-00000000a120
```

## 선택과 Evidence 규칙

- Focused Rust/server/Unity test를 먼저 실행한다. Message ordering, live server gate 또는
  multi-command WebSocket flow가 변경에 중요할 때 probe를 실행한다.
- 정확한 server 명령, probe 경로, 환경 변수 override, 결과와 날짜를 active goal evidence에
  기록한다. 계약에 적힌 과거 “verified” 날짜는 현재 변경의 증거가 아니다.
- Probe script는 consumer이지 schema authority가 아니다. Wire shape가 바뀌면 외부 canonical
  계약과 typed test를 함께 갱신한다.
- Probe를 임의로 중복 복사하지 않는다. 각 probe는 위 등록 경로 한 곳만 소유하며 이동하거나
  대체되면 이 registry를 갱신한다.
