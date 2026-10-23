# U13 최종 독립 재감사 — Complete / High

GS-CFMN-001 / GOAL-CFMN-01 / PHASE-CFMN-01의 **`u13_current_002`에 대한 GEX-20 판정은 Complete / High, 열린 완료 차단 finding 0건**이다. F-U13-001/002의 교정을 독립 확인했고, 원문30수용 조건·10통합 seam·10검증 차원·5개 하위 실행 node를 검토했다. single-phase이므로 별도 GEX-30은 없다. U14는 시작하지 않았다.

## 대상·권한·독립성

- 사용자 요청은 실행 지시서를 읽고 U13 최종 독립 감사에 착수하는 GEX-20 `stage-run`이다. 과거의 교정까지만 수행하라는 인계 이후 별도로 승인된 감사이며, 제품 교정이나 후속 구현을 포함하지 않는다.
- [동결 대상](target/u13_current_002/AUDIT_TARGET_RECORD.json): HEAD `2f725172edbffd27c5078d29000a1a2ba6ddc4d9`, SOURCE SHA-256 `b60142fe59f70e58fa0c345690dfa6970a9d6319224f5545e4b0f9bfcb82317b`.
- 기준은 HA-CFMN-001 / FPF-CFMN-001 / PP-CFMN-001, [정식 Goal](../../P2G_50_GOAL_SET.ko.md), GEX-20/§6.4/§6.7, 연속 지시서 §4·7·8 및 로드맵 §1.9다. P2G 작성 당시의 미활성 표기는 현재 감사 권한을 대체하지 않는다.
- 감사자는 현재 대화에서 U13 구현·설계 선택·교정에 참여하지 않은 별도 실행자다. 제품 수정 없이 직접 검토·관측했으며 서브에이전트나 위임은 없다. 확인하지 않은 실행 모델 정체성을 주장하지 않는다.
- [입구](audit/u13_independent_002/CURRENTNESS_ENTRY.json)와 [출구](audit/u13_independent_002/CURRENTNESS_EXIT.json)에서 source701/evidence169 및 재사용 predecessor evidence1082의 바이트·크기, 제품 파일 목록과 HEAD를 대조했다. 대상은 `current / unchanged`다. PLAN·EXPERIMENTS·기존 증거·제품 코드를 수정하지 않았다. 감사 및 명시적으로 제외된 현황 locator만 갱신했다.
- 이전 [Not Complete 감사 원문](audit/u13_independent_002/PREDECESSOR_COMPLETION_AUDIT.md), target001, finding·반례·실패·교정 기록은 보존한다. target002의 `audit_performed:false`는 동결 당시 GEX-10 상태이며 이 감사에서 덮어쓰지 않는다.

## 교정 finding 판정

**F-U13-001 닫음.** 미완료 방의 예약 조우 비공개 조건을 Run.map, RequestMapData, 시설 내부 지도, 노드 완료 응답과 선택/취소·후퇴/재선택 투영이 공통으로 소비한다. 선택은 원본 session을 먼저 보존하고 응답 복사본만 가린다. 전투 Presentation과 실제 BattleStart resolver가 분리되어 실행할 조우는 소실되지 않는다. NodeConfirm snapshot의 session과 캐시된 preview도 같은 조건으로 가려진다.

기존 반례와 같은 정상 작성→공식 packaging/loader→공개 시작/조회 경로에서 `suppress_little_red`가 세 지도 모두 정확한 `Encounter { encounter_id: null }`로 숨겨졌다. key 누락을 null과 혼동하는 검사는 사용하지 않았다. 현재 교정 회귀의 실제 지원 활동 경로는 선택/취소·노드 완료, 완료 전 Continue, 실제 조우 진입, Gate 이탈·재방문·완료 후 Continue의 공개 전환을 검증한다. 조회/취소의 저장·공개 roots 불변 assertion과 원본 실행 identity도 검토했다.

**F-U13-002 닫음.** 정상 로더가 배정 역할과 실제 category/payload, 일반전·정예 소비 의미 및 구체 조우 등급을 대조한다. 기존 `normal_activity_id=maintenance` 반례를 역할 경계에서 거절한다. 독립 추가 사례에서 일반전 역할의 비일반 조우, 정예 역할의 일반 조우도 거절하고, 올바른 일반 조우 참조와 지원 활동 ID 별칭은 허용한다. 허용 입력의 공개 초기 방은 지원≤2·일반전≥2를 만족한다. 기존 support 선추첨·가중치·최소수·좌표 독립성·공식 콘텐츠는 변경되지 않았다.

두 finding은 계속 `범위 안 — 지원 경로`로 분류하며 실제 교정과 검증으로 닫았다. 외부 임의 저장/checksum/메모리 변조 사례로 재분류하여 면제하지 않았다. [finding 기록](audit/u13_independent_002/FINDINGS.json), [소스 검토](audit/u13_independent_002/SOURCE_REVIEW.json), [조건별 판정](audit/u13_independent_002/CONDITION_REVIEW.json)을 참조한다.

## 새 실행과 재사용 증거

이번 감사에서 다음을 실행했다. 빌드 성공을 기능 성공으로 집계하지 않는다.

- **독립 공개 사례7/7:** 비공개 조우2사례, 잘못된 역할/조우 등급 거절3사례, 올바른 구체 참조/지원 별칭2사례. 거절 사유가 해당 allocation role인지 확인했고, 허용 사례는 실제 starter 선택과 공개 지도 조회까지 진행했다. [계획](audit/u13_independent_002/PROBE_PLAN.md), [명령·binary·종료·입출력 결속](audit/u13_independent_002/PROBE_RECEIPT.json), [관측](audit/u13_independent_002/observations/RESULT.json).
- **현재 코드로 재빌드한 기존 공개 소비자:** Gate 왕복, 이동 prepare 폐기, 자식 프로세스 Continue의 전체 roots 일치, 하강, SavePoint 복원 시 영구 지식 보존, Gate Return과 active checkpoint 제거가 통과했다. 기존 frozen input002와 소비자 소스는 수정하지 않았다. [빌드](audit/u13_independent_002/CONSUMER_BUILD_RECEIPT.json), [실행·복사 검증](audit/u13_independent_002/CONSUMER_RECEIPT.json), [원시 결과](audit/u13_independent_002/consumer/route/RESULT.json).
- **현재 코드 비용6표본:** 공식89방과 확장177방을 각각 새 프로세스에서3회 측정했다. 모든 측정 단계 성공, 종료 후 requested heap의 기준값 회수6/6. 전체 시설 조회 최대는 공식1.976ms/확장2.532ms였다. [원시 측정·요약](audit/u13_independent_002/consumer/RESULT.json). 기존 결과와 같은 유한 workload의 관측이며 성능 개선이나 최종 SLO를 주장하지 않는다. 실제 시간/native 작업량/requested heap/OS RSS를 구분한다.
- 채택 JSON과 공식 RON의 **89방·846활동·139연결 차이0**, predecessor raw 승인 호스트30단계의 exit/result/admission 각각30통과, 이전 실제 전투의 브리핑/setup10필드 일치·3배치·202tick·승리/도감 등록을 재집계했다. [원시 증거 대조](audit/u13_independent_002/RAW_REVIEW.json).

교정자가 실행한 **Core121/121, 실패0·무시0**은 최종 소스/시험과 원시 출력·종료 상태를 대조해 재사용했다. 이전119개 시험 이름이 현재121개에 전부 포함됨을 확인했다. 새 공개 관측은 기존 교정자의 공개3사례 통과 주장만을 반복 인용한 것이 아니다. [교정/복원 증거 검토](audit/u13_independent_002/CORRECTION_REVIEW.json).

기존 genuine V12 writer/profileV4 읽기·현재 V5 쓰기, 실제 profile 임시파일 IO 실패와 재시도, 승인 호스트30단계 및 U12/U04/U11 paired CAS/journal/권한 증거는 변경되지 않은 메커니즘에 한정해 재사용했다. [명시적 선행 참조17개 일치](audit/u13_independent_002/BASELINE_REFERENCE_REVIEW.json)와 successor15파일 delta를 확인했다. 기존 Normal 승인·calibration은 predecessor 실행 파일에 귀속된다. **이번 현재 소스의 진단 소비자는 새 Normal 승인 호스트가 아니다.** 새 worker 승인이나 전체 저장소 시험 통과를 주장하지 않는다.

이전 test014의10통과/2실패와 교정 중 컴파일/시험 실패는 보존했다. 당시 V12 current-writer 기대값 및 Standard 표기와 현재 V14/Expedition의 차이는 실제 새 프로세스 경로·genuine 구writer 증거와 분리하여 판정했다. 실패 batch 전체를 PASS로 바꾸지 않았다.

## 상태·품질·복원·종료

validated catalog, 현재 RunState.map, 비현재 방문방 map/progression, 영구 profile의 소유 경계를 확인했다. Gate 이동은 실제 연결과 완료 경로를 검사하고 fallible 준비/회복 뒤 영구 완료를 확정한다. 재방문은 같은 배정/UUID/입장 receipt를 보존하며 하강만 회복한다. Continue/SavePoint는 원정 상태를 복원하고 profile 지식은 단조 병합한다. 고정 graph가 과거 자동 boss/omen 교체를 우회하고 미지원 executor를 명시 거절하는 경계도 유지된다. 검토한 변경에서 시험 전용 제품 분기, 누락 참조 묵살, 가짜 executor, 중복 상태 권위나 정책 변경은 발견하지 못했다.

target001 원본 archive와 target002 SOURCE의 실제 멤버/manifest 바이트를 읽어 successor15개 정역 연산을 독립 대조했다. source701↔695의 전체 바이트와 목록이 일치하며 기존 GEX-10의 격리 filesystem 복원 증거도 유효하다. 이번 추가 대조는 메모리상 재검산이며 작업 트리에 패치를 적용하지 않았다. 선행 target001의82연산/초기650개 복원 증거도 확인했다.

필수 미검증이나 열린 완료 차단 항목은 없다. U16의 다섯 유물 활동 실행, U20 전조/조건 producer, 고유 최종보스, 공식 전체 플레이, FacilityLayout mod add/replace witness, Unity/배포 및 최종 성능 SLO는 기존 제외 범위다. 공식 초기 유물 때문에 전체 공식 진행이 아직 불가능하다는 지원 상태를 유지한다.

재현은 각 receipt의 빌드·실행 명령과 보존 binary를 사용하고 출력은 새 빈 경로로 지정한다. 기존 소비자 입력은 동결된 `test_parts/public_inputs002`다. 명령은 [공개 probe](audit/u13_independent_002/PROBE_RECEIPT.json), [기존 소비자](audit/u13_independent_002/CONSUMER_RECEIPT.json)에 있다. 최종 결속은 [AUDIT_CLOSURE](audit/u13_independent_002/AUDIT_CLOSURE.json), 이후 실행 위치와 권한은 [최신 인수인계](../../../../roadmaps/core_first/execution/HANDOFF_LATEST.ko.md)가 소유한다. 이번 요청은 U13 감사 완료로 종료한다.
