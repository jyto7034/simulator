# Engineering Workflow

이 문서는 `core_enhanced`와 `game_server`의 코드 변경, 리팩토링, 문서 동기화, goal 운용에 공통으로 적용하는 작업 절차다. 구현 기록이나 특정 시점의 추천 작업 목록이 아니라, 변경을 안전하게 판단하고 완료하기 위한 현재형 기준만 유지한다.

게임 규칙과 runtime 세부 정책 자체는 이 문서의 책임이 아니다. 현재 공식 플레이 흐름은 [game_rulebook.md](game_rulebook.md), 전투 결정론·event log·checkpoint 경계는 [core_runtime_contract.md](core_runtime_contract.md), 스킬/평타 타겟팅은 [skill_target_contract.md](skill_target_contract.md), live data loading은 [data_loading_contract.md](data_loading_contract.md)를 따른다.

## 문서 책임

| 구분 | 내용 |
| --- | --- |
| 문서 등급 | 작업 가이드 |
| 소유 범위 | 코드·문서 동기화, 리팩토링 판단, 레거시 정리 원칙, 테스트 순서, 사용자 확인 gate, goal lifecycle, Git/file safety, 완료 보고 |
| 소유하지 않는 범위 | gameplay 정책, runtime·targeting·data 계약, Unity wire payload, 개별 goal의 역사적 evidence, 완료 goal 감사 verdict |
| 함께 확인할 문서 | [README.md](README.md), 변경 주제의 canonical 문서, [goal_completion_review_guide.md](goal_completion_review_guide.md), [probe_registry.md](probe_registry.md) |

## 목적

- source of truth 개수를 줄인다.
- 같은 정책을 바꾸기 위해 열어야 하는 파일 수를 줄인다.
- 코드, live data, 문서, Unity-facing 계약, 테스트가 같은 내용을 말하게 한다.
- live 경로에서 사용하지 않는 레거시와 호환 계층을 제거한다.
- 테스트는 내부 추상화보다 실제 gameplay flow와 client-facing 계약을 고정한다.
- 리팩토링은 현재 흐름을 더 쉽게 읽고 안전하게 바꾸는 범위에서 수행한다.

## Source Of Truth 순서

프로젝트 전체의 source-of-truth 순서와 문서 등급은 [README.md](README.md#source-of-truth-순서)가
소유한다. 이 작업 가이드에서는 그 순서를 다음처럼 적용한다.

- 문서를 무조건 신뢰하지 않고 runtime code와 live data에서 현재 동작을 먼저 확인한다.
- 문서보다 더 단순하고 안전한 개선안이 보이면 판단의 코드 근거를 작업 기록에 남긴다.
- Unity-facing 변경은 외부 canonical과 실제 DTO/message producer를 함께 확인한다.
- 게임 정책 판단이 필요한 부분은 goal 기록이나 과거 문서만으로 확정하지 않고 사용자와 의논한다.

## Canonical 경계 확인

전체 문서 등급, 주제별 canonical과 탐색 기준은 [README.md](README.md)가 소유한다. 작업을 시작할
때는 README에서 변경 주제의 canonical을 선택하고, 이 문서의 변경 유형별 갱신 매트릭스로 함께
확인할 테스트와 외부 계약을 정한다.

Unity-facing 계약과 클라이언트 구현 문서는 이 저장소가 아니라 외부 Unity 프로젝트가 canonical이다.

```text
F:\unity projects\ark\docs
/mnt/f/unity projects/ark/docs
```

특히 다음 외부 문서를 기준으로 확인하고 갱신한다.

- `unity_core_contract.md`: Unity와 `/game` WebSocket의 command, snapshot, result 통합 계약
- `core_unity_battle_transport_contract.md`: battle setup, live update, checkpoint, resync, 종료 전송 계약
- `unity_client_implementation_goal.md`: Unity 클라이언트 구현 순서와 UX 작업 지침

과거 setup/update 분리 전투 문서는 `core_unity_battle_transport_contract.md`로 통합되었다. 분리 문서나 goal 기록은 migration context로만 읽는다.

이 저장소 안에는 `docs/unity_core_contract.md`, `docs/unity_client_implementation_goal.md` 같은 local copy를 두지 않는다. 같은 이름의 문서가 생기면 stale copy로 간주하고 외부 canonical을 확인한다.

## 코드와 문서 동기화

코드를 수정할 때는 다음을 함께 확인한다.

- gameplay rule 변경이면 `game_rulebook.md` 또는 가장 가까운 도메인 계약을 갱신한다.
- Unity-facing DTO, snapshot, command, admin command가 바뀌면 외부 Unity canonical 문서를 갱신한다.
- live RON schema나 authored data 의미가 바뀌면 data/도메인 계약과 loading validation을 갱신한다.
- WebSocket/admin 계약이 바뀌면 문서뿐 아니라 Python probe 또는 server message test도 갱신한다.
- 새 Unity-facing field에는 의미, casing, nullability, source of truth를 기록해 Unity가 core 규칙을 추론하지 않게 한다.
- field를 제거하거나 의미를 바꾸면 관련 legacy 문구와 테스트를 함께 정리한다.

문서 업데이트는 코드 변경의 부속 작업이 아니라 완료 조건의 일부다.

### 변경 유형별 갱신 매트릭스

| 변경 유형 | 반드시 확인/갱신할 문서 | 반드시 확인할 테스트/검증 |
| --- | --- | --- |
| `PlayerBehavior` request 추가/변경 | 외부 `unity_core_contract.md`, 관련 정책 문서 | server deserialize test, focused core test, 필요 시 WebSocket probe |
| `BehaviorResult` 추가/변경 | 외부 `unity_core_contract.md`, 관련 snapshot/command 계약 | server result mapping test, focused core test |
| `selected_event` snapshot 변경 | 외부 `unity_core_contract.md`, 관련 도메인 계약 | snapshot shape test, Python probe |
| admin command 추가/변경 | 외부 `unity_core_contract.md`, 필요 시 admin 전용 외부 계약 | server admin test, 사용 가능한 admin probe |
| Unity-facing DTO field 추가/삭제/의미 변경 | 외부 `unity_core_contract.md` | DTO shape test, server mapping test, 필요 시 probe |
| live RON schema 변경 | `data_loading_contract.md`와 관련 도메인 계약 | live RON loading/validation test |
| live RON content 의미 변경 | `game_rulebook.md` 또는 관련 도메인 문서 | live RON loading, focused gameplay/data validation test |
| live skill/delivery/reference/fragment catalog 변경 | `audit/live_skill_catalog_evidence_contract.md`, `audit/live_skill_catalog_manifest.ron`, 필요 시 `skills/live_content_index.md` | strict live loading, `live_skill_catalog_audit`, 변경 behavior를 직접 assert하는 named test |
| gameplay rule 변경 | `game_rulebook.md` 또는 관련 도메인 문서 | focused gameplay flow test |
| battle setup snapshot 변경 | `game_rulebook.md`, 외부 `core_unity_battle_transport_contract.md`, 외부 `unity_core_contract.md`, 관련 core 계약 | setup DTO shape test, server message-order test, 필요 시 probe |
| battle update/checkpoint/event 변경 | `game_rulebook.md`, 외부 `core_unity_battle_transport_contract.md`, 외부 `unity_core_contract.md`, `core_runtime_contract.md` | battle focused test, server delta/checkpoint mapping test, 필요 시 probe |
| Maintenance/Shop/Reward/Support 같은 non-combat node 계약 변경 | 외부 `unity_core_contract.md`, 관련 gameplay 계약 | node focused test, non-combat/admin probe |
| Unity 구현 목표/UX 정책 변경 | 외부 `unity_client_implementation_goal.md` | Unity 쪽 goal/test/probe, core 계약과 충돌 여부 확인 |

매트릭스에 없는 변경도 Unity가 표시하거나 command로 호출하는 표면이면 Unity-facing 계약 변경으로 취급한다.

## 구현 원칙

장기적인 구조를 선택하되, 미래 가능성을 위한 추상화를 먼저 만들지 않는다.

기본적으로 피할 것:

- 임시방편이나 특정 테스트만 통과시키는 패치
- compatibility layer, fallback path, dual schema
- 레거시 동작을 보존하기 위한 adapter
- ignored test로 구정책을 보존하는 방식
- Unity가 live RON을 직접 읽거나 core 규칙을 재계산하게 하는 구조
- typed DTO와 shape test 없이 `json!` 조립을 계속 확장하는 방식
- 실패한 테스트의 기대값만 바꾸고 사용자-visible behavior를 확인하지 않는 방식

정말 호환 계층이 필요하면 이유, 제거 조건, 지원 범위, 테스트 범위를 goal 문서에 기록하고 사용자 확인을 받는다.

처음부터 장기 구조가 명확하지 않다면 작은 trial and error로 확인한다. 실패한 접근과 이유를 `EXPERIMENTS.md`에 기록해 같은 시도를 반복하지 않는다.

## Goal Lifecycle And Review Order

Goal은 active 작업의 범위와 evidence를 기록하며 durable 정책을 대신하지 않는다. Master campaign이
있으면 master `PLAN.md`의 child/status table이 dependency와 활성 순서를 소유한다.

상태:

- `active`: 구현 또는 review 진행 중.
- `blocked_by_dependency`: 선행 작업이 닫힐 때까지 시작하지 않음.
- `deferred_by_user`: activation trigger와 새 사용자 지시 전까지 보류.
- `complete` + `Implementation complete: true`: 해당 goal exit gate와 자체 review 통과.
- `correction_required`: material defect가 있어 correction과 같은 review rerun이 필요.
- `excluded`/`not_applicable`: 근거와 owner를 기록한 terminal disposition.

Child는 runtime/data/external contract/canonical 변경과 focused→broad evidence를 완료한 뒤에만
complete가 된다. Phase review는 child verdict를 proof로 재사용하지 않고 결합 경계를 독립
검증하며, defect가 있으면 correction 뒤 같은 review를 다시 실행한다. Phase가 Complete/High일
때만 다음 phase를 열고 마지막 master review가 전체 scope와 disposition을 재검토한다.

완료 goal의 장기 정책은 durable canonical에 옮긴다. Goal 문서는 historical evidence로 보존할 수
있지만 현재 정책처럼 인용하지 않는다.

## 리팩토링 판단

Refactor는 현재 기능의 ownership과 source-of-truth 경계를 단순화할 때 수행한다. 이동/targeting/
Unity UI/밸런스처럼 수명주기와 검증 표면이 다른 변경은 별도 goal로 나누고, 외형이 비슷하다는
이유로 다른 gameplay contract를 합치지 않는다.

판단 순서:

1. Runtime/live data에서 중복, 산탄 수정, fallback 같은 문제가 실제로 확인됐는가?
2. 먼저 obsolete code/schema를 삭제할 수 있는가?
3. 기존 함수와 typed range/targeting/delivery/validation 조합으로 같은 의미를 표현할 수 있는가?
4. Timing, event order, target eligibility, DTO와 사용자-visible 동작이 유지되는가?
5. Canonical source와 derived cache/snapshot/projection을 구분했는가?
6. Focused test 또는 probe로 결과를 고정할 수 있는가?
7. Unity DTO, RON schema, save migration, UX, balance 또는 gameplay 결정이 필요한가?

판정:

- **리팩토링 후보**: source가 중복되거나 기존 기능 조합으로 대체 가능하고 검증 안전망이 있다.
- **보류**: 의미가 달라질 수 있거나 안전망이 부족하다.
- **하지 않음**: 다른 수명주기/계약을 모양만 보고 합치려는 작업이다.
- **사용자 확인 필요**: 정책 또는 외부 shape 결정을 먼저 받아야 한다.

과도한 추상화 신호는 실제 변형보다 먼저 trait/policy/strategy 계층을 만들거나, 단순 함수로
충분한데 dynamic dispatch/generic을 추가하거나, 미래 기능 때문에 현재 API/RON/DTO를 복잡하게
만드는 경우다. 추상화 이름보다 gameplay 규칙이 먼저 읽혀야 하고 테스트는 helper 배치보다
동작을 보호해야 한다.

우선 점검축은 상태 전이와 allowed action, map→preview→battle handoff, core/server command,
result/reward commit, live RON과 runtime validation의 ownership 일치다. 한 goal에서는 source 하나
또는 밀접한 경계 하나만 정리한다.

## 레거시와 문서 정리

- 현재 정책과 충돌하는 레거시는 compatibility layer나 dual schema로 보존하지 않는다.
- 구 테스트는 기대값만 고치지 말고 보호하던 사용자-visible behavior를 확인한다. 최신 focused test로 교체하거나 의미가 없으면 제거한다.
- 공식 live flow에 없는 data, fixture, 문서가 source of truth처럼 보이지 않게 정리한다.
- 외부 Unity canonical 대신 local stale copy만 갱신해 작업을 완료하지 않는다.
- git 과거 내역에서 코드를 복사해 현재 파일에 붙이지 않는다. 꼭 필요하면 이유를 설명하고 사용자 허락을 먼저 받는다.

durable canonical 문서에는 현재 의사결정과 client-facing 계약만 남긴다.

- 완료된 goal의 장기 정책은 가장 가까운 durable 계약으로 옮긴다. goal 문서는 historical
  evidence로 유지할 수 있지만 current source처럼 링크하거나 인용하지 않는다.
- 완료된 실험 로그, 감사 로그, 구현 후기, 과거 작업 목록을 canonical 현재 정책처럼 쓰지 않는다.
- 미래 아이디어는 현재 구현 지시처럼 쓰지 않고, 필요할 때 짧은 TODO 또는 보류 정책으로 구분한다.
- goal 문서는 진행 중 작업의 기억장치이며 durable 계약의 대체물이 아니다.

## 테스트와 검증

테스트는 내부 helper 배치보다 다음 계약을 고정한다.

- 사용자-visible behavior
- Unity-facing DTO shape
- server command/result mapping
- live RON loading과 data validation
- 실제 gameplay flow와 timing/order
- admin/debug command와 WebSocket 계약

권장 검증 순서:

1. focused unit/integration test
2. 관련 `cargo check`
3. 관련 package test
4. live RON 변경이면 loading/validation test
5. Unity-facing 변경이면 server mapping test와 WebSocket/Python probe
6. 필요한 범위의 broader `cargo test`

대표 명령은 현재 workspace package 이름과 변경 범위에 맞게 조정한다.

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

WebSocket/admin probe 경로와 실행 gate는 [probe_registry.md](probe_registry.md)를 사용한다.
예를 들어 admin non-combat probe는 다음과 같다.

```text
APP__SERVER__BIND_ADDRESS=127.0.0.1 APP__SERVER__PORT=18083 ENABLE_ADMIN_COMMANDS=true ADMIN_COMMAND_TOKEN=dev cargo run -p game_server
ADMIN_COMMAND_TOKEN=dev WS_PORT=18083 python3 "/mnt/f/unity projects/ark/docs/probe/unity_admin_debug_command_probe.py"
```

작은 변경 단위마다 빠른 검증을 실행하고, 실패 원인, 수정 내용, 재검증 결과를 `EXPERIMENTS.md` 또는 동등한 작업 기록에 남긴다. probe 실행 뒤에는 자신이 만든 로그와 임시 부산물만 정리한다.

### 테스트와 debug 산출물 분류

- `tests/`: 공식 Rust integration test 위치
- `src/game/**/tests/`: 해당 runtime module의 공식 unit/integration-style test 위치
- `tests_bak/`: Cargo test 대상이 아닌 legacy backup; 현재 정책으로 재작성하기 전에는 근거로 사용하지 않음
- `event_log_exports/`, `target/debug_event_log_exports/`: 재생성 가능한 debug event-log output; golden fixture가 아님
- `target/combat_traces/`: 실제 battle UUID별 재생성 가능한 비권위 debug JSON trace; runtime/save/checkpoint state가 아니며 생성 실패는 gameplay transition에 영향을 주지 않음
- `logs/`, `tmp/`: runtime/작업 산출물; 정책과 fixture가 아님
- `src/old/`: live code가 참조하지 않는 한 legacy 보관 위치

debug 파일을 golden fixture로 만들려면 테스트가 그 shape를 명시적으로 고정해야 한다. 기존 산출물 디렉터리의 삭제나 추적 상태 변경은 별도 cleanup 범위 또는 사용자 확인을 거친다.

## 사용자 확인이 필요한 변경

다음 항목은 임의로 확정하지 않는다. 필요한 코드/data 근거와 선택지를 정리하고 작업을 중단한 뒤 사용자에게 질문한다.

- Unity-facing DTO shape 변경
- live RON schema 변경
- 저장 데이터 migration
- UX 의미 변화
- 밸런스 기준
- 기존 콘텐츠 삭제 또는 대체
- 실패, 보상, 소비 시점 변화
- 게임 룰 또는 노드 흐름 변경
- 전투 성공/실패 판정 변경
- 장비, 스킬, 아이템 성장 정책 변경
- core와 Unity 중 source of truth가 불명확한 경우

goal 완료 조건에는 다음 문장을 포함한다.

> 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Goal 운용

큰 작업은 `docs/goals/<goal_name>/` 아래 다음 파일로 관리한다.

- `PLAN.md`: objective, source of truth, scope, 구현 계획, 완료/중단 조건, 검증 명령
- `EXPERIMENTS.md`: 시도와 실패 원인, 수정, 재검증 결과
- `EXPERIMENT_NOTES.md`: 정책 충돌, 판단, 범위 밖 후속 후보

범위 밖 개선은 note에 남기고 현재 완료나 blast radius 축소에 필수일 때만 같은 goal에서 다룬다.
PLAN에는 최소한 다음 구조를 둔다.

```md
# <Title> Goal

## Objective
## Source Of Truth
## In Scope
## Out Of Scope
## Implementation Plan
## Test Requirements
## Completion Conditions
## Stop Conditions
```

Objective는 한 문장으로 쓰고 focused→broad 검증과 사용자 정책 결정이 필요한 stop condition을
명시한다.

## Git/File Safety

- `git restore`, `git reset` 같은 git file modifier를 수행하지 않는다.
- git 과거 내역의 코드를 현재 파일에 `cp`하지 않는다.
- 파일을 과거 상태로 되돌려야 한다면 이유를 설명하고 사용자 허락을 먼저 구한다.
- dirty worktree의 기존 변경은 사용자 변경으로 간주하고 되돌리지 않는다.
- unrelated change를 건드리지 않는다.
- 자신이 만든 임시 로그와 테스트 부산물만 정리한다.

## 완료 기준과 보고

완료 전 확인:

- 가장 가까운 canonical과 runtime/data/DTO가 같은 내용을 말하는가?
- Unity-facing shape와 RON 변경을 해당 mapping/shape/loading test 또는 probe로 검증했는가?
- Stale copy, legacy 문구, 새 compatibility/fallback/dual schema가 남지 않았는가?
- Unity가 core 규칙을 재계산하지 않고 테스트가 사용자-visible 동작을 보호하는가?
- 자신이 만든 임시 부산물만 정리했는가?

완료 보고에는 변경·계약·제거한 legacy, 갱신한 문서/test/probe, 실행한 검증과 결과, 남은 위험과
정책 질문을 포함한다. 하나라도 불명확하면 완료하지 않고 code/data를 다시 확인하거나 사용자에게
질문한다.
