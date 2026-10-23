# S01-P1 Run snapshot 소비 경계·구성 비용과 개선 범위

작성·갱신일: 2026-10-18

**선택한 Core/서버 경계 조사·비용 분리·후속 설계 완료. 성능 수용과 Unity 통합 확인은 미완료다.**
현재 snapshot은 117,922 bytes이며 생성·Value 구성만으로 p95 0.610926ms였다.
직렬화만의 개선이나 기존 JSON을 그대로 보관하는 캐시로 시간·크기 두 상한을 함께 해결할 수 없다.
게임 실행 코드·공개 DTO·cap은 유지하고, 내부 생성 비용 후보와 공개 응답/예산 판단을 분리한다.

## 1 범위·기준·재사용

[README](README.md)의 S01-P1 재개 지시와 [1차 측정 §4](PERFORMANCE_RESOURCE_MEASUREMENT.ko.md#4-설계-판단과-확정-후속)를 따른다.
[기준](../refactoring_criteria.ko.md) §3.1–3.3·5·7.4와 [실행 지침](../system_refactoring_execution_guide.ko.md) §3–6·9를 적용한다.
기존 [runtime engineering cap](../core_runtime_contract.md#final-core-performance-guard)은 `<500µs / <64KiB`로 유지된다.
실제 사용자 체감 SLA·전투 tick 예산과 동일한 의미로 해석하지 않는다.

[입력 대조](evidence/snapshot_001/reused-evidence.json)의 18개 직접 입력이 이전 기록과 일치했다.
기존 snapshot/history/checkpoint·전투 부하 결과와 선정 리팩토링의 기능 근거를 재사용한다.
이번에 읽은 생산자·소비자 13개 경계는 [선택 입력](evidence/snapshot_001/selected-inputs.json)에 기록했다.
전체 소스·기능·feature 조사나 테스트를 반복하지 않았다. 기존 dirty 변경은 이번 성과로 귀속하지 않는다.

추가 실행은 기존 public_guards에 명시적인 `--inspect-snapshot <output.json>`을 더한 plain release 진단 1회다.
준비 함수·기본 guard 본문·101 표본·상한을 유지했고 기본 판정 경로를 우회한 통과 주장을 하지 않는다.
진단 모드는 `performance_pass_claimed=false`, `baseline_comparison_applicable=false`를 남긴다.
release build는 79초·exit 0, 진단은 0.67초·exit 0이었다. [build 로그](evidence/snapshot_001/build.log),
[명령·binary·affinity](evidence/snapshot_001/execution.json), [raw 표본과 실제 snapshot](evidence/snapshot_001/inspection.json)을 연결한다.
진단 exit 0은 관측 성공이며 성능 기준 충족이 아니다. allocator·기존 전체 guard·P01 비교는 다시 실행하지 않았다.

## 2 실제 생성·소비와 갱신 책임

| 경계 | 확인한 구현·책임 | 변경 시 보존할 의미 |
| --- | --- | --- |
| Core 생성 | [snapshot.rs](../../src/game/world/snapshot.rs)의 `get_run_snapshot_dto` → `get_run_snapshot_json`, [application.rs](../../src/application.rs)의 `snapshot(Run)` | 현재 원정·profile·publication에서 매번 파생. authoritative state와 조회 DTO를 구분 |
| JSON projection | [projection.rs](../../src/authoring/projection.rs)의 `to_value` | 일반 호출은 native serde Value 구성. authoring scope에서는 byte/work admission을 선행하므로 제어된 경로의 검사를 임의 제거할 수 없음 |
| 시설 | [facility_rooms.rs](../../src/game/world/facility_rooms.rs)의 `facility_view`, [facility_layout.rs](../../src/game/map/facility_layout.rs)의 DTO | 현재 방 원본은 RunState, 다른 방문방은 visited_rooms, 영구 지식은 profile, 정적 정의는 publication. 조회가 별도 상태 권위가 되면 안 됨 |
| 스킬 | `get_skill_catalog_snapshot_dto`가 publication의 전체 skill을 변환·ID 정렬, version 3 생성 | 현재 특정 직원의 장착 스킬 목록으로 축소할 근거가 없음. 캐시를 고려한다면 해당 publication 수명에 귀속 |
| 서버 | [actor read_snapshot](../../../game_server/src/game/player_game_actor/mod.rs)에서 ApplicationSnapshot Value clone → RunSnapshotDto decode; [build_state_snapshot](../../../game_server/src/game/player_game_actor/handlers.rs)에서 [host overlay](../../../game_server/src/game/player_game_actor/state.rs) → Value 재구성 | selected_event 운반, host persistence·Continue 상태, action capability 합성, fault 전용 snapshot을 보존 |
| 발행 빈도 | attach/reconnect·일반 변경 명령·전투 종료에서 전체 snapshot 발행. battle command mapping은 `send_state_snapshot=false`, 일반 tick은 battle_update만 발행 | 전체 snapshot 비용을 모든 전투 tick 비용으로 계산하지 않음. ReadOnly 및 전투 전용 메시지 계약 유지 |

서버의 Value clone/decode/re-encode는 실제 추가 작업이다. 이번 Core guard 시간에는 포함되지 않는다.
typed 전달이나 host overlay 조립의 변경은 서버 소비 경계를 함께 검증해야 하며, Core 내부 생성 시간의 해소와는 별도 후보다.

### 시설 공개 계약과 겉보기 중복

[runtime U13](../core_runtime_contract.md#u13-authored-facility-rooms)은 일반 방 위치·연결 공개,
이름·컨셉의 인접/방문/영구 탐사 조건, 숨은 최종방·연결 제외, 방문방 활동·내부지도와 영구 지식의 차이를 규정한다.
시설 전체를 정적 catalog로 취급하거나 미방문 방을 일괄 제거하면 이 계약이 달라진다.

최상위 `map`과 현재 방의 `current_expedition_map`은 각각 current_map_view와 facility_view가 만든다.
둘 다 entry preview helper를 재사용하지만 최상위 지도는 전투 준비·상대 위협도까지 추가한다.
실제 관측에서 전투 노드 네 개의 `relative_threat`와 Gate의 `entry_preview`가 달랐다.
두 JSON을 동일 사본으로 치환하는 최적화는 동작 보존이 아니다.
규칙 권위는 helper에 유지하고, 이후 중복 생성 최적화가 필요하면 공통 기초 지도와 경계별 decoration을 구분해야 한다.

### 외부 Unity 확인의 제한과 발견 사항

[문서 인덱스](../README.md#외부-unity-canonical-문서)가 지시한
`/mnt/f/unity projects/ark/docs/unity_core_contract.md` 및 `core_unity_battle_transport_contract.md`는 현재 없다.
같은 이름은 `docs/archive/`에서 발견했으나 현행 canonical로 승격하지 않았다.
따라서 공개 응답 형식 전환의 현행 외부 계약 확인은 미완료다.

로컬 [GameSnapshotStore.cs](</mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameSnapshotStore.cs>)는
ApplySnapshot에서 `State = state`로 교체하고 map·action capability를 읽는다.
[NodeMapSnapshotModel.cs](</mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/Reading/NodeMapSnapshotModel.cs>)의 실제 지도 reader는 최상위 map/navigation을 사용한다.
따라서 서버가 필드 생략을 delta로 바꿔도 클라이언트가 이전 값을 병합할 것이라고 가정할 수 없다.

[SkillCatalogModel.cs](</mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/Reading/SkillCatalogModel.cs>)는 version 2만 허용하지만
현재 Core 출력은 version 3이다. **reader를 호출하면 거절되는 정적 schema 불일치**로 기록한다.
Runtime 소스 검색에서 이 reader의 외부 호출은 찾지 못했고 EditMode 테스트는 이를 직접 호출한다.
이 사실을 현재 화면의 실제 장애 재현으로 확대하지 않는다. 로컬 runtime에서 root `facility`를 소비하는 경로도 확인하지 못했다.
서버는 전체 DTO를 운반하고 Core 공개 계약은 존재하므로, 이 검색 결과만으로 어느 필드도 미사용·삭제 가능으로 판정하지 않는다.
Unity 실행·제품 수정·외부 계약 정리는 이번 범위 밖이며 외부 통합 owner의 후속이다.

## 3 측정한 비용과 크기

기존 `live_run_with_checkpoint`와 같은 seed·입력·release 환경이다.
관측 시점은 **SavePoint 완료 직후 ViewingMap, 전투 준비 전**이다. 이후 별도 staging guard의 전투 상태와 구분한다.
한 방 방문·영구 탐사 0개, 공개 방 85개·연결 127개·스킬 36개다. 전체 콘텐츠·최대 방문 이력 부하가 아니다.

| 구간 | median | p95 | 의미 |
| --- | ---: | ---: | --- |
| `snapshot(Run)` 반환까지 | 0.576361ms | 0.610926ms | DTO 생성 + Value 구성 + application 후처리. 필드별 생성 시간을 따로 잰 것은 아님 |
| 반환된 snapshot compact 직렬화 | 0.095331ms | 0.103666ms | 이미 생성된 Value와 ApplicationSnapshot wrapper를 byte 배열로 인코딩 |
| 같은 표본의 두 구간 합 | 0.673043ms | 0.707479ms | split timer 관측. 기본 guard의 이전 0.728398ms와 회귀 비교하지 않음 |

각 구간은 101개이며, 같은 표본에서는 구간 시간 합이 total과 일치한다.
각 분포의 p95를 더해 total p95라고 하지 않는다. equality 검사·소멸·파일 쓰기는 측정 밖이고 서버/Unity 비용도 포함하지 않는다.
출력 117,922 bytes와 top-level 구성 byte는 이전 측정과 모두 일치한다. 모든 표본이 reference와 byte 단위로 동일하다.
생성 구간의 내부 DTO/Value 비중과 개별 facility/skill 생성 시간은 미확인이다. 아래 직렬화 시간을 해당 필드의 생성 시간으로 사용하지 않는다.

| 구성 | JSON value bytes | 이미 생성된 구성의 직렬화 p95 |
| --- | ---: | ---: |
| facility | 51,821 | 37.010µs |
| skill_catalog | 32,379 | 23.825µs |
| roster | 16,684 | 11.232µs |
| inventory | 6,651 | 4.268µs |
| map | 5,266 | 3.507µs |

facility 내부의 rooms 배열은 42,036 bytes, connections는 9,452 bytes다.
현재 방 하나는 13,740 bytes이고 그 안의 map 4,993·activities 5,661·interior 2,728 bytes가 포함된다.
나머지 공개 방 정보와 연결도 큰 비중이다. 이 값들은 중첩된 부분집합이므로 전체와 합산하지 않는다.
시설/스킬 두 value만 84,200 bytes다. 캐싱·소유 사본 축소는 동일 JSON의 byte 수를 줄이지 못한다.
현재 데이터가 기존 cap을 초과한다는 사실과 cap/응답 계약 중 무엇을 조정해야 하는지는 별도 판단이다.

## 4 설계 판단·후속 작업의 경계

| 항목 | 분류·판정 | 목표·검증과 다음 처리 |
| --- | --- | --- |
| 시설 상태·지식·정적 정의의 분리 | 유지 | 공개 조건과 현재/방문방 원본이 구분된다. 영구 지식에서 과거 활동 상태를 재구성하거나 무효화 책임 없는 전체 cache를 만들지 않음 |
| 최상위/시설 지도 조립 | 유지, 비용 후보 | 현 차이는 실제 표시 의미다. 규칙 helper 공유는 유지. 공통 DTO로 무조건 통합하지 않음 |
| 정적 template의 owned 반환 | **S01-P2 내부 개선 후보** | `node_flow::facility_template_for_map`은 immutable publication에서 검증 후 template 전체를 clone한다. snapshot의 current_map_view·facility_gate_reached 등 읽기 경로도 이를 사용. borrowed lookup을 공통 원본으로 하고 소유가 필요한 전이만 명시적으로 clone하는 대안을 검토. 공개 bytes·오류·저장 의미는 유지 |
| 스킬 catalog 캐시 | 보류 | 정적 입력이라는 근거는 있으나 DTO 생성/Value 구성 각각의 비용과 캐시 clone 비용이 아직 분리되지 않음. GameCore/후보 복사 수명에 새 cache를 무작정 추가하지 않음 |
| 서버 Value→DTO→Value | **S01-P3 경계 개선 후보** | 중간 표현·전체 clone이 실제 존재. typed read boundary와 기존 Value API 소비자를 함께 설계해야 함. Core cap 밖이고 host overlay/fault·selected_event를 보존해야 하므로 현재 내부 작업에 섞지 않음 |
| 응답 축소 또는 64KiB 예산 변경 | **S01-C1 공개 계약 판단 필요** | 현행 Core 계약·현재 외부 소비 대상/계약·발행 빈도·방문 규모를 기준으로 전체/분할 응답과 예산의 범위를 함께 판단. 필드 삭제, 새 delta API, cap 증액을 이번에 결정하지 않음 |
| Unity reader v2 / Core v3·canonical 경로 | **S01-C2 외부 연결 미확인/불일치** | 정적 불일치와 누락 경로를 기록. 현행 배포/개발 대상과 계약 owner 확인 뒤 외부 통합 범위에서 처리. 현재 화면 장애·Core 오류로 단정하지 않음 |

### S01-P2를 선택할 경우의 구체적 구현 단위

문제는 immutable template lookup이 모든 caller에게 소유 사본을 강제한다는 점이다. 규칙·상태 권위의 중복은 아니다.
현재 helper는 snapshot 외 이동·복원·전투/노드 완료도 사용하므로 이름만 바꿔 모든 caller의 ownership을 추정하지 않는다.
단위는 **template lookup의 borrowed/owned 책임 정리**이며 전체 facility cache나 snapshot schema 개편과 독립적이다.

1. 같은 `self.game_data`에 묶인 borrowed resolver가 lookup·version 오류를 한 곳에서 결정한다.
   기존 owned caller가 필요하면 이 resolver의 결과만 clone한다. 정책 검증을 병렬 구현하지 않는다.
2. 우선 current_map_view·facility_gate_reached의 읽기 경로를 borrowed로 연결한다.
   mutable transition caller는 lifetime이 실제로 필요한 곳만 owned 경로를 유지한다.
3. 기존 map/시설 은닉·node preview·Continue/SavePoint 사례와 같은 seed 전체 JSON 동등성을 사용한다.
   기준선의 실패(cap 초과)를 보존하며 같은 plain guard의 전후 비용과 복사 감소를 확인한다.
4. 수용은 template 권위/검증의 단일성, 읽기 경로 불필요 clone 제거, 공개 결과·오류·순서의 보존이다.
   별도 전후 측정 없이 성능 개선을 주장하지 않고, 이 작은 변경이 64KiB 문제를 해결한다고 하지 않는다.

후보 구현 시 기존 [u13_facility 테스트](../../src/game/world/tests/u13_facility.rs)의
`u13_official_public_disclosure_and_unsupported_approach_are_read_only`,
`u13_real_gate_roundtrip_preserves_assignments_receipts_and_continue`,
`u13_fixed_encounter_disclosure_tracks_real_room_completion_and_continue`를 중심으로 영향 범위를 정한다.
이 목록은 다음 구현의 검증 계획이며 이번에 해당 테스트를 다시 실행했다는 뜻이 아니다.

이는 구현 가능한 후보 설계이며 이번에는 실행하지 않았다. 현재 읽기 비용만으로 긴급 최적화 필요를 단정하지 않는다.
우선 독립된 기존 후속 **X01-P1 concurrent 단계별 비용 귀속**을 진행한다. 10ms 초과 원인이 thread 시작·core 생성·step·직렬화 중
어디인지 구분한 뒤 측정된 효과와 위험으로 구현 우선순위를 판단한다.
S01-C1/C2는 snapshot 형식/예산 전환 전에 필요한 조건이고 X01 조사나 내부 계약 보존 후보의 선행 조건은 아니다.

## 5 분리 검토·종료점

측정과 문서 정리 뒤 별도 읽기 전용 pass에서 현재 소스·기본 guard 보존·원시 표본·크기·실제 소비·한계를 대조했다.
[대조 결과](evidence/snapshot_001/validation.json)와 [완료 검토](evidence/snapshot_001/review.json)가 판정을 소유한다.
동일 작성자의 분리 검토이며 비참여 독립 감사가 아니다.

이번 완료 범위는 Core/서버의 공개 snapshot 흐름, generation/encoding 분리, 실제 payload 구성, 내부/계약 후보의 분리와 구현 설계다.
**성능 기준 초과는 남아 있고, 구성별 내부 생성 비용·현행 외부 계약·Unity 실행은 미확인으로 유지한다.**
성능 해결·제품 구현·모든 소비자 조사 완료를 주장하지 않는다. 다음 실행 위치는 X01-P1의 `concurrent_sample`이다.


### 후속 구현 기록 (2026-10-18)

위 본문은 S01-P1 종료 시점의 조사·계획이다. X01-P1을 거쳐 선정한 S01-P2를
[시설 template 소유권 구현·검토](S01_TEMPLATE_BORROW_IMPLEMENTATION.ko.md)에서 구현했다.
두 읽기 호출의 전체 clone 제거, 기존 영향 테스트 7개와 전체 snapshot 동일성을 확인했다.
최초 지연 증가와 같은 CPU 6쌍 교차 비교의 혼합된 결과도 보존하며 성능 수용은 미완료다.
현재 다음 행동은 [README](README.md)의 X01-P2 step 내부 비용 귀속이다. 기존 S01-C1/C2·S01-P3는 유지한다.
