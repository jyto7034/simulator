# Simulator 개발 현황 — 단일 진입점

**2026-10-06 U08 R06 사전 검증 교정028:** 공개 RON의 혼합unit Sum/Max에 외부 능력치 참조를 더하면 validation이 성공하는 결함을 재현했다. [교정 인수](../core_first/execution/u08_public_20261006_028/RECORD.ko.md)에 따라 기존 거부 요건을 유지해 compiler를 교정하고 관측 공백을 보강한다. 통합006의 Core default/fixtures 각각1958PASS는 보존하며 나머지54역할은 미시작이다. U08 전체 완료·독립 감사·U09 진입은 아직 아니다. 아래027은 교정 전 인수 이력이다.

**2026-10-06 U08 공개 경로 보강·최종 통합 입력 인수027:** Normal 실제 피해180/Corrupt210 및 비정상 입력16건의 집중 검증을 같은 source438/380e7408에서 통과했다. [현재 인수](../core_first/execution/u08_public_20261006_027/RECORD.ko.md)는 실행 입력1322개와 외부archive2개/8members를 고정하여 Sol의 통합006을 승인한다. 현재 환경은 fullaccess이며 추가 승인 인자를 사용하지 않는다. 전체 기능·성능·최종화·독립 감사와 U09 진입은 아직 미완료다. 아래 상태는 각 당시 이력이다.

**2026-10-06 U08 승인 실행 가능 확인·통합 검증 재개:** 사용자 권한 변경 후 승인 실행에서 Unix/로컬TCP 소켓 생성·bind가 모두 통과했다. [재개 인수026](../core_first/execution/u08_public_20261006_026/RECORD.ko.md)에 따라 동일 source438/8734111c·harness122입력에서 Sol의 실제 결속·통합/후속 검증을 재개한다. 전체 통합·성능·최종화·독립감사는 아직 미완료다. [최신 인수인계](../core_first/execution/HANDOFF_LATEST.ko.md)를 따르며 아래 EPERM 안내는 권한 변경 전 이력이다.

**2026-10-06 U08 권한 변경 전 환경 장애 인계(이력):** 통합004는 Core default 1,872 PASS/12 FAIL로 종료했고, catalog·capability 회귀 11건을 교정하여 최종 source438/8734111c에서 각1/1과 포맷검사가 통과했다. 남은 소켓 실패는 AF_UNIX bind·AF_INET 생성 EPERM으로 확인됐다. 현재 권한에서 전체 통합·성능 검증을 완료할 수 없어 [최신 인수인계](../core_first/execution/HANDOFF_LATEST.ko.md)와 [교정·현재 source·재개 기록](../core_first/execution/u08_public_20261006_025/RECORD.ko.md)에 보존했다. 실행자/입력 writer는0이며 사용자 중지·완료 판정은 아니다. 기존 연속 권한은 유지한다. U08 자체검증·동결·독립감사 및 U09 진입은 미완료다. 아래 착수·완료 안내는 각 당시 이력이다.

**2026-10-06 U08 계획 완료·GEX 인수 착수 당시 안내(이력):** [유물 실행 PLAN](../../goals/core_relic_numeric_variants/execution/single_goal/PLAN.md)의 P2G-00→60/quality2 검증이 통과했다. 16수용군·10차원·121파일 코드근거와 정의/사본/수치/저장의 동시 전환을 인계한다. 기존 연속 권한으로 지정 Sol/xHigh가 구현·검증을 맡고, 동결 뒤 비참여 Astra/High가 U08 전체를 독립 감사한다. 현재 제품 구현·독립 완료는 아직 아니며 U09는 U08 독립 완료 뒤 진행한다.

**2026-10-06 U07-M 완료 공표 당시 안내(이력):** [전체 독립 감사](../../goals/core_runtime_measurement_profiles/post_audit/u07m_20261006_001/COMPLETION_AUDIT.md)가 **Complete / High / 열린 finding0**으로 종료됐다. 기본8MiB를 유지한 trusted 개발16/32MiB 프로파일과 bounded 실제 Core/host 측정·재현을 완료했다. [완료 인수·검증·남은 경고](../core_first/execution/u07m_closure_20261006_001/RESULT.ko.md)가 현재 완료를 소유한다. 최종 배포 수치는 U30-C/U30-P에 남기며, 기존 연속 권한으로 U08의 현재성 인수·P2G를 진행한다. 아래 U07-M 착수 안내는 당시 이력이다.

**2026-10-05 U07 완료·U07-M 착수 당시 안내(이력):** [전체 독립 재감사](../../goals/core_independent_objects/post_audit/u07_20261005_002/COMPLETION_AUDIT.md)가 **Complete / High / 열린 finding0**으로 종료됐다. 객체 진화·제거의 저지·벽 접촉 보존 F01을 닫았고 `target/u07_002`가 출구까지 유지됐다. [완료 인수·검증·남은 경고](../core_first/execution/u07_closure_20261005_001/RESULT.ko.md)를 따른다. U07-M은 P2G-00→60/quality2를 완료하고 [새 실행 PLAN](../../goals/core_runtime_measurement_profiles/execution/single_goal/PLAN.md)에 따라 GEX 실행 인수·구현에 착수했다. root 계획·조정, Sol/xHigh 구현·검증, 비참여 Astra/High 독립 감사 역할을 유지하고 U08은 U07-M 완료 뒤 진행한다. 아래 중지·재개 안내는 당시 이력이다.

**2026-10-05 후속 작업 예약 당시 안내(이력) — U07-M:** U07의 진행 중 구현·검증은 현재 PLAN이 소유한다. U07 독립 완료 뒤 U08 전에 [실측 기반 실행 예산 조정 준비](../core_first/execution/u07_memory_budget_followup_20261005_001/FOLLOWUP.ko.md)를 수행하도록 등록했다. 현재 U07 기준과 8MiB 실패 판정은 유지한다. 측정/설정 준비는 U07-M, Core 대표 최대 부하·수치 후보는 U30-C, 실제 Unity 플레이·배포값 재검토는 U30-P가 소유한다. 준비 완료를 전체 플레이 집계나 최종 값 확정으로 표시하지 않는다. 다음 안전한 pass 경계에서 [발행 기록](../core_first/execution/u07_memory_budget_followup_20261005_001/RECORD.ko.md)을 인수한다.

**2026-10-05 재개 당시 안내(이력):** 후속 사용자 지시로 중지 상태를 해제했다. U07의 미검증 저장 교정과 남은 검증·최종화를 이어간다. [현재 PLAN과 재개 교정 구현서](../../goals/core_independent_objects/execution/single_goal/PLAN.md)가 최신 위치를 소유한다. 계획 조정과 GPT-6.1 Sol / xHigh 구현을 분리했으며, 최종 동결 후 비참여 Astra / High가 전체 U07을 독립 감사한다. U00–U06 완료를 인수하고 U08은 U07 감사 통과 전 진입하지 않는다. 아래 중지 안내·수치는 당시 이력이다.

**2026-10-05 사용자 인계 요청으로 중지 당시 상태(이력):** U00–U06 독립 완료. U07은 기능 통합·공개 검증 후 PC 8MiB 예산 회귀를 발견했고, 공통 출처 저장 교정 12파일을 적용한 채 중지했다. **마지막 교정은 포맷·컴파일·테스트 전이며 U07은 미완료다.** [최신 인수인계](../core_first/execution/HANDOFF_20261005_002.ko.md)와 [중지 manifest](../core_first/execution/u07_pause_20261005_001/PRODUCT_CURRENT.json)를 먼저 읽는다. 실행 중인 구현/검증·U07 감사는 없으며 U08은 미진입이다. 아래 통과 수치는 각 당시 source에 귀속된다.

**2026-10-05 U06 완료:** 독립 재감사 Complete / High, F01 해소·미해결 finding0. 현재 Core1794/1794/1803·host92/100+heap1과15개 새 실행·30개 명시적 재사용을 독립 판정했다. U07/B13·C01은 P2G-00→60·GEX-00을 완료하고 구현에 진입했다.

**2026-10-04 실행 지침 개정 인수:** 진행 중 구현 담당자는 다음 안전한 pass 경계에서
[변경점·적용 순서·동결 감사 영향](../core_first/execution/workflow_efficiency_update_20261004_001/RECORD.ko.md)을 확인한다.
P2G r8 / GEX gex-r8 / 운영안 v2.1.3은 네 단계와 완료 기준을 유지한다. 아래 진척 이력을 갱신하거나
진행 중 명령을 중단하라는 알림이 아니며, 기존 감사의 현재성은 변경 영향에 따라 별도로 판정한다.

**2026-10-04 문서 교정·경로 정리 / CF-D08·09:** 최신 게임 정책은 [game_rulebook.md](../../game_rulebook.md)와
`game_rules/` 상세 규칙·후속 사용자 합의다. [과거 룰북](../../archive/game_rulebook_legacy_2026-10-04.md)은 비교 자료이며 새 정책·구현 결과를 동기화하지 않는다.
오늘 U04/U05 문서 갱신의 혼입을 확인해 과거 본문과 권위 안내를 교정했다.
[변경 사실·보관본·검증](../core_first/execution/rulebook_authority_correction_20261004_001/RECORD.ko.md)을 후속 작업이 인수한다.
[이름 변경·동결 기록 경로 대응표](../core_first/execution/rulebook_path_migration_20261004_001/RECORD.ko.md)를 함께 인수한다.
제품 소스·기존 감사 원본은 변경하지 않았으며 아래 완료·진행 이력을 소급 변경하지 않는다.

2026-10-06 · v1.156 · 현재 개발 주작업: **U08 GEX-10 R06 사전 검증 교정**.
[재개 인수](../core_first/execution/u06_resume_20261005_001/INTAKE.ko.md)에서 [중지 당시 인수인계](../core_first/execution/HANDOFF_20261005.ko.md)를 이어받았으며 현재 위치는 아래 실행 PLAN이 소유한다.
[활성 로드맵](../core_first/ROADMAP.ko.md), [연속 실행 위임](../core_first/execution/CONTINUOUS_REQUEST_20261004.ko.md),
[U01 완료 인수](../core_first/execution/u01_closure_20261004_001/RESULT.ko.md),
[U02 Master 독립 감사](../../goals/core_runtime_safety_observability/post_audit/master_20261004_002/COMPLETION_AUDIT.md),
[U02 완료·검증·남은 경고](../core_first/execution/u02_closure_20261004_001/RESULT.ko.md)를 따른다.
U02 S1 설정·공개 host와 S2 bounded 관측은 각각 독립 Complete / High이며 Master GEX-30도 통과했다.
기존 감사와 실패 원본은 보존했다. U03의 전투 fault 지속 상태·재접속·저장 보호도 독립 Complete / High로 종료했다.
R13~R16 요구는 활성 Core 우선 순서의 해당 owner에 인수한다.

아래 R 기록은 당시 target에 귀속된 이력이다. R06·R07의 과거 Complete / High와 현재 통합 source 판정을 구분한다.

개발을 재개할 때 이 문서에서 시작한다. 다음 작업과 전체 진행표는 여기서 안내하고, 상세 설계·실행·감사 원본은 링크로 확인한다.
문서 정리나 후속 기능 논의를 했다는 이유로 개발 주작업을 바꾸지 않는다. 사용자 최신 지시가 우선한다.

## 현재 작업

U06은 [최종 독립 재감사](../../goals/core_battle_starter_tactics/post_audit/u06_20261005_002/COMPLETION_AUDIT.md)에서 **Complete / High**로 종료했다.
기본 순수 치유·정상6세트·전술/작업/저지·시간 선택/설정·공개 query와 복원을 구현하고 회복량0 자격 F01을 닫았다.
20 수락 조건·10차원·141 선행 조건·별도31 R06이 통과했다. 현재 최종 Core1794/1794/1803·host92/100+heap1,
공개8사례·strict작성11·원래감사재현을 포함한15개 새 실행과30개 명시적 재사용을 구분했다.
[완료 인수·검증·남은 경고](../core_first/execution/u06_closure_20261005_001/RESULT.ko.md)를 따른다.
R05는 기존 peak 단독 경고(+605,609bytes)를 유지한다. public_guards의 정확한 재사용 근거는 최종 감사 정정을 따른다.
**U07/B13·C01은 독립 Complete / High로 종료했다.** 20수락 조건·10차원·161선행+31R06·6실행 노드가 현재 최종 증거에 결속됐다. [완료 인수](../core_first/execution/u07_closure_20261005_001/RESULT.ko.md)와 [최종 감사](../../goals/core_independent_objects/post_audit/u07_20261005_002/COMPLETION_AUDIT.md)가 현재 상태를 소유하고 동결 PLAN·첫 감사·모든 실패 원본은 보존한다. U07-M도 [전체 독립 감사](../../goals/core_runtime_measurement_profiles/post_audit/u07m_20261006_001/COMPLETION_AUDIT.md)에서 Complete / High·열린 finding0으로 종료했다. 10수락조건·10차원·181선행+31R06·5노드/1phase를 닫았으며 [완료 인수](../core_first/execution/u07m_closure_20261006_001/RESULT.ko.md)가 현재 완료를 소유한다. 동결 [실행 PLAN](../../goals/core_runtime_measurement_profiles/execution/single_goal/PLAN.md)은 당시 자체검증 기록으로 보존한다. 현재는 [U08 실행 PLAN](../../goals/core_relic_numeric_variants/execution/single_goal/PLAN.md)에 따라 GEX 인수/구현을 진행한다. U07-M은 단일 phase로 별도 Master GEX-30이 없다.

### U06 재개·교정 이력

**첫 감사 직후 위치(이력):** 독립 감사001 Not Complete / High. zero-effective 기본 Heal의 자동 자격과 query F01 하나를 correction GEX-10에서 교정·재검증 중이다. 첫 target/감사와 유효 선행 증거를 보존한다. U07 미진입. [감사](../../goals/core_battle_starter_tactics/post_audit/u06_20261005_001/COMPLETION_AUDIT.md)·[교정 인수](../../goals/core_battle_starter_tactics/execution/single_goal/correction/f01_001/INTAKE.json). 아래 자체검증 완료 문단은 첫 감사 전 이력이다.

**2026-10-05 재개 검증:** P2G-00→60/quality2와 GEX-00을 재시작하지 않고 BASIC 잔여부터 구현을 이어갔다.
현재 source의 Core1791/1791/1800·host92/100+별도heap1, fmt/check/event-log/clippy, 정상6세트·strict 작성·외부 API·5모드fault·실제 프로세스 Continue/rewind가 통과했다.
기존 V11 checksum 호환성은 기본 Damage의 serialization 생략으로 교정했고 버전/체크섬 기준은 유지한다.
PC576·공개 비용 guard·선행 조건 결속을 완료했고 R05는 기존 concurrent peak 경고(+605,609bytes)만 해당한다. 다음은 exact 동결 뒤 비참여 Astra High 독립 감사다. U06 Complete 및 U07 진입을 아직 주장하지 않는다.
[실행 PLAN](../../goals/core_battle_starter_tactics/execution/single_goal/PLAN.md)과
[실제 검증·실패 기록](../../goals/core_battle_starter_tactics/execution/single_goal/EXPERIMENTS.md)을 먼저 읽는다.
중지 당시에는 기본 Heal/AllySingle 부분 구현의369파일·집중9/9까지였고 큐4/4·native46/46은 더 이전 소스였다. 그 원본은 인수인계와 보관본에 유지한다.

U05 공명 접수·사용 수명은 [최종 독립 재감사](../../goals/core_resonance_lifecycle/post_audit/u05_20261004_002/COMPLETION_AUDIT.md)에서
**Complete / High**를 받았다.14 수락 조건·10차원·127 선행 조건·별도31 R06을 판정했다.
접수 시 전량 소비와 private 자격, 실제 사용 종료 잠금, 공통 양수 획득 gate·직원 초당1 충전,
엄격한 형태 최대치·사용 수명·대상·현재 저장/재접속을 정상 Core/host/data 경로로 연결했다.
Core1758/1758/1767·host91/99와 별도 heap1, 공개 소비자·PC576 전투 기준이 통과했다.
실제 비용 증가분을 명시해 비교했고 기존 수치 상한을 유지했다. R05는 승인된 peak 단독 경고(+466,821bytes)다.
첫 감사 중 외부 문서 이동으로 생긴 F01을 새 경로·대상 인수로 닫았으며 제품368개와43개 실행은 그대로였다.
[완료 인수·검증·남은 경고](../core_first/execution/u05_closure_20261004_001/RESULT.ko.md)를 따른다.
U05 완료 당시 다음 작업은 U06 P2G-00→60이었다. U06의 현재 완료는 위 최종 재감사와 완료 인수가 소유한다.
구현은 U05의 독립 완료를 선행 조건으로 인수하며 최신 룰북 경로와 기존 유효 증거를 재사용한다.

### 선행 U03·U04 완료 이력

U03 [최종 독립 감사](../../goals/core_runtime_fault_isolation/post_audit/u03_20261004_003/COMPLETION_AUDIT.md)는
**Complete / High**다. F01·F02를 닫았고 12 수락 조건·V01–V12·10 검토 차원·100 선행 조건·31 R06 조건의
현재 의미와 증거 적용성을 모두 판정했다. 종료 시 동결 구성원2,289개와 추가 입력647개가 일치했다.
[완료 인수·검증·남은 경고](../core_first/execution/u03_closure_20261004_001/RESULT.ko.md)가 현재 실행 요약이다.

최종 Core1717/1717/1726·host81/88 및 별도 heap1, 정적 검사·release 빌드, 공개 소비자16회와
PC576행 exact non-timing 비교를 통과했다. R05 raw101은 기존 승인 concurrent peak 단독 경고
(+232,950 bytes)이며 다른 hard guard 실패는 없다. 관측5모드 fault 결과가 같고 실제 stock WebSocket에서
fault·재접속·쓰기 차단·Quit/new owner/valid Continue를 확인했다. 원래 독립 F02 재현 시험도2/2 통과했다.

활성 로드맵 **U04 / S01–S08**은 P2G-00→60, GEX-10과 독립 GEX-20을 **Complete / High**로 마쳤다.
[정책·Goal 인계](../../goals/core_result_persistence/P2G_60_HANDOFF_001.json)와
[실행 현황·검증 위치](../../goals/core_result_persistence/execution/single_goal/PLAN.md)를 따른다.
승패 즉시 정산, 선택권, 후퇴2회, 미확정 노드 종료복구, 최신 Continue와 독립 SavePoint,
실제 host의 조건부 paired intent/CAS·tick/명령·프로필·query/ack 경로를 구현했다.
Compact V10 import 비용 누락을 재현·교정했고, 수정본 Core1734/1734/1743·host89/97 및 별도 heap1,
정적 검사와 공개 소비자16종·PC576 비교·실제 Continue/rewind·fault5모드 검증을 마쳤다.
R05 raw101은 기존 승인 peak 단독 경고(+269,128 bytes)이며 다른 hard guard 실패는 없다.
[자체 검증 결과](../../goals/core_result_persistence/execution/single_goal/GEX_10_RESULT.json)는 독립 판정이 아니다.
[첫 독립 감사](../../goals/core_result_persistence/post_audit/u04_20261004_001/COMPLETION_AUDIT.md)는
Not Complete / High로 종료했다. 현재 실행·증거에서 다른 차단 결함은 없었으나 필수 host 프로토콜
문서 갱신이 누락된 F01이 남았다. 원문 자체는 처음부터 target에 포함됐으며 감사의 별도 누락 주장은
실제 archive 대조로 정정한다. [교정 기록](../../goals/core_result_persistence/execution/single_goal/corrections/f01_host_contract_001/CORRECTION.md)에
현재 DTO·V5/V10·결과/선택/후퇴/Continue/SavePoint·paired intent와 profile-only 모드의 정합,
원문 보존과 현재 target 복원을 기록한다. 제품 소스와 최종 실행40건이 동일함을 검증했다.
[최종 독립 재감사](../../goals/core_result_persistence/post_audit/u04_20261004_002/COMPLETION_AUDIT.md)가
F01을 닫았고, 잘못된 파일 누락 주장을 명시적으로 정정했다. 15 수락 조건·15 검증·10 차원·112+31 선행 조건이
통과했으며 종료 시2,891개 구성원이 같았다. [완료 인수·검증·남은 경고](../core_first/execution/u04_closure_20261004_001/RESULT.ko.md)를 따른다.
U04 완료 직후에는 U05/B02·B03·B04·B09의 공명 접수·사용 수명 P2G로 인계했다.
미래 기능 schema와 최종보스 고유 구현은 해당 후속 범위다.


U02의 12 수락 조건과 단계 간 검증 13개, U01/R06/R07 선행 조건 88개의 현재 적용성을 확인했다.
최종 통합 검증은 Core default/test-fixtures 각각1701, observation1707; host77/84와 별도 allocator 시험이다.
35회 release off/on·수집 실패 비교와 실제 Prometheus/Grafana/browser 검증을 보존했다.
R05 raw exit101은 기존 승인 concurrent peak 단독 경고(+343,390 bytes)로 유지한다. OBS001과 20분 시험 제외도 유지한다.
위 수치는 완료된 U02 target의 선행 증거이며 현재 U03 교정 검증 수치와 구분한다.
U03에서는 오류를 패배로 정산하거나 원본 저장을 삭제하는 복구를 구현하지 않는다.

### 이전 F01 감사 상태 — 2026-10-03 target 이력

사용자의 R13~R16 연속 구현 요청에 포함된 R08~R12 선행 GEX-20을 실제 수행했다.
[독립 감사 보고서](../../goals/content_authoring_r08_r12_runtime/post_audit/20261003_001/COMPLETION_AUDIT.md)의 판정은 **Not Complete / F01**이다.
보스의 `CancelUnreleased` timeout·강제 state 이탈 후 미실행 structured Flow가 남아 후속 광역 피해를 발생시킨다.
공개 RON/release CLI의5개 oracle 중2개 대조군은 통과했고3개 취소 조건은 실패했다.

X09는 제외 없는 기본/feature 전체 시험 각각1677 PASS로 해소했다. source membership387개, 동결 기록1435개와 격리 정역 복원을 확인했다.
55조건·R06/R07 영향·현재성·정확 명령은 보고서에 연결한다. 제품·기존 target은 변경하지 않았고 감사자가 교정을 수행하지 않았다.
**다음 개발 작업은 R08/R10 F01 교정 successor·영향 검증·독립 재감사**다. 선행 PASS 없이 R13~R16을 시작하지 않는다.
[R13~R16 현재 위치](../../goals/content_authoring_r13_r16_runtime/PLAN.md)는 정식 계획/구현/최종 target이 아직 없음을 명시한다.
이 현황 갱신은 감사 종료 후 별도 문서 기록이며 [보존·적용성 기록](../../goals/content_authoring_r08_r12_runtime/post_audit/20261003_001/PUBLICATION_RECORD.json)에 귀속한다.

### 이전 구현 인계 상태 — 2026-09-30

아래는 독립 감사 전 구현자의 인계 기록이다. 현재 판정은 위 감사가 소유한다.

사용자의 추가 요청 “감사 수행하고 r08~r12 구현 수행해”에 따라 R06/R07 선행 감사를 별도 pass로 종료했다.
[R06 감사](../../goals/content_authoring_r08_r12_runtime/reviews/r06_r07_20260930_001/COMPLETION_AUDIT_R06.md)와
[R07 감사](../../goals/content_authoring_r08_r12_runtime/reviews/r06_r07_20260930_001/COMPLETION_AUDIT_R07.md)는 각각 Complete / High다.
[종료 기록](../../goals/content_authoring_r08_r12_runtime/reviews/r06_r07_20260930_001/AUDIT_CLOSURE.json)의 exact367 source를 인수한 뒤 제품 변경에 착수했다.
회복 preview 직원 배열 순서 차이 OBS001은 값/전투/저장에 영향 없는 기존 관찰 사항으로 보존했다.

현재 원본은 [실행 PLAN](../../goals/content_authoring_r08_r12_runtime/execution/single_goal/PLAN.md)이다.
[정식 계획](../../goals/content_authoring_r08_r12_runtime/PLAN_HANDOFF.ko.md)의 P2G40 receipt 생성/검증과 P2G50 quality2 검증을 실제 통과했다.
WP08~WP12를 구현했고, 같은 Goal에서55조건의 최종 source 증명과 통합 검증·동결을 수행한다.
최신 판정·환경 제약·검증 원자료는 실행 PLAN 및 [통합 인계](../../goals/content_authoring_r08_r12_runtime/execution/single_goal/REVIEW_HANDOFF.md)가 소유한다. 중간 집중 시험 수를 전체 통과로 해석하지 않는다.

| 항목 | 현재 기준 |
| --- | --- |
| 선행 | R06/R07 별도 감사 Complete / High; 감사 종료 source367/367 일치 |
| 실행 target | GS-CARX-001 / GOAL-CARX-01 / HA-CARX-001 |
| 현재 | WP08~12 구현 후 통합 검증·target 인계. 독립 감사 미발행 |
| 원본 보존 | 실행 baseline 실제 dirty/untracked source367+roadmap, worktree/index patches |
| 종료점 | R08~R12 구현·자체 검증 종료 / 최종 통합 target 동결 / 독립 감사 요청 대기 |
| 제외 | 차기 원정/GUI/편집기/R13+/실험실 stress/metrics/20분 시험 |

[이전 준비 조사](../../goals/content_authoring_r08_r12_runtime/preparation/20260930_001/PLAN.md)의 감사 대기는 당시 상태다.
[R08~R12 상세 지시서](handoffs/R08_R12_CONTINUOUS_IMPLEMENTATION_REQUEST_20260930.ko.md)와
[설계·수용표](handoffs/R08_R12_DESIGN_AND_ACCEPTANCE_20260930.ko.md)는 실행 기준으로 유지한다.
기존 R06/R07 최종 인계 원문은 자체 검증 당시 기록이며 소급 개정하지 않았다. 구현자는 R08~R12의 독립 PASS를 발행하지 않는다.

2026-10-02 후속 준비: [R13~R16 구현 지시서](handoffs/R13_R16_CONTINUOUS_IMPLEMENTATION_REQUEST_20261002.ko.md)와
[새 대화 시작 요청문](handoffs/R13_R16_START_PROMPT_20261002.ko.md)을 작성했다. 유물 침식·수치 합성·활성 조건·연쇄 정책의 최신 합의를
인수하며, R08~R12 선행 감사 확인 후 R13→R16을 연속 구현하는 범위와 수용 조건을 정했다.
지시서 v3는 이번 범위의 미정 정책을 구현자가 재량으로 결정·문서화한 뒤 질문 대기 없이 구현을 계속하는 사용자 위임을 포함한다.
선행 감사가 없으면 R08~R12 독립 감사를 먼저 수행하고, 통과 후 별도 재개 승인 없이 R13~R16 계획·구현으로 이어가는 요청도 반영했다.
문서 작성만으로 개발 주작업·선행 감사 상태·R13~R16 구현 상태가 바뀐 것은 아니다.

## 이전 요청과 인계

### R06→R07 당시 실행 요청과 인계 기록

[R06→R07 연속 구현 지시서](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/R06_R07_CONTINUOUS_IMPLEMENTATION_REQUEST_20260929.ko.md)와
[당시 구현 시작 요청문](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/R06_R07_CONTINUOUS_START_PROMPT_20260929.ko.md)이 R06/R07 실행 범위와 종료점을 소유한다.
R06 상세 설계는 아래 기존 지시서를 재사용했고, R06 target 보존 후 R07 정식 계획·구현과 최종 통합 검증을 수행했다.
R07의 정식 Goal/Handoff와 P2G 실제 validator 결과는 별도 Topic에 준비했다. 계획 검증은 구현 독립 감사 PASS와 구별한다.
R06/R07의 Goal·수용표·기록은 구별하고 최종 통합 source에서 함께 회귀를 확인한다. 미정 정책 결정 위임은 두 범위에 적용한다.

[R06 잔여 전체 구현 지시서](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/R06_COMPLETION_IMPLEMENTATION_REQUEST_20260929.ko.md)와
[이전 R06 단독 시작 요청문](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/R06_COMPLETION_START_PROMPT_20260929.ko.md)은 이전 범위 이력이며, 최신 요청은 위 연속 지시서를 따른다.
최신 C07 manifest `41b98656…7a882`의347개 source currentness·membership 및 격리 양방향 복원을 확인했다.
이는 C07의 독립 동작 검토 PASS가 아니다. 기존 단계별 중단 지시는 이번 잔여 전체 구현 실행에 한해 위 지시서로 대체한다.
R06 기록은 기존 `execution/single_goal/continuations/r06_completion_001/`에 보존한다. R07 및 최종 통합 current locator는 [실행 디렉터리](../../goals/content_authoring_fragment_copies/execution/single_goal/PLAN.md)다. 두 Goal의 범위와 수용표를 분리했다.
이하 단계별 중단·차단 설명은 각 당시 구현/검토 이력이다. 최신 구현 범위·종료 조건은 위 지시서를 따른다.

사용자에게 집계 정책을 확인하고 [R06 범위 재대조·후속 계획](../../goals/content_authoring_value_expressions_reassessment/successors/r06_scope_20260927_001/REVIEW_AND_PLAN.ko.md)에 반영했다.
R06은 이미 resolve된 유닛의 진영·실제 직원·명시한 착용 슬롯의 실제 장비 사본별 집계와 명시적 제외 조건까지 구현한다.
소환물을 직원에 자동 합산하거나 부적격 member를 조용히 제외하지 않는다. 효과·상태의 **출처 장비별** 조회·집계는
R11 기반 및 최초 소비자/R17 통합으로 배정했다. R 순서·의존성은 유지한다.

유닛 관계·필터·장착 슬롯 전달 구현 뒤 발견한 D04-OWNER-F01은 교정 successor에서 해소되었다.
[D04 독립 재검토](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d04_owner_f01_correction_001_review_001/REVIEW.md)는
**이번 D04 유닛 관계 범위 PASS**다. 전역 장비 탐색을 정확한 runtime ID 직접 조회로 바꾸고,
기존 identity 검사·계량·생성 경로를 유지했다. 독립 전체 시험1,526개와337개 currentness/격리 복원이 통과했다.
R05 concurrent peak 11,612 bytes 초과는 합의된 경고로 유지한다.

[D07-A 독립 단계검토](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d07_static_explain_001_review_001/REVIEW.md)는
**부분 범위 PASS**다. 기존 Effect explain에 실제 winning compiled plan의 필요 입력·단위·moment·출처·fingerprint,
segments와 명시적 미평가 상태를 연결했다. 338개 currentness/격리 복원, 동결 기록98개,
같은 소스의 독립 전체 시험1,533개와 실제 CLI 설명·HealExpr 공개 거부를 확인했다.
정적 설명은 값을 계산하거나 실제 피해량을 추정하지 않으며 production publication 차단도 유지한다.

[D07-B 구현 지시서](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/WP02_D07_B_IMPLEMENTATION_REQUEST_20260927.ko.md)를 작성했다.
구현자는 D07-B 실제 평가 문맥·amount/trace 관측을 구현하고 target을 동결했다.
사용자 결정에 따라 전투별 최근 평가 1건을 보관하며 오래된 기록은 보관되지 않음으로 명시한다.
Snapshot release frame과 Live occurrence frame을 실제 소비 시점에 보존하고 공통 evaluator로 읽기 전용 조회한다.
host의 명시적 숫자 관측 설정, 기존 active/last-terminal 수명, quota·rollback·clone/retained 계량,
public legacy와 staged dynamic 검증 및 B01–B20을 지시서에 지정했다. 미래 시전 예측·전체 이력은 이번 범위가 아니다.
계획·단계검토는 이 대화에서 진행하고, 구현자는 자체 검증·target 동결 뒤 검토 요청 대기에서 중단한다.

[D07-B 단계검토](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d07_runtime_amount_001_review_001/REVIEW.md)는
**수정 필요**다. 현재 source와 동결 대상·복원 자료는 일치한다. 정상 CLI의 trace on/off 4개 amount 문서와
집중 시험 5개를 재확인했지만, 요청 파일 인자가 없는 simulate의 panic을 재현했고 새 legacy 관측 배열의
allocation 전 admission 누락과 동적 Snapshot/Live·한도·취소 등 필수 검증의 부족을 확인했다.
F01–F03 교정과 successor 재검토가 다음 작업이다. R05 concurrent peak는 합의대로 경고이며 이 판정의 차단 원인이 아니다.

[D07-B 교정 지시서](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/WP02_D07_B_CORRECTION_REQUEST_20260928.ko.md)를 작성했다.
실제 parser/배열 admission 수정 방향과 C01–C16의 동적 입력·수명·실패·비용·출력 시험을 명시했다.
첫 후보는 독립적으로 검증 가능한 CLI 교정이며, 의미 있는 부분이 닫히면 남은 finding을 표시하고 동결·검토한다.
이후 사용자가 분기 완료와 원본 구현 시작을 요청했다. 첫 구간은 F01/C01에서 중단했다.
[F01 교정 인계](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_d07_b_review_001/REVIEW_HANDOFF.ko.md)에
교정 전 panic 재현, 교정 후 실제 CLI 검증과 successor 위치를 연결했다.
[F01 교정 단계검토](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d07_b_f01_correction_001_review_001/REVIEW.md)는
**F01 부분 범위 PASS**다. 실제 CLI 오류 시험4개와 정상 실행8건을 재확인했고, 동결 대상·추가 파일 membership·
양방향 복원 및 runtime 근거 재사용의 적용성을 확인했다. 다음 구현은 원본 대화의 재개 요청에 따라 진행한다.
사용자 재개 요청으로 F02/C02 교정을 수행했다. [F02 인계서](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_d07_b_f02_001/REVIEW_HANDOFF.ko.md)에
공통 배열 admission, 정확 한도/1 byte 부족/overflow 및 actual legacy transaction의 기존 record 보존을 연결했다.
최종 전체1545개·집중C02 3개·authoring112개·value_expr94개가 통과했고 PC36 결과는 동일하다.
R05 concurrent peak9,049,037 bytes의 원 cap18,849 초과만 경고다.
[F02/C02 사본 단계검토](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d07_b_f02_correction_001_review_001/REVIEW.md)는
**부분 범위 PASS**다. 집중 시험3개 재실행, 11개 최종 실행 영수증·PC 원자료576행·R05 sample 대조,
341개 currentness와 격리 양방향 복원을 확인했다. 제품 소스나 기존 동결 기록은 변경하지 않았다.
사용자 재개 요청으로 F03의 C04/C05 실제 동적 소비·기록 조회 시험을 보강했다.
[F03 C04/C05 인계](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_d07_b_f03_replay_001/REVIEW_HANDOFF.ko.md)와
[현재 위치/target](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_d07_b_f03_replay_001/PLAN.md)에 연결했다.
신규5개 시험/11개 시나리오, authoring112개·value_expr99개·전체1550개, check/Clippy/format이 통과했다.
제품 실행 코드 byte가 보존되어 PC36/R05는 F02 동결 원자료를 재사용했다. 이번 새 측정값은 아니다.
342개 successor의 [C04/C05 사본 단계검토](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d07_b_f03_replay_001_review_001/REVIEW.md)는
**부분 범위 PASS**다. 집중5개 재실행·11시나리오의22쌍 문서, 최종 영수증7건·선행 측정 재사용 적용성,
342개 currentness·375개 기록·격리 양방향 복원을 확인했다. private encoder로 public selector까지 검증했다고 주장하지 않는다.
그 검토 당시 C03·C06–C15가 남아 있었으며, 이후 사용자 재개 요청으로 C03을 진행했다.
[C03 구현 인계](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_d07_b_f03_legacy_001/REVIEW_HANDOFF.ko.md)와
[현재 위치/target](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_d07_b_f03_legacy_001/PLAN.md)을 연결한다.
legacy Damage/Heal MIN·음수·0·양수·MAX의 실제 command 입력/HP/완료 fact와 관측 on/off 동등성을 검증했다.
신규2개 시험(10경계값×3설정), authoring112개·value_expr101개·전체1552개 및 check/Clippy/format이 통과했다.
343개 successor의 [C03 사본 단계검토](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d07_b_f03_legacy_001_review_001/REVIEW.md)는
**runtime 부분 범위 PASS**다. 집중2개·30회 command/HP 대조, 최종 영수증7건, 343개 currentness·414개 기록·격리 복원을 확인했다.
모든 변경은 cfg(test)이며 PC36/R05 재사용 적용성도 확인했다.
음수 Damage는 공개 거부를 유지하고 내부 staged compatibility로만 검증했다. 원문 RON MIN 파싱 실패(C03-OBS01)는
해결되지 않았다. 사본에서 현재 RON 의존성의 MIN scalar 직렬화 성공·역직렬화 실패와 부호 처리 원인을 재현했다.
C15 공개 입력/직렬화 통합 때 해결·재검증 또는 명시적 범위/정책 결정을 확인해야 한다. Heal MIN의 typed authoring→catalog→runtime 통과와 구분한다.
그 검토 당시 C06–C15가 남아 있었으며, 사용자 재개 요청으로 C06을 진행했다.
[C06 구현 인계](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_d07_b_f03_aggregate_001/REVIEW_HANDOFF.ko.md)와
[현재 위치/target](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_d07_b_f03_aggregate_001/PLAN.md)을 연결한다.
신규3개 시험·10시나리오로 진영/실제 직원/지정 장비 사본의 독립 node view, 명시적 필터,
Snapshot37/Live40의 실제 command·HP와 보관된 amount/trace, 빈 집합0 및 관계 오류4종의 transaction rollback을 검증했다.
18쌍 preview/explain 문서는 재조회와 후속 입력 변경에도 보존된다. private encoder 검증이며 공개 selector 통과를 주장하지 않는다.
최종345개 입력 기준 전체1556개·authoring112개·value_expr104개와 check/Clippy/format이 통과했다.
이번 작성 변경은 cfg(test) 모듈 선언1개와 신규 시험 파일1개다. 실행 중 별도 대화가 추가한 `tests/battle_showcase.rs`는
변경하지 않고 별도 실행 입력으로 보존했다. 최초 영수증이 이 추가 파일을 계량하지 못해 최종 입력을 확장하여 검증을 다시 수행했다.
따라서 343→345 복원에는 이번 시험 파일과 별도 통합 시험 파일이 포함되며, 전체 증가4개 중 C06은3개다.
PC36/R05는 제품 실행 코드 불변 근거와 함께 F02 동결 원자료를 재사용했다. 새 비용 측정은 아니다.
[C06 사본 단계검토](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d07_b_f03_aggregate_001_review_001/REVIEW.md)는
**부분 범위 PASS**다. 집중3개 재실행·18쌍 문서·오류4종, 최종 영수증7건·시연 파일 identity,
345개 currentness·472개 기록·격리 양방향 복원을 확인했다. C07–C15 및 F03/D07-B 전체 완료는 남아 있다.

사용자 재개 요청으로 [C07 구현 인계](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_d07_b_f03_order_001/REVIEW_HANDOFF.ko.md)와
[현재 위치](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_d07_b_f03_order_001/PLAN.md)를 작성했다.
최근 A→B 실제 교체 뒤 A Exact not_retained, Latest/B Exact의0값 기록, 같은 timestamp의 복수 effect/target 순번,
관통0/2 amount의 연속 적중과 D04-F01 empty impact의 후속 resource·완료를 검증했다.
새4개 시험과 공개 preview/explain7쌍이 통과했고 전체1560개가 약132초에 통과했다. 제품 실행 코드 변경은 없다.

**C07-OBS01 미해결:** 적중한 관통 projectile 만료 시 native carrier/facts는 닫히지만 active cast와 step pending_delivery_count1이 남는다.
[재현·사용자 결정](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/corrections/wp02_d07_b_f03_order_001/evidence/C07_OBS01.json)에 따라
**C09 수명 검증에서 교정**하고 C07을 먼저 동결한다. C07의 만료 후 record 보존을 전체 시전 수명 PASS로 해석하지 않는다.
max_pierces=1 terminal hit 대조에서는 정상 정리된다. F03 전체 완료 전 교정 필수이며 C03-OBS01/C15와 별개다.

[successor 계획 §5](../../goals/content_authoring_value_expressions_reassessment/successors/r06_scope_20260927_001/REVIEW_AND_PLAN.ko.md#5-구현-순서와-완료-판단)의 전체 의무는 유지한다.
실제 frame amount/trace preview와 usefulness는 D07-B, 최종 effective input·Certified fingerprint·legacy save는
D07-C/WP03 연계 구간으로 구분했다. A/B/C는 지시서 내부 단계이며 새 R/Goal이 아니다.
WP03 전에 D07 전체나 WP02/R06 전체를 완료로 표시하지 않는다. WP04와 최종 전체 검증도 남는다.
이번 독립 검토에서는 제품 소스를 변경하거나 publication을 해제하지 않았다.

이전 [수정 필요 검토](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d04_owner_relation_001_review_001/REVIEW.md),
[교정 지시서](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/WP02_D04_OWNER_F01_CORRECTION_REQUEST_20260927.ko.md),
[O01–O15 정책](../../goals/content_authoring_value_expressions_reassessment/successors/r06_scope_20260927_001/IMPLEMENTATION_REQUEST.ko.md)은 당시 근거로 보존한다.
[스킬 모델](../../skills/skill_authoring_model.ko.md)은 계속 개선하는 기준안이며 전체 모델의 영구 확정·편집기 상세 설계를 새 선행 조건으로 두지 않는다.
다른 R이나 실험실 작업으로 자동 전환하지 않는다. MS01 완료는 R06 재개의 선행 조건이 아니다.

### R06 재개 근거

1. [기존 계획 인계](../../goals/content_authoring_value_expressions_reassessment/PLAN_HANDOFF.ko.md): `GS-CAVER-P2G50-001 / GOAL-CAVER-01`, `HA-CAVER-P2G60-001`. **계획 당시** 기록이며 최신 계획은 8번 successor, 구현 상태는 아래 동결 근거와 함께 읽는다.
2. [D04 구현 인계·잔여 범위](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/segments/wp02_d04_resolved_target_001/REVIEW_HANDOFF.ko.md).
3. [D04-F01 교정 독립 검토](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d04_f01_correction_001_review_001/REVIEW.md): 교정 기능 PASS. WP02/R06 전체 완료가 아니다.
4. [이후 비용 정책 변경](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/R06_COST_POLICY_UPDATE_20260927.ko.md): 약9MB concurrent peak 초과만의 개발 차단을 경고·관찰로 전환. 당시 실패와 감사 판정은 보존한다.
5. [상세 설계 원본](../../goals/content_authoring_value_expressions_reassessment/CODE_REVIEW_003.ko.md): 잔여 설계와 현재 구현의 차이를 확인할 때 사용한다.
6. [콘텐츠 작성 체계 검토·후속 공개 범위 합의](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/AUTHORING_SYSTEM_CONTENT_REVIEW_20260927.ko.md): §9–10이 후속 사용자 합의다. 본편 전용 기능을 허용하고 모드의 작성 보장은 공개 기능 범위로 한정한다. 기존 공식 콘텐츠 교체 권한은 유지한다.
7. [스킬 작성·실행 모델](../../skills/skill_authoring_model.ko.md): 현재 모델 설계의 기준 문서. 래티씨아는 공격 가능한 친구만 공격·소비하고 나머지는 유지하는 사용자 결정을 반영했다. 예시는 설계용이며 현 parser/실행기 지원을 뜻하지 않는다.
8. [R06 successor 계획 인계](../../goals/content_authoring_value_expressions_reassessment/successors/r06_scope_20260927_001/PLAN_HANDOFF.ko.md): 최신 집계 정책·실제 코드 대조·다음 D04 구간·완료 기준. 기존 계획의 부분 구현과 동결 증거를 이어받는다.
9. [D04 유닛 관계 독립 단계검토](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d04_owner_relation_001_review_001/REVIEW.md): manifest `c2e703c2…a2a9f`, 337개 source 대상. 당시 수정 필요 판정은 보존하며 F01의 이후 상태는10번을 따른다.
10. [D04-OWNER-F01 독립 재검토](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d04_owner_f01_correction_001_review_001/REVIEW.md): manifest `58505bbc…14e549`, 337개 source/변경5개 경로. F01 해소, 이번 유닛 관계 범위 PASS. WP02/R06 전체 완료가 아니다.
11. [D07-A 독립 단계검토](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d07_static_explain_001_review_001/REVIEW.md): manifest `ae6609c1…9bb1ac`, 338개 source/기존6개 변경·1개 추가. 정적 설명 부분 범위 PASS. 전체1,533개 시험은 동일 소스의 선행 독립 실행을 검증해 재사용했다. D07-B/C와 WP03/04는 남아 있다.

구현·검토 근거의 시험 결과는 각 당시 target에 귀속한다. 최신 D07-A 검토 근거는11번, D04 교정 판정은10번에 기록한다.
이전333개/337개 확인과 신규 모듈을 포함한 최신338개 검토를 구별한다. 선행 완료 감사를 다시 판정하지 않는다.

## 전체 진행표

기본 R 순서와 기능/검증 선행은 [기존 의존성 검토](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/DEPENDENCIES.ko.md)를 유지한다.
아래는 일상적인 순서·현황 조회표다. 각 선택 범위의 실제 구현 순서와 완료 조건은 current Phase Plan이 소유한다.
미확인을 미착수·완료로 바꾸지 않는다. R01–R05의 완료는 링크한 당시 successor 감사에 귀속한다.

| R | 작업 / 상세 카드 | 현재 상태 | 기능 선행 | 검증 선행 | 완료·진행 근거 |
| --- | --- | --- | --- | --- | --- |
| R00 | [기존 M0 인수·전체 대응 기준선](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R00.ko.md) | 기존 인수 기록 | — | — | [기준선](records/CA00_NEXT-0002_REQUIREMENTS.ko.md) |
| R01 | [효과 주소·태그·typed property 최소 전환 — 현 GOAL 유지](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R01.ko.md) | 완료 기록 있음 | — | R00 | [successor 감사](../../goals/content_authoring_minimal_effect_contract/execution/single_goal/COMPLETION_AUDIT_RESUME_001.md) |
| R02 | [외부 다중 팩·검증 세대·새 런/저장 결속](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R02.ko.md) | 완료 기록 있음 | R01 | — | [successor 감사](../../goals/content_authoring_pack_generation/execution/single_goal/COMPLETION_AUDIT_RESUME_001.md) |
| R03 | [공통 작성 API·CLI·설명·headless 최소 포트](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R03.ko.md) | 완료 기록 있음 | R01, R02 | — | [successor 감사](../../goals/content_authoring_tool_ports/execution/single_goal/COMPLETION_AUDIT_RESUME_001.md) |
| R04 | [사건·실제 결과·출처·현재 전달체의 ActionHandle](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R04.ko.md) | 완료 기록 있음 | R01 | — | [successor 감사](../../goals/content_authoring_runtime_facts/execution/single_goal/COMPLETION_AUDIT_RESUME_001.md) |
| R05 | [타입화 자원·owner×lifetime·원자적 소비](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R05.ko.md) | 완료 기록 있음 | R04 | — | [successor 감사](../../goals/content_authoring_resource_lifetimes/execution/single_goal/COMPLETION_AUDIT_RESUME_001.md) |
| R06 | [ValueExpr·집계·Attack/SpellPower와 무기 성장](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R06.ko.md) | **구현·자체 검증 종료 · 독립 감사 대기** | R01, R04, R05 | — | [재개 근거](#r06-재개-근거) |
| R07 | [파편 사본·개화 선택·정상 획득/장착/저장 동시 전환](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R07.ko.md) | **구현·자체 검증 종료 · 독립 감사 대기** | R01, R05 | — | [연속 지시서](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/R06_R07_CONTINUOUS_IMPLEMENTATION_REQUEST_20260929.ko.md) |
| R08 | [보스 최소 스킬 요청·완료·실패·취소·방향 adapter](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R08.ko.md) | 후속 범위 · 완료 미확인 | R01, R04 | — | — |
| R09 | [파편 침식·강제 시계·직원 대가/Breakdown 동시 전환](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R09.ko.md) | 후속 범위 · 완료 미확인 | R04, R05, R07 | — | — |
| R10 | [대상·전달·구조화 Flow·유한 반복](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R10.ko.md) | 후속 범위 · 완료 미확인 | R01, R04, R05, R06 | — | — |
| R11 | [가역 modifier·membership·channel·기본 공격 교체](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R11.ko.md) | 후속 범위 · 완료 미확인 | R05, R06, R10 | — | — |
| R12 | [개인/공유 보호막·생존·제어·HP 비용 중재](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R12.ko.md) | 후속 범위 · 완료 미확인 | R04, R05, R06, R11 | — | — |
| R13 | [소환 actor·정지 trap·field object·진화](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R13.ko.md) | 후속 범위 · 완료 미확인 | R04, R05, R07, R10, R12 | — | — |
| R14 | [유물 관리 + 최소 수치 합성의 실제 실행 연결](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R14.ko.md) | 후속 범위 · 완료 미확인 | R01, R05, R07, R10 | — | — |
| R15 | [유한 활성 closure·생성 효과·구조 recipe](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R15.ko.md) | 후속 범위 · 완료 미확인 | R11, R12, R14 | — | — |
| R16 | [반응형 연쇄·branch-local 재방문·비동기 인과](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R16.ko.md) | 후속 범위 · 완료 미확인 | R04, R05, R14, R15 | — | — |
| R17 | [연결망·원천별 감염·공간 집계/일제 소비](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R17.ko.md) | 후속 범위 · 완료 미확인 | R06, R13, R16 | — | — |
| R18 | [보스 형태·조건·선택·전장 신호/다중 actor 통합](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R18.ko.md) | 후속 범위 · 완료 미확인 | R08, R11, R13 | — | — |
| R19 | [등록·획득 선택·정책 profile·전투력 최종 입력](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R19.ko.md) | 후속 범위 · 완료 미확인 | R02, R07, R14, R15, R18 | — | — |
| R20 | [런 결과·후퇴·재시도·저장·재접속 교차 검증](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R20.ko.md) | 후속 범위 · 완료 미확인 | — | R09, R17, R18, R19 | — |
| R21 | [대표 공식 최신 RON C01–C07·전체 표현력 대조](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R21.ko.md) | 후속 범위 · 완료 미확인 | R03 | R15, R16, R17, R18, R20 | — |
| R22 | [엔진 동결 후 제3자 data-only 재조합 인수](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R22.ko.md) | 후속 범위 · 완료 미확인 | — | R21 | — |
| R23 | [결정론·예산·회귀·측정 기반 성능 닫힘](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R23.ko.md) | 후속 범위 · 완료 미확인 | — | R22 | — |
| R24 | [독립 완료 감사·제작 기반 인계](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R24.ko.md) | 후속 범위 · 완료 미확인 | — | R23 | — |

R09 정책 대기 시 R10–R19의 독립 범위를 진행할 수 있으나 R20–R24 전체 완료는 막히는 기존 조건부 분기를 유지한다.
다른 미정 recipe/정책의 분기는 [기존 순서의 조건부 진행](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/ROADMAP.ko.md#5-정책-대기-때의-제한된-분기)을 따른다.
R23 변경이 의미·지원 범위에 영향을 주면 관련 R22 증거를 재획득한다. 이는 최초 실행 순서의 순환이 아니다.
기존 M/CA 번호는 [요구 추적 대응표](reviews/2026-09-09-dependency-review-v2.2/report/M_CA_MAPPING.ko.md)에서만 확인한다.

## 후속 편입 후보

[모드 검증·실행 안전성 계획](../mod_validation_safety/ROADMAP.ko.md)은 지원 범위·내부 작업·의존성을 보관한다.
다음 위치는 **잠정 편입안**이며 새 실행 지시나 현재 R06의 완료 조건이 아니다.

| 후보 위치 | 검토한 작업 |
| --- | --- |
| R06 뒤 | 기존 한도의 RON 분리, 메트릭, 기본 실험실 시험 |
| R07–R18 기능 도입 시 | 해당 기능의 시험 profile 보완 |
| R19 전 | 전투 오류 처리, 모드 검증·캐시·적용. 본편 모드 제공을 앞당기면 함께 앞당김 |
| R21 뒤·R22 동결 전, R23–R24 | 대표 시험 기준 정리, 기존 통합 검증/감사 연결 |

세부 MS 번호는 후속 계획 안의 작업 구분이다. 프로젝트의 다음 작업을 결정하지 않는다.
[확정 정책](../../plans/runtime_safety_observability/LAB_STRESS_PROPOSAL.ko.md)은 유지하며 20분 시험·안전 초과 스킬 자동 취소는 제외한다.

## 운영과 이력

- [운영 규칙](OPERATING_RULES.md): 재개·범위·현재 근거 확인. [단일 R 계획 요청문](AUTORUN_REQUEST.ko.md)은 사용자가 명시적으로 채택할 때만 사용한다.
- [P2G 원문](../../policy_to_goal_pipeline_revised_v2.md) / [GEX 원문](../../goal_execution_pipeline%281%29.md): 선택한 계획·구현·검토에 필요한 절만 읽는다.
- 과거 기록: [0001–0030](history/2026-09.md), [0031–0060](history/2026-09-02.md), [0061–0072](history/2026-09-03.md), [0073–0080 및 후속 기록](history/2026-09-04.md).
- `roadmap/`는 CA 기반 참조 원문, `reviews/`는 의존성 연구 원문, `records/`는 당시 인수·변경 증거다. 현재 상태를 결정하는 별도 진입점으로 사용하지 않는다.
- 배포 보조 파일과 cleanup 전 원문: [아카이브·정리 기록](archive/cleanup_20260927/README.ko.md).

### 최근 변경 · 2026-09-27

D07-B의 실제 runtime 평가·소비, trial query, observer/terminal, clone/retained 및 CLI 경로를 코드로 대조했다.
전투별 최근 평가1건 보관을 사용자에게 확인하고 구현 지시서를 작성했다. 큰 i128 trace는 기존 Value 기반 step에
끼워 넣지 않고 typed API 문서로 출력하도록 정했다. 이 작업은 문서·읽기 전용 currentness 확인이며 제품 구현/시험은 수행하지 않았다.

D07-A 동결 target을 독립 검토해 부분 범위 PASS로 기록했다. 338개 source와 기록98개·격리 복원을 확인하고,
같은 source digest의 선행 독립 전체 시험1,533개를 재사용했다. 실제 CLI 설명과 추가 HealExpr 공개 거부도 확인했다.
다음 작업은 D07-B 실제 평가 문맥·수치/trace 관측 설계와 구현 지시서다. D07/WP02/R06 전체 완료는 아니다.

D07의 현재 explain/공간 preview/runtime 평가 연결을 코드로 대조하고 첫 정적 설명 구간의 구현 지시서를 작성했다.
public legacy 설명과 내부 staged expression 설명을 구분하고 production 차단을 유지하도록 명시했다.
실제 frame 관측과 WP03 저장·최종 입력은 후속으로 연결했으며, 현재337개 source는 PASS target과 일치한다.

D04-OWNER-F01 successor를 독립 재검토하여 해소/부분 범위 PASS로 기록했다.
별도 product target에서 전체1,526개 시험을 재실행했고, PC 원자료 재비교와 R05 peak 단일 경고를 확인했다.
다음 작업을 D07 설명·미리보기의 구현 범위/지시서 정리로 갱신했다. 제품 코드나 기존 동결 기록은 수정하지 않았다.

D04-OWNER-F01 교정 지시서를 작성했다. 실제337개 source가 동결 after와 일치함을 재확인하고,
슬롯의 runtime item ID 연결·생성 순번 보존·직접 조회·C01–C09 시험 및 successor 인계 절차를 지정했다.
이 작성 작업에서는 제품 코드 변경이나 제품 시험을 수행하지 않았다.

D04 유닛 관계 구현의 동결본을 독립 검토했다. 집계 밖 장비가 증가해도 조회 작업량이 계량되지 않는
D04-OWNER-F01을 재현하여 수정 필요로 기록했다. 직접 조회 연결과 identity 검사 유지 등 교정 방향을
검토서에 남겼으며, 현재 작업은 R06 교정·재검토다. 제품 코드와 기존 동결 자료는 변경하지 않았다.

구현 담당자의 설계 판단을 줄이도록 코드를 다시 읽고 구체적인 작업 지시를 추가했다.
집계 node별 view, layout/빈 슬롯의 구별, 두 spawn 경로, 수동 fingerprint encoding, clone/extent 및 시험 helper의 한계를 명시했다.
사용자 정책·기존 successor 동결 산출물과 R 순서는 변경하지 않았다.

R06 목적을 수치 계산 기반으로 유지하며 E01–E09를 모델·실제 코드와 대조했다. 직원·두 장비 관계·명시적 제외·후속 조회 배정의
사용자 정책을 기록하고 D04 구현 지시서를 작성했다. 기존 27개 수용 기준을 유지하고 집계 기준 4개를 추가했다.
기존 동결 계획/target은 보존하며 successor 변경 관계로 연결한다. 제품 구현은 별도 대화에서 진행한다.

스킬 모델 우선 설계 순서를 채택해 모델 기준안 한 개를 작성했다. 대표 7종과 경계 시나리오 18개를 정적으로 대조하고,
래티씨아의 무대상 node 보존을 사용자에게 확인했다. 다음 작업은 동결 계약과의 차이 대조이며 추가 제품 구현은 진행하지 않았다.

콘텐츠 설계 노트 전체를 검토하고 본편 전용 기능과 공개 작성 기능을 분리하는 사용자 합의를 기록했다.
상세 근거는 위 재개 근거 6번에 연결한다. R21/R22의 표현력·재조합 인수 기준도 향후 이 경계에 맞춰 구체화한다.
R 순서·의존성과 동결 계획/증거는 이번 문서 변경으로 수정하지 않는다.

로드맵 cleanup으로 개발 주작업/이번 문서 작업/다음 개발 작업을 구분하고 R06 재개를 복구했다.
R 순서·의존성·동결 자료는 보존하고, 배포용 파일 6개를 아카이브했으며 과거 기록 12건을 이력으로 옮겼다.
[검증 결과](archive/cleanup_20260927/VALIDATION.json)는 링크·보존·문서 구조 확인이며 제품 시험 결과가 아니다.
