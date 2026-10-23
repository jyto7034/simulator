# Core 우선 실행 이력 안내

이 문서는 과거 요청·인계·규칙의 보관 위치를 찾는 색인이다. 기본 재개 자료는 [현재 상태](HANDOFF_LATEST.ko.md)와 [상시 규칙](CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md)이다. 과거 원문의 “현재 중지”, “계속”, “U00부터 시작”은 해당 시점의 기록이며 새 실행 지시가 아니다.

| 확인할 근거 | 보관 원문 | 읽는 경우 |
| --- | --- | --- |
| 최초 연속 실행 위임과 2026-10-04/05 후속 발화 | [CONTINUOUS_REQUEST_20261004](CONTINUOUS_REQUEST_20261004.ko.md) | 현재 권한·정책의 출처에 구체적인 의문이 있을 때 해당 절만 확인 |
| 읽기 경로 정리 직전의 진행 중 실행·다음 행동 | [HANDOFF_20261013_002](HANDOFF_20261013_002.ko.md) | 2026-10-13 실행 담당자가 갱신한 인계 원문과 문서 정리 차이를 확인할 때 |
| 2026-10-12 재개 입구의 인계·제품/시험 잔여 | [HANDOFF_20261013_001](HANDOFF_20261013_001.ko.md) | 이번 현재 상태 갱신 이전의 원문 확인. 보관일은 10월 13일이며 본문 작성 시점은 10월 12일 |
| 그 이전 시험 담당 인수 전 인계 | [HANDOFF_20261012_007](HANDOFF_20261012_007.ko.md) | 이전 담당자·변경 귀속을 확인할 때 |
| 상시 지시서 축약 전 규칙 | [개정 전 원문](history/20261013_rules_before/CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md) | 규칙 변경 대조가 필요할 때. 현재 실행 규칙으로 중복 적용하지 않음 |

날짜별 HANDOFF·실행 기록·감사·동결 target은 각 원본 위치에 보존한다. 이 색인이 전체 이력을 다시 복제하거나 모든 기록을 필수 읽기로 지정하지 않는다. 현재 상태 갱신 전 최신본을 충돌하지 않는 날짜별 HANDOFF에 보존하고 필요한 링크만 추가한다.

## 2026-10-13 읽기 경로 정리

상시 지시서에서 중복 규칙을 통합하고 문서별 읽기 조건·용어를 명시했다. HANDOFF_LATEST는 최신 기록에 근거한 현재 상태의 진입점, PLAN은 상세 작업·역할, EXPERIMENTS는 실제 실행 결과의 원문 연결을 맡는다. 로드맵·ROADMAP_STATUS는 현재 상태를 가리키며 별도 STATE 장부를 만들지 않는다.

과거 요청 원문·동결 정책/감사·제품 코드는 이 정리로 변경하지 않는다. Astra 테스트 직접 작성·교정, Sol 제품 구현·실행, 별도 Astra 독립 감사와 기존 수용·실패 기준을 유지한다. 실행 기록은 진행 중 갱신될 수 있으므로 HANDOFF의 확인 범위와 이후 결과를 구분한다.

## 2026-10-13 실제 연속 실행 재개

사용자 “마저 진행해줄래?” 요청으로 기존 승인 작업을 계속한다. 재개 전 최신본은 [HANDOFF_20261013_003](HANDOFF_20261013_003.ko.md)에 원문 보존했다. 새 현재 상태는 release006의 실제 마지막 관측과 세션 회수 결과를 문서 갱신 시각과 구분하며, 후속 실행 원문은 Goal EXPERIMENTS가 소유한다.


## 2026-10-13 복구 완료와 공개 Tutorial 후속

복구 빌드 성공·Tutorial003의 실제 앞3전투 통과/넷째 실패·정상 입력 준비 완료를 최신 인수인계에 반영했다. 직전05:40UTC 최신본은 [HANDOFF_20261013_004](HANDOFF_20261013_004.ko.md)에 원문 보존했다. 실행 종료 시각·원문은 Goal EXPERIMENTS와 각 RESULT가 소유한다.


## 2026-10-13 전술 진행·서버 시험 등록과 release loader 교정

전술 진행기/Return module 등록의 검토 완료, server_compile001의 실제 실패와 공개 loader 교정 동결을 최신 인수인계에 반영했다. 직전06:13UTC 최신본은 [HANDOFF_20261013_005](HANDOFF_20261013_005.ko.md)에 원문 보존했다. 동일 Sol capacity 재개와 실제 후속 실행은 Goal EXPERIMENTS/RESULT가 소유한다.


## 2026-10-13 실제 보스 격파 관측과 일반 전술 fallback

서버/probe 빌드 성공, Tutorial004의 Porcubbus 통과·primary boss 격파와 남은 적 미종료, 일반 접점/Swap 진행기 교정 동결을 최신 인수인계에 반영했다. 직전06:55UTC 최신본은 [HANDOFF_20261013_006](HANDOFF_20261013_006.ko.md)에 보존했다. G11 설계와 worker/calibration 후속 경계는 현재 필수 작업으로 유지한다.

- [HANDOFF_20261013_007](HANDOFF_20261013_007.ko.md): 07:34 UTC 최신본 보존. 뒤이어 Tutorial005/Expedition002 완주·server003/actor exact1 종료와 G11 작성 ownership을 최신본에 반영했다.

- [HANDOFF_20261013_008](HANDOFF_20261013_008.ko.md): 08:01 UTC 최신본 보존. G11 source/fixture 작성·메인 수용과 offline lock 실패/고정 버전 재사용 복구를 최신본에 반영했다.

## 2026-10-13 Astra 단독 수행 규칙 적용

사용자가 저장소의 갱신된 지시서를 앞으로 적용하도록 지정했다. 기존 에이전트는 writer0/runner0으로 종료했고 현재 상태는 최신 인수인계에 반영했다. 직전08:35UTC 최신본은 [HANDOFF_20261013_009.ko](HANDOFF_20261013_009.ko.md)에 보존했다. 기존 역할 분리 기록은 당시 이력이며 현재 실행 규칙은 아니다. G11 compile002 성공·exact1 통과·exact2 실패 및 provenance 교정 미검증 상태를 유지한다.

- [HANDOFF_20261013_010](HANDOFF_20261013_010.ko.md):09:06UTC 최신본 보존. 이후 단독 G11 Core/heap 검증, 공개 실제 패배→명시 포기→새 원정 검증과 최신 지시서 확인을 최신본에 반영했다. normal admission/영속 복원/full60/최종화는 미완료다.

- 2026-10-13 11:03 UTC: 실제 RunFailed·normal admission/귀환·장애복구 실행 후 위치 갱신. 직전 최신본을 [HANDOFF_20261013_011](HANDOFF_20261013_011.ko.md)에 원문 보존했다. ACK 양성 대조 교정의 server_compile005가 실행 중이며 나머지 잔여는 최신 인계에 있다.

- 2026-10-13 11:23 UTC: 정상 local/server·ACK교정·full60 측정/채택/허용·거절을 마치고 수용 coverage 추가검증으로 이동. 이전 최신본 [HANDOFF_20261013_012](HANDOFF_20261013_012.ko.md) 보존.

- [HANDOFF_20261013_013](HANDOFF_20261013_013.ko.md): 11:23 UTC의 추가80검사 실행 중 인계를 보존. 후속13PASS와 final identity/승인 결속 단계는 최신본 참조.

- 2026-10-13 12:42 UTC: 최종 일반/server/full60 소비·fresh Continue,38항목 coverage와 target/u12_current_002 정역복원/동결 완료. 이전12:03 UTC 최신본은 [HANDOFF_20261013_014](HANDOFF_20261013_014.ko.md)에 보존했다. 비참여 독립 감사는 미실행이다. 사용자의12:37 UTC 지시로 U12까지만 수행하며 완료 후에도 후속 U13에 자동 착수하지 않는다.

- 2026-10-13 12:45 UTC: 지시서 수행 요청으로 재개 근거·남은 감사 경계를 확인했다. 동결 MANIFEST 및 7개 참조 기록의 결속, 마지막 6개 RESULT의 exit0/source불변을 확인했으며 새 기능 실행이나 독립 판정은 없다. 비참여 감사 미충족과 U12 종료 경계를 유지한다. 이전12:42 UTC 최신본은 [HANDOFF_20261013_015](HANDOFF_20261013_015.ko.md)에 byte 동일 보존했다.


## 2026-10-13 13:13 UTC U12 독립 감사 음성 종료

사용자의 직접 감사 요청에 따라 고정 u12_current_002를 GEX20으로 검토했다. [이전 최신본](HANDOFF_20261013_016.ko.md)을 원문 보존했고 [독립 감사](../../../goals/core_expedition_lifecycle/post_audit/u12_current_002_pass001/COMPLETION_AUDIT.md)는 Not Complete/F-U12-001을 기록했다. 정상 backup이 미지원 profile 필드를 가려 Continue를 허용하고 후속 명령이 원본을 덮어쓰는 결함을 감사 전용 복사본에서 실제 재현했다. target 입출구 무결성은 통과했고 제품 교정/U13 착수는 하지 않았다. 현재 필수 경계는 U12 GEX10 교정과 successor 재감사다.


## 2026-10-13 13:50 UTC U12 현재 범위 재판단 완료

사용자의 재판단 요청에 따라 F-U12-001을 `범위 밖 — 외부 임의 변조`로 재분류하고 이전 감사의 미종결 필수 검토를 완료했다. [현재 판정](../../../goals/core_expedition_lifecycle/post_audit/u12_scope_reassessment_001/COMPLETION_AUDIT.md)은 **Complete / High / 열린 완료 차단0**이다. 이전 [Not Complete 인계](HANDOFF_20261013_017.ko.md)와 감사·재현 원본은 보존했다. 제품/시험 수정·전체 재동결·시험 재실행 없이 유효한 원시 증거와 source 판단을 재사용·보완했으며, 과거 전체 감사를 새 기준으로 재실행한 것으로 기록하지 않는다. 이전 F-U12-001 교정 반환 요구는 현재 효력이 없다. U12 완료 후 중단하고 U13에는 착수하지 않았다.


## 2026-10-13 U13 착수·정책 질문

사용자의 U13 착수 요청으로 기존 U12 종료 경계를 U13 범위에서 해제했다. [직전 U12 완료 인계](HANDOFF_20261013_018.ko.md)를 byte 동일 보존했다. [U13 Working Policy](../../../goals/core_facility_map_nodes/WORKING_POLICY.ko.md)에서 P2G-00을 고정하고 후보 위치 추첨·보상5종 비중의 명시적 미결을 P2G-10 질문으로 남겼다. 제품/시험 변경·실행 없이 답변 대기하며, 추천안을 확정 정책으로 사용하지 않는다.


- 2026-10-13 U13 정책 답변·재개: [질문 대기 원본](HANDOFF_20261013_019.ko.md)을 보존하고 단순 균등 위치 선택·보상5종 각20%를 확정했다. WR004/P2G10 완료 후 코드 검토로 이어간다.

- 2026-10-14: U13 P2G40/50 검증·P2G60/GEX00 인수 후 GEX10 구현 진행. 이전 인계 [020](HANDOFF_20261014_020.ko.md) 보존; 미완료·기능검증 전.

- 2026-10-14: U13 GEX-10 구현·자체검증 종료. source695/evidence1082/delta82의 [target u13_current_001](../../../goals/core_facility_map_nodes/execution/single_goal/target/u13_current_001/AUDIT_TARGET_RECORD.json), 격리 정역복원 PASS. Core119/119·서버1/1·승인 호스트30단계·메모리 회수6/6. 원 실패·공식 첫 유물 U16 미지원·보조 기대값2실패를 보존하며 감사/Goal Complete를 주장하지 않는다. 기존 인계021 보존. 새 감사 요청 전 GEX20/30·U14 미착수. SOURCE SHA256 `b941dfed5114b62337b2db060ee0601baaf736b3c48da33ef70bf85fd73f9a10`.


- 2026-10-14 U13 독립 완료 감사: 사용자 직접 요청으로 `u13_current_001`의 GEX-20을 수행해 **Not Complete / F-U13-001·002 / 열린2건**으로 종료했다. 정상 작성/public 경로에서 미탐사 고정조우ID 노출과 잘못된 일반전 역할 수용(정비6/일반전0)을 재현했다. 제품·기존 동결증거 수정 없음. [감사](../../../goals/core_facility_map_nodes/execution/single_goal/COMPLETION_AUDIT.md), [직전 감사대기 원문](HANDOFF_20261014_022.ko.md) 보존. 다음은 GOAL-CFMN-01 GEX-10 교정과 successor 재감사이며U14 미착수.


## 2026-10-14 U13 교정만 수행 후 종료

사용자 최신 지시 “일단 교정만 해줘 감사는 하지말고”에 따라 F-U13-001/002를 교정하고 관련 Core121/121·공개 반례3/3을 검증했다. successor u13_current_002와 정역복원을 준비했으며 재감사/U14는 미실행이다. 직전 감사 인계는 [023](HANDOFF_20261014_023.ko.md), 현재 위치는 [최신 인수인계](HANDOFF_LATEST.ko.md)를 따른다. 과거 Not Complete를 새 감사 완료로 변경하지 않는다.

## 2026-10-14 U13 최종 독립 재감사 완료

사용자의 새 U13 최종 독립 감사 요청으로 target002의 GEX-20을 수행했다. [직전 교정까지만 수행한 인계](HANDOFF_20261014_024.ko.md)를 원문 보존했다. [현재 감사](../../../goals/core_facility_map_nodes/execution/single_goal/COMPLETION_AUDIT.md)는 Complete / High / 열린 완료 차단0이며 두 finding을 닫았다. 공개7사례·현재 소비자 이동/새프로세스 복원·비용6표본을 새로 실행하고 교정121시험 및 비영향 선행 증거를 대조했다. 제품·고정 증거는 출구까지 동일하며 이전 Not Complete는 [../../../goals/core_facility_map_nodes/execution/single_goal/audit/u13_independent_002/PREDECESSOR_COMPLETION_AUDIT.md](../../../goals/core_facility_map_nodes/execution/single_goal/audit/u13_independent_002/PREDECESSOR_COMPLETION_AUDIT.md)에 보존했다. U14는 시작하지 않았다.


## 2026-10-14 U14 착수와 정책 의논

사용자가 상시 지시서 확인 후 U14 착수를 요청했다. 직전 U13 Complete / High 인계는 [HANDOFF_20261014_025](HANDOFF_20261014_025.ko.md)에 원문 보존했다. U14 규칙·코드 인수 중 부활 단계/무억제 강화 결합 방식의 미정을 확인하여 P2G §8.6에 따라 질문 대기한다. 제품 변경·빌드·시험은 아직 없으며, 재개 지점과 질문은 최신 인수인계를 따른다.


## 2026-10-14 U14 정책 답변과 구현 인수

사용자 “1번”으로 부활/무억제 배율 곱셈을 확정하고 U14를 재개했다. 직전 대기 인계는 [026](HANDOFF_20261014_026.ko.md)에 보존했다. P2G 구조검증과 GEX00 baseline 인수 후 실제 제품 구현은 최신 PLAN/EXPERIMENTS를 따른다.


## 2026-10-14 U14 구현·자체검증 종료

입장비용·억제력·Breakdown·고정난이도·전투보상·황금가지와 복원을 구현했다. Core82/82, G11기본4/계측4, 서버29/29, 승인호스트30+10단계, 실제12승 성장, 비용6표본을 확인했다. 기존138실패는 변경전 재현으로 분리했고 전체 green은 주장하지 않는다. [u14_current_001](../../../goals/core_expedition_deterrence_rewards/execution/single_goal/target/u14_current_001/AUDIT_TARGET_RECORD.json) source704/evidence980/delta117과 격리 정역복원 PASS. [직전 인계027](HANDOFF_20261014_027.ko.md) 보존. GEX20/Goal Complete/U15 미실행, 별도 감사 요청 대기.


## 2026-10-14 U14 독립 감사 Not Complete

사용자의 독립 감사 요청으로 GEX-20 stage-run을 완료했다. [직전 구현 인계028](HANDOFF_20261014_028.ko.md)를 보존했다. [감사 보고서 원문](../../../goals/core_expedition_deterrence_rewards/execution/single_goal/audit/u14_independent_002/PREDECESSOR_COMPLETION_AUDIT.md)는 Not Complete / 열린 finding1건(F-U14-001, 다른 방문 방의 봉인·배율 preview 오류)이다. 정상 제작/public 방왕복 반례를 새로 실행하고 기존 시험·host·성장·비용 원문을 재사용/독립 검산했다. 출구 source704/evidence980 동일, 제품 변경·교정·U15 없음. 다음은 별도 요청의 U14 GEX-10 교정과 독립 재감사다.

## 2026-10-14 U14 교정·자체검증·재감사 인계

사용자 상시 지시서 적용/U14 교정 요청에 따라 F-U14-001과 정상 보충 저장에서 추가 발견한 C-U14-002를 교정했다. Core99/G11기본4·계측4/public3·fresh-process6/계량6 통과, target002 source713/evidence600/delta21 정역복원 완료. 직전 Not Complete 인계 원문은 [HANDOFF_20261014_029](HANDOFF_20261014_029.ko.md), 현재 결과·재감사 대상은 [최신 인계](HANDOFF_LATEST.ko.md)를 따른다. 기존 감사는 보존했으며 비참여 재감사·Goal Complete·U15는 미실행이다.

## 2026-10-14 U14 독립 재감사 완료

사용자의 독립 재감사 요청으로 target002의 GEX-20 stage-run을 완료했다. [직전 교정 인계030](HANDOFF_20261014_030.ko.md)을 원문 보존했다. 판정은 **Complete / High / 열린 완료 차단0**이며 F-U14-001·C-U14-002를 닫았다. 새 공개3사례와 수집 전/후 실제SavePoint2사례를 통과하고31수용·10차원·10통합을 판정했다. 교정Core99·G11·서버/host·성장/비용의 유효 원시증거는 영향 범위에 맞춰 재사용/검산했다. source713/evidence600+980 입구·출구 동일, 제품수정0. [독립 감사](../../../goals/core_expedition_deterrence_rewards/execution/single_goal/COMPLETION_AUDIT.md)와 [closure](../../../goals/core_expedition_deterrence_rewards/execution/single_goal/audit/u14_independent_002/AUDIT_CLOSURE.json)가 현재 근거다. U15는 착수하지 않았다.

## 2026-10-14 기존 Core 실패 138개 진단

사용자 요청으로 현재138실패를 재현하고 구형 준비109·기대값28·환경1로 최초 원인을 분류했다. 격리65개 전체 assertion 통과 및 원본 대체후보12/12통과, 유지135·통합후중복제거1·정상유지/변조분기제거1·환경분리1 권고를 기록했다. 제품/시험 변경·삭제는 하지 않았다. [보고서](../../../testing/core_baseline_failure_triage_20261014_001/REPORT.ko.md), [직전 U14 감사 인계031](HANDOFF_20261014_031.ko.md). 제품 무결함 또는 원본138개 해결을 주장하지 않는다.

## 2026-10-14 기존 Core 실패 138개 교정 완료

사용자의 후속 교정 요청에 따라 원138을 현재139개 대응 사례로 교정·통합·분리하고 모두 통과했다. 기본 lib 고유2,097통과/기존ignore6, 계측선택23통과, 외부CLI12완주를 확인했다. 추가 통합은56통과/원138밖17실패이므로 전체 green은 아니다. 테스트/fixture35파일 delta와 원 실패·preimages를 보존했다. [교정 결과](../../../testing/core_baseline_failure_correction_20261014_001/REPORT.ko.md), [직전 진단 인계032](HANDOFF_20261014_032.ko.md). 제품 실행 로직·U14 동결 기록은 변경하지 않았고 U15는 미착수다.
