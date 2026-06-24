# Code And Documentation Sync Guidelines

이 문서는 core/server 코드 수정 시 반드시 따를 작업 지침서다.

핵심 원칙은 단순하다.

- core 코드와 문서는 항상 같은 내용을 말해야 한다.
- 코드가 바뀌면 관련 정책 문서, Unity-facing 계약 문서, probe/test도 함께 바뀌어야 한다.
- 문서가 바뀌면 실제 runtime code, live RON/data, server DTO가 그 문서를 만족하는지 확인해야 한다.

## Source Of Truth 순서

작업자는 아래 순서로 사실을 확인한다.

1. 실제 runtime code
2. live RON/data
3. Unity-facing snapshot/command 계약
4. 최신 정책 문서

문서를 무조건 신뢰하지 않는다. 문서와 코드가 충돌하면 먼저 코드를 읽고, 정책 판단이 필요한 부분은 사용자와 의논한다.

## Canonical 문서 위치

게임 규칙과 core 정책:

- `docs/game_rulebook.md`
- 도메인별 계약 문서

Unity-facing 계약/구현 문서:

```text
F:\unity projects\ark\docs
/mnt/f/unity projects/ark/docs
```

특히 아래 문서는 외부 Unity 프로젝트의 파일이 canonical이다.

- `F:\unity projects\ark\docs\unity_core_contract.md`
- `F:\unity projects\ark\docs\unity_client_implementation_goal.md`
- `F:\unity projects\ark\docs\core_unity_battle_transport_contract.md`
- `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- `/mnt/f/unity projects/ark/docs/unity_client_implementation_goal.md`
- `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`

`core_unity_battle_transport_contract.md`가 현재 live battle transport의 canonical 통합 문서다. setup/update 분리 문서는 과거 논의와 migration context 확인용으로 남을 수 있지만, 새 battle transport 계약을 바꿀 때는 통합 문서를 우선 갱신한다.

이 저장소 안에는 `docs/unity_core_contract.md`, `docs/unity_client_implementation_goal.md` 같은 local copy를 보관하지 않는다. 같은 이름의 문서가 다시 생기면 stale copy로 간주하고, Unity-facing 계약을 바꿀 때는 외부 canonical 문서를 확인하고 갱신한다.

## 코드와 문서 동기화 규칙

코드를 수정할 때는 다음을 함께 확인한다.

- gameplay rule 변경이면 `docs/game_rulebook.md` 또는 관련 정책 문서를 갱신한다.
- Unity-facing DTO, snapshot, command, admin command가 바뀌면 외부 Unity 계약 문서를 갱신한다.
- live RON schema나 authored data 의미가 바뀌면 RON 문서/계약과 loading validation을 갱신한다.
- WebSocket/admin 계약이 바뀌면 문서뿐 아니라 Python probe 또는 server message test도 갱신한다.
- 새 field를 추가하면 Unity가 추론하지 않아도 되도록 field 의미, casing, nullability, source of truth를 문서에 적는다.
- field를 제거하거나 의미를 바꾸면 관련 legacy 문구를 문서에서 제거한다.

문서 업데이트는 코드 변경의 부속 작업이 아니라 완료 조건의 일부다.

## 변경 유형별 갱신 매트릭스

작업자는 변경 유형에 따라 아래 문서와 테스트를 함께 확인한다.

| 변경 유형 | 반드시 확인/갱신할 문서 | 반드시 확인할 테스트/검증 |
| --- | --- | --- |
| `PlayerBehavior` request 추가/변경 | 외부 `unity_core_contract.md`, 관련 goal/정책 문서 | server message deserialize test, focused core test, 필요 시 WebSocket probe |
| `BehaviorResult` 추가/변경 | 외부 `unity_core_contract.md`, 관련 snapshot/command 계약 문서 | `game_server` result mapping test, focused core test |
| `selected_event` snapshot 변경 | 외부 `unity_core_contract.md`, 필요 시 해당 도메인의 외부 Unity 계약 문서 | snapshot shape test, Python probe |
| admin command 추가/변경 | 외부 `unity_core_contract.md`, 필요 시 admin 전용 외부 Unity 계약 문서 | `cargo test -p game_server admin -- --nocapture`, 사용 가능한 admin probe |
| Unity-facing DTO field 추가/삭제/의미 변경 | 외부 `unity_core_contract.md` | DTO shape test, server mapping test, 필요 시 probe |
| live RON schema 변경 | `docs/game_rulebook.md` 또는 도메인 계약 문서 | `cargo test -p game_core --test ron_loading -- --nocapture` |
| live RON content 의미 변경 | `docs/game_rulebook.md`, 도메인 문서 | live RON loading, focused gameplay/data validation test |
| gameplay rule 변경 | `docs/game_rulebook.md` 또는 도메인 문서 | focused gameplay flow test |
| battle setup snapshot 변경 | `docs/game_rulebook.md`, 외부 `core_unity_battle_transport_contract.md`, 외부 `unity_core_contract.md` | battle setup DTO shape test, server message order test, 필요 시 WebSocket probe |
| battle update/checkpoint/event 변경 | `docs/game_rulebook.md`, 외부 `core_unity_battle_transport_contract.md`, 외부 `unity_core_contract.md` | battle focused test, server delta/snapshot mapping test, 필요 시 WebSocket probe |
| Maintenance/Shop/Reward/Support 같은 non-combat node 계약 변경 | 외부 `unity_core_contract.md`, 필요 시 해당 도메인의 외부 Unity 계약 문서 | node focused test, Python non-combat/admin probe |
| Unity 구현 목표/UX 정책 변경 | 외부 `unity_client_implementation_goal.md` | Unity 쪽 goal/test/probe, core 계약과 충돌 여부 확인 |

매트릭스에 없는 변경이라도 Unity가 표시하거나 command로 호출하는 표면이면 Unity-facing 계약 변경으로 취급한다.

## 완료 전 셀프 체크리스트

작업 완료 전 아래 질문에 답한다.

- 코드 변경이 관련 문서에 반영됐는가?
- 문서에 적은 JSON shape, enum casing, nullability가 실제 DTO 출력과 일치하는가?
- 외부 Unity canonical 문서를 갱신해야 하는 변경인가?
- 외부 Unity canonical 문서를 갱신했다면 probe 또는 server mapping test도 갱신했는가?
- live RON/data 의미가 바뀌었다면 loading/data validation test를 갱신했는가?
- stale copy 문구가 남아 있지 않은가?
- 구계약과 신계약을 동시에 살리는 dual schema가 생기지 않았는가?
- Unity가 core 대신 RON이나 내부 규칙을 직접 추론하게 만들지 않았는가?
- 테스트가 내부 구현 모양이 아니라 사용자-visible behavior와 계약을 고정하는가?
- 내가 만든 임시 파일, 로그, probe 부산물만 정리했는가?

하나라도 답이 불명확하면 완료하지 말고 코드를 다시 읽거나 사용자에게 질문한다.

## Stale 문서 방지 규칙

이 저장소에는 `docs/unity_core_contract.md`, `docs/unity_client_implementation_goal.md`를 보관하지 않는다.

- Unity-facing 계약의 canonical은 외부 Unity 프로젝트 문서다.
- 로컬 stale copy를 갱신하는 방식으로 작업을 완료하지 않는다.
- 외부 문서를 갱신했는데 로컬 stale copy가 다음 작업자를 오도할 가능성이 있으면 로컬 문서는 제거한다.
- 문서 검색 시 같은 제목의 문서가 여러 위치에 있으면 외부 canonical 문서를 우선한다.
- 완료 보고에는 어떤 문서를 갱신했는지 경로를 명시한다.

## 구현 원칙

코드 수정은 장기적인 방향으로 한다.

피해야 할 것:

- 임시방편
- 최소한의 수정만으로 특정 테스트를 통과시키는 패치
- compatibility layer
- fallback path
- dual schema
- 레거시 동작을 보존하기 위한 adapter
- ignored test로 레거시를 보존하는 방식

구체적인 금지 예시:

- 코드만 바꾸고 계약 문서를 갱신하지 않는다.
- 문서만 바꾸고 실제 DTO/probe/test를 확인하지 않는다.
- Unity가 live RON을 직접 읽게 해 admin/item/skill id를 채운다.
- Unity가 core 규칙을 재계산하거나 추론하게 한다.
- DTO를 `json!` ad-hoc 조립으로 계속 확장하면서 typed DTO/test를 만들지 않는다.
- 기존 구계약과 새 계약을 동시에 받는 dual schema를 기본 선택으로 둔다.
- `ignored` test로 구정책을 보존한다.
- 실패한 테스트의 기대값만 새 값으로 바꾸고 사용자-visible behavior를 확인하지 않는다.
- 외부 canonical Unity 문서 대신 로컬 stale copy만 갱신한다.

정말 compatibility layer가 필요하다면 이유, 제거 조건, 테스트 범위를 문서에 기록하고 사용자 확인을 받는다.

처음에는 어떤 구조가 장기적인 방향인지 판단하기 어려울 수 있다. 작은 trial and error를 통해 확인하고, 실패한 접근과 이유를 `EXPERIMENTS.md` 또는 작업 기록에 남겨 같은 실수를 반복하지 않는다.

## 레거시 처리

레거시는 과감하게 제거한다.

- 현재 정책과 충돌하는 테스트는 기대값만 바꾸지 않는다.
- 그 테스트가 보호하던 사용자-visible behavior를 확인한다.
- 최신 정책에 맞는 focused test로 교체하거나, 더 이상 의미가 없으면 삭제한다.
- 공식 live flow에 없는 데이터/문서/fixture가 source of truth처럼 보이면 정리한다.

git 과거 내역에서 코드를 복사해 현재 파일에 붙이지 않는다. 반드시 필요하면 이유를 설명하고 사용자 허락을 먼저 받는다.

## 테스트 원칙

테스트는 내부 구현 모양보다 아래 계약을 고정한다.

- 사용자-visible behavior
- Unity-facing DTO shape
- server command/result mapping
- live RON loading
- data validation
- 실제 gameplay flow
- admin/debug command 계약
- Python smoke probe로 확인 가능한 WebSocket 계약

작은 변경 단위마다 빠른 focused test 또는 `cargo check`를 돌린다. 마지막에는 변경 범위에 맞는 넓은 테스트를 돌린다.

권장 검증 순서:

1. focused unit/integration test
2. 관련 `cargo check`
3. 관련 package test
4. live RON loading test
5. Unity-facing 변경이면 WebSocket/Python probe
6. 필요 시 broader `cargo test`

대표 검증 명령 예시:

```text
cargo check -p game_core
cargo check -p game_server
cargo test -p game_core admin -- --nocapture
cargo test -p game_server admin -- --nocapture
cargo test -p game_core maintenance -- --nocapture
cargo test -p game_core support -- --nocapture
cargo test -p game_core --test ron_loading -- --nocapture
cargo test -p game_core
cargo test -p game_server
```

WebSocket/admin probe 예시:

```text
APP__SERVER__BIND_ADDRESS=127.0.0.1 APP__SERVER__PORT=18083 ENABLE_ADMIN_COMMANDS=true ADMIN_COMMAND_TOKEN=dev cargo run -p game_server
ADMIN_COMMAND_TOKEN=dev WS_PORT=18083 python3 "<available-admin-probe-path>"
```

probe 실행 후 생성된 로그/임시 파일이 있다면 내가 만든 부산물만 정리한다.

테스트 실패는 실패 원인, 수정 내용, 재검증 결과를 기록한다.

## 사용자와 의논해야 하는 경우

아래 항목은 임의로 확정하지 않는다. 작업을 멈추고 사용자에게 질문한다.

- Unity-facing DTO shape 변경
- live RON schema 변경
- 저장 데이터 migration
- UX 의미 변화
- 밸런스 기준
- 기존 콘텐츠 삭제/대체
- 실패/보상/소비 시점 변화
- 게임 룰 변경
- 노드 흐름 변경
- 전투 성공/실패 판정
- 장비/스킬/아이템 성장 정책
- core와 Unity 중 어느 쪽이 source of truth인지 불명확한 경우

완료 조건에는 항상 다음 원칙을 포함한다.

- 사용자와 의논하여 정해야 할 정책이 있을 경우 작업을 종료하고 질문 목록을 보고한다.

## Goal 작업 방식

큰 작업은 goal로 분리한다.

goal 시작 시 아래 파일을 만든 뒤 계속 갱신한다.

```text
docs/goals/<goal_name>/PLAN.md
docs/goals/<goal_name>/EXPERIMENTS.md
docs/goals/<goal_name>/EXPERIMENT_NOTES.md
```

각 파일의 역할:

- `PLAN.md`: 계획, 범위, 완료 조건, 중단 조건, 검증 명령
- `EXPERIMENTS.md`: 시도/실패/성공 결과, 테스트 실패 원인, 수정 내용, 재검증 결과
- `EXPERIMENT_NOTES.md`: 작업 중 판단, source-of-truth 충돌, 정책 질문, 후속 후보

goal 범위를 벗어난 개선안은 `EXPERIMENT_NOTES.md`에 후속 후보로 기록한다. 현재 goal 완료에 필수이거나 blast radius를 줄이는 경우에만 적용한다.

## Git/File Safety

작업자는 아래 규칙을 지킨다.

- `git restore`, `git reset` 등 git file modifier 명령은 수행하지 않는다.
- git의 과거 내역 코드를 현재 파일에 `cp`하지 않는다.
- 파일을 과거 상태로 돌려야 한다고 판단하면 이유를 설명하고 사용자 허락을 먼저 구한다.
- worktree가 dirty여도 사용자 변경으로 간주하고 되돌리지 않는다.
- unrelated change는 건드리지 않는다.
- 내가 만든 임시 로그/테스트 부산물만 정리한다.

## 완료 보고

완료 시 아래 내용을 보고한다.

- 변경 요약
- 갱신한 문서
- 새로 고정한 계약
- 제거한 레거시
- 갱신한 테스트/probe
- 실행한 검증 명령
- 남은 위험
- 사용자와 의논해야 할 정책 질문이 있었는지

문서와 코드가 같은 말을 하지 않으면 작업은 완료된 것이 아니다.
