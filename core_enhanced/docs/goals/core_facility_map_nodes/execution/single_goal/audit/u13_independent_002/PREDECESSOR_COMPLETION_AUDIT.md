# U13 최종 독립 감사 — Not Complete

GS-CFMN-001 / GOAL-CFMN-01 / PHASE-CFMN-01의 `u13_current_001`에 대한 GEX-20 판정은 **Not Complete, 열린 완료 차단 finding 2건**이다. 두 건 모두 정상 RON 제작 → 공식 pack 작성/로딩 → 공개 Core 명령/조회로 재현했으며 `범위 안 — 지원 경로`다. 제품·동결 입력·기존 증거는 수정하지 않았다.

## 대상과 독립성

- 사용자 요청: `CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md`를 읽은 뒤 U13 최종 독립 감사 착수. 현재 요청은 GEX-20 `stage-run`이며 제품 교정·U14 실행을 포함하지 않는다.
- 대상: [Audit Target Record](target/u13_current_001/AUDIT_TARGET_RECORD.json), HEAD `2f725172edbffd27c5078d29000a1a2ba6ddc4d9`, SOURCE SHA-256 `b941dfed5114b62337b2db060ee0601baaf736b3c48da33ef70bf85fd73f9a10`.
- 상위 계약: HA-CFMN-001 / FPF-CFMN-001 / PP-CFMN-001, [정식 Goal의 A01–A30](../../P2G_50_GOAL_SET.ko.md), GEX revision `2026-10-04-gex-r8`, 연속 지시서 §4·7·8와 로드맵 §1.9. 작성 artifact의 미활성 상태는 당시 P2G 권한 표시이며 현재 사용자 감사 권한과 구분한다.
- 감사자는 현재 대화에서 U13 구현·설계·교정에 참여하지 않은 별도 실행자다. 구현 pass와 작업 문맥·쓰기 범위·산출물을 분리했다. 서브에이전트/위임 없음. 확인하지 않은 실행 모델 정체성을 주장하지 않는다.
- [입구 대조](audit/u13_independent_001/CURRENTNESS_ENTRY.json): source695/evidence1082 전체 바이트·크기와 record의 주요 artifact hash, HEAD 일치. [출구 대조](audit/u13_independent_001/CURRENTNESS_EXIT.json)와 함께 `current / unchanged`를 확인한다. 기존 dirty 변경을 U13 변경으로 새로 귀속하거나 되돌리지 않았다.
- 감사 source review는 동결 SCOPED.diff의 누적82변경, 새 facility_layout/facility_rooms, 실제 loader/allocator/query/entry/Continue/profile/store 소비 경로를 기준으로 했다. 감사 파일·관측용 crate·출력은 target의 명시적 제외 영역이다.

## F-U13-001 — 미탐사 고정 조우 ID가 다른 공개 지도 경로에서 노출됨 (P2)

정상 작성 데이터에 유효한 조우 ID를 가진 고정 조우가 있으면, **방을 완료하거나 해당 노드에 입장하기 전에** `Run.map`과 `RequestMapData`가 `suppress_little_red`를 노출한다. 같은 snapshot의 `facility.rooms[].current_expedition_map`은 같은 노드의 ID를 null로 가린다. 수용 A19와 A24의 비공개 정보 보호를 충족하지 못한다.

원인은 [current_map_view](../../../../../src/game/world/node_flow.rs:205)가 `MapProgression::view`의 payload를 그대로 소비하고, [facility_view](../../../../../src/game/world/facility_rooms.rs:129)에만 고정 조우 비공개 처리가 있기 때문이다. Run snapshot과 RequestMapData는 전자를 사용한다. 제품의 단일 상태 권위와 별개로 공개 투영의 필터가 일치하지 않는다.

[재현 원본](audit/u13_independent_001/observations/hidden_concrete/run.json), [지도 조회](audit/u13_independent_001/observations/hidden_concrete/map_query.json), [작성 입력](audit/u13_independent_001/observations/hidden_concrete/facility_templates.ron): 공식 복사본에서 검증 가능한 기존 A2-02의 방 템플릿/예약을 초기 B1-01에 배치하고, 기존 catalog에 있는 `suppress_little_red`를 그 예약에 참조했다. 깊이1에 맞춰 정예 후보를 제거했으며 미지원 실행 상태는 유지했다. 공식 packaging과 loader가 수용한 뒤 실제 브리핑·starter 선택까지만 진행했다. `explored=false`, activity label=null, facility map encounter_id=null인데 일반 지도 encounter_id는 실제 ID다. 조회 전후 상태는 동일하다.

공식 unresolved 활동(`encounter_id=None`)을 그대로 쓴 대조군도 로더를 통과했고 미탐사 활동에 `미확인 고유 조우 실행 미지원` 표시가 나타났다. **이 표시만을 별도 finding으로 세지 않는다.** 현재 공식 placeholder 자체가 구체 ID를 노출했다고 주장하지도 않는다. 완료 차단 근거는 loader가 지원하는 구체 참조 입력에서의 실제 identity 노출이다. 작성 입력 수정은 지원 제작 행위이며 저장·checksum·메모리 조작이 아니다.

반환 owner: GOAL-CFMN-01의 공개 정보 경계(GEX-10). 교정 후 같은 미탐사/탐사 완료 입력에서 Run, RequestMapData, 시설 조회 및 관련 preview의 공개 조건을 대조해야 한다. 기존 숨은 최종방·인접 이름·지원 거절 검증은 유지한다.

## F-U13-002 — 배정 역할과 실제 활동 종류의 불일치를 정상 로더가 수용함 (P2)

공식 작성 입력의 `allocation.normal_activity_id`를 기존 `maintenance`로 잘못 지정하면 공식 packaging/loader가 성공하고, 실제 B1-01에 **Maintenance6·Combat0·Event1·Reward2**를 생성한다. 지원 총수 최대2와1층 일반전 최소2라는 A09/A10을 위반한다. A08/A29의 지원 작성 오류 검증과 관련 통합 seam에도 영향을 준다.

[FacilityLayout::validate](../../../../../src/game/map/facility_layout.rs:155)는 배정 참조의 존재를 확인하고, [live authoring validation](../../../../../src/game/data/live_ron.rs:1145)은 각 활동 payload와 자체 node kind 및 slot category의 정합을 확인한다. 그러나 `normal_activity_id`가 일반전 역할을 수행하는지는 확인하지 않는다. `count_combinations`의 남은 일반전 수와 `sample_activities`가 채우는 실제 활동 의미가 분리되어 있다. 정비는 ordinary slot에서 허용되는 category라 기존 category 검사를 통과한다.

[재현 원본](audit/u13_independent_001/observations/wrong_normal_role/run.json), [작성 입력](audit/u13_independent_001/observations/wrong_normal_role/facility_templates.ron): 독립된 공식 복사본에서 위 참조 한 곳만 바꾸고, 정상 pack/loader와 공개 시작 명령을 사용했다. catalog 의미와 플레이 상태를 직접 주입하거나 저장을 변조하지 않았다. 정상 제작 시 가능한 잘못된 참조 지정이다. 기존 공식 참조의 지원 분포·균등 추첨 시험 자체를 실패로 바꾸지는 않지만, 그 통과가 허용된 작성 경계 전체의 불변 조건을 증명하지 않는다.

반환 owner: GOAL-CFMN-01의 작성 validation/배정 경계(GEX-10). 잘못된 역할 참조를 정상 입력 검증에서 거절하고 올바른 참조는 받아들이는 대조가 필요하다. 같은 방식으로 지원/정예/Event/보상 역할이 개수·적격성 검증과 실제 활동 의미를 일치시키는지도 교정 영향 범위에서 확인한다. 배정 확률·최소 일반전·지원 한도를 바꾸는 정책 변경은 요구하지 않는다.

## 검증과 재사용

이번에 실행한 것은 감사 전용 독립 공개 consumer의 **3개 유한 관측 사례**다. release offline/locked build와 실행은 exit0이며 3개 모두 정상 로더 수용, 그중2개가 위 결함의 실제 반례다. 관측기 종료 성공을 제품 PASS로 집계하지 않는다. [사전 설계](audit/u13_independent_001/PROBE_PLAN.md), [명령·binary hash·종료 상태](audit/u13_independent_001/PROBE_RECEIPT.json), [관측 요약](audit/u13_independent_001/observations/RESULT.json)에 결속했다. 재현 시 receipt의 build 명령을 사용하고 실행 출력 경로는 새 빈 경로로 지정한다.

기존 기능 시험을 전부 다시 실행하지 않았다. 입력 바이트·변경 영향과 실제 원문을 확인해 다음 범위에서 재사용했다. [독립 원시 대조](audit/u13_independent_001/RAW_REVIEW.json), [재집계 도구](audit/u13_independent_001/review_evidence.py)를 참조한다.

- 공식89방/846활동,88개 활동방의 ID·깊이·계열·이름·템플릿·예약·전조·정예·안전 접근 목록,139연결을 채택된 배정 JSON과 대조해 차이0. 기존 좌표 독립성·seed·5000표본 분포 시험의 oracle과 source를 검토했다.
- `test013`119/119, `test012`1/1, server store1/1의 실제 선택/통과/종료를 확인했다. `test014`10/12의2실패는 당시 V12/Standard 기대값이며 원문을 보존한다. 전체 저장소 PASS가 아니다. 현재V14/Expedition, 진짜 구writer12/profile4 증거와 분리했다.
- 실제 승인 호스트30단계에서 각 실행 exit0, 개별 RESULT passed, 개별 admission passed를 재집계했다. transient 선택·입장은 같은 프로세스이며 저장 가능한 경계의 재시작이다. `saved.json`과 `resumed.json`의 전체 공개 roots 및 checkpoint bytes가 동일함을 직접 비교했다.
- 전투 원시 기록의 briefing/실제 setup10필드가 일치하고, 실제 배치 명령3개·host tick202개·종료 및 guard/marksman 등록을 확인했다. 미발생 wave·소환·preview·패배/후퇴의 제외는 기존 선택 시험과 후보 수집/승리 commit 코드를 대조했다.
- 실제 profile 임시파일 IO 실패/재시도와 genuine 구writer 읽기, 공식/확장177방 각3표본·요청 heap 회수6/6, exact/one-short/cancel/retry를 재사용했다. plain 미계측 값은0이 아니며, 초기방 생성/full catalog와 모든 방 방문·최종SLO는 다른 범위다.
- SOURCE/BEFORE archive의 실제 멤버·hash와82개 정역 연산을 메모리상에서 독립 대조했다(source695/baseline650). GEX-10의 격리 filesystem 복원 PASS와 구별하며 공유 tree에 patch를 적용하지 않았다.
- 감사 재집계기의 첫 실행은 ENTRY가 첫 줄 중간에 오는 RON 형태를 가정하지 못해 KeyError로 종료했다. 정규식 범위만 교정한 뒤89개 방·139연결과 나머지 assertion이 통과했다. 제품·기존 시험·기대값 교정은 없었다.

## 조건·선행·품질 판정

[CONDITION_REVIEW](audit/u13_independent_001/CONDITION_REVIEW.json)는 원문30수용 조건 각각에 source review·현재 증거·이번 판정을 연결하고10통합 seam/10검증 차원/5실행 node를 추적한다. 구현자의 COVERAGE satisfied를 그대로 독립 PASS로 인수하지 않았다. F-U13-001은 A19/A24, F-U13-002는 A09/A10을 직접 실패시키고 A08/A29 작성 검증 및 관련 seam의 완료를 막는다. 다른 항목의 `supported`는 기록된 범위의 source/증거 판단이며 전체 Goal 완료가 아니다.

상태 소유는 validated catalog, 현재 run map, 비현재 방문방 store, permanent profile로 나뉘며 공개 DTO가 권위 상태를 대체하지 않는다. Gate movement의 fallible 준비/transaction과 완료 지식 commit 순서, 하강만 회복, 재방문 비용/배정 보존, run checkpoint 밖의 영구 지식과 V14/V5 저장·구writer12/4 수용을 검토했다. 고정 graph 경로는 이전 자동 boss/omen replacement를 건너뛰고 미지원 relic executor를 거절한다. 새로운 가짜 executor·silent missing-reference 제거·테스트 전용 제품 분기는 검토 범위에서 발견하지 못했다. 단, 공개 투영의 중복 필터 불일치와 검증/실행 역할 불일치는 위 finding이다.

선행 U12의 현재 범위 재판단 Complete/High는 기존 원문으로 인수했다. U12/U04/U11의 전체 과거 audit hash가 U13에도 동일하다고 주장하지 않는다. [BASELINE_REUSE](BASELINE_REUSE.json)에 지정된 unchanged paired CAS/journal/authority 부분만 재사용하고, 변경된 room/clone/codec/actual consumer는 현재119선택·host30단계·현 calibration/새 host criterion으로 대체된 범위를 확인했다. 새 consumer의 이전 criterion 거절, transient 재시작 실패 및 버전 gate12-only 실패는 후속 성공과 별개로 보존되어 있다.

## 제한과 다음 경계

공식 첫 유물 및5종 executor(U16), 전조/조건 producer(U20), 개별 최종보스 고유 실행, Unity/배포, 공식 전체 플레이, FacilityLayout mod add/replace normal-consumer witness, 최종 성능SLO는 기존 제외 범위다. private issuer 비밀은 보관 대상이 아니며 공개 승인/실제 소비 증거만 인수한다. 이번 반례는 해당 미구현 기능을 요구하지 않는 trusted-base 작성/조회/배정 경로다. 외부 임의 저장·checksum·내부 상태 변조만 가능한 사례는 finding이나 추가 완료 차단으로 만들지 않았다.

감사는 위2건의 `Not Complete`로 종료한다. 제품 교정은 별도 GOAL-CFMN-01 GEX-10에서 처리하고, 변경 영향 검증과 successor target 동결 뒤 GEX-20 재감사가 필요하다. 기존 target/실패/감사 원문은 보존한다. single-phase이므로 별도 GEX-30은 없고, U14 이후는 시작하지 않았다.
