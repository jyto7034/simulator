# U14 독립 재감사 — Complete / High

2026-10-14, GEX-20 stage-run. `GS-CEDR-001` / `GOAL-CEDR-01` / `PHASE-CEDR-01`.

**판정: Complete / High, 열린 완료 차단 finding 0건.** `u14_current_002`에서 F-U14-001과 교정 중 발견된 C-U14-002를 독립적으로 닫았다. 31수용 조건·10검증 차원·10통합 조건이 충족된다. U15는 착수하지 않았다.

## 권한·독립성·고정 대상

사용자의 “상시 지시서를 읽은 뒤 u14 독립 재감사 작업에 착수” 요청에 따른 감사 한 단계다. 이번 대화의 실행자는 U14 구현·교정에 참여하지 않았다. 제품 수정·서브에이전트·자동 위임 없이 소스 검토와 별도 감사용 소비자 실행을 수행했다. 모델 전환을 주장하지 않는다.

- [구현 target002](target/u14_current_002/AUDIT_TARGET_RECORD.json), [감사 인수 기록](audit/u14_independent_002/AUDIT_TARGET_RECORD.json).
- HEAD `2f725172edbffd27c5078d29000a1a2ba6ddc4d9`; SOURCE SHA256 `94e464a506ec9899ffcd40c9aa69f7711aa1063bc83d207b8d1502616f55bee7`.
- [입구](audit/u14_independent_002/ENTRY_CURRENTNESS.json)·[출구](audit/u14_independent_002/EXIT_CURRENTNESS.json): source713 / 교정 evidence600 / predecessor evidence980 모두 일치. 세 production root의 누락 파일 없음.
- [정역복원 검산](audit/u14_independent_002/RESTORE_RECHECK.json): predecessor archive704 → successor713의21개 delta를 정·역으로 적용해 전체 목록과 bytes를 독립 검산했다. 기존 격리 filesystem 복원은 재사용했다. 작업트리 patch 적용0.
- 이전 target001 Not Complete [보고서 원문](audit/u14_independent_002/PREDECESSOR_COMPLETION_AUDIT.md)과 [독립 감사 기록](audit/u14_independent_001/AUDIT_CLOSURE.json)은 보존한다. 동결 PLAN/EXPERIMENTS/COVERAGE의 재감사 대기 문구는 교정 인계 당시 상태이며, 최신 감사 판정은 이 보고서와 HANDOFF가 가리킨다.

판정 기준은 FPF-CEDR-001, PP-CEDR-001, [Goal Set Completion Matrix](../../P2G_50_GOAL_SET.ko.md), 현행 상시 지시서 및 GEX-20이다. 외부 임의변조 전용 사례는 새 finding이나 완료 조건으로 추가하지 않았다.

## 두 결함의 closure와 소스 검토

[FINDINGS](audit/u14_independent_002/FINDINGS.json)에 발생 경로·교정 근거·독립 closure를 연결했다. 두 사례 모두 지원 제작/명령/실제 저장 경로에서 발생한 범위 안 결함이다.

**F-U14-001 — 방문 방 봉인·전투 배율 공개:** `facility_rooms.rs`의 공통 predicate는 명시적 방ID와 원래 node의 slot을 조회한다. 전체 지도는 순회 중인 방을 전달하고, 전투/정산은 기존 current-map wrapper를 유지한다. collected 및 monolith 제외가 동일하게 적용되며 두 공개DTO는 같은 preview를 사용한다. 새로운 봉인 장부나 DTO 권위를 추가하지 않았다.

감사 소비자는 정상 작성→package_official→load→공개 시작/이동 명령으로 세 사례를 실행했다. 동일 slot의 다른 방 보스를 대조하고, 서로 다른 봉인180/170과 무억제150/150으로 잘못된 중첩을 구별했다.

| 공개 경로 | 시작→옆방→복귀 억제력 | 옆방에서 본 원방 보스 HP/ATK | 봉인 |
| --- | --- | --- | --- |
| 미회수·억제 | 12→9→6 | 180% / 170% | 있음 |
| 미회수·무억제 | 3→0→0 | 150% / 150%, 승리 수량2배 | 있음 |
| 실제 승리로 회수 | 12→승리 보충18→옆방16→복귀13 | 100% / 100% | 없음 |

모든 사례에서 node/activity preview 일치, 같은slot 타방 미봉인, 초기 hidden identity, 조회 전후 roots 불변, 같은 node·marker와 왕복 배율을 확인했다. 각 왕복 전후 child-process Continue6회가 정확한 roots를 복원했다. [실제 결과](audit/u14_independent_002/public001_observations/RESULT.json), [진행기](audit/u14_independent_002/probe/src/main.rs).

**C-U14-002 — 정상 보충 후 초기값 초과 저장 거절:** `node_flow.rs`의 검증은 각 payload의 초기 공급 + 회수한 가지 수 × 보충량을 넓은 정수로 계산한다. Continue의 current/SavePoint와 명시적 SavePoint 불러오기가 같은 검증을 사용한다. 과거 SavePoint에 현재 회수 내역을 대입하지 않는다. 실제 지급 checked_add, 저장 writer15, reward producer, application 원자성 및 host journal은 변경되지 않았다.

추가 독립 소비자는 위 실제 승리의 정상 export를 새 프로세스로 받아 공개 `load_run_checkpoint`를 수행했다.

| 실제 입력 | 불러온 SavePoint | 기대 및 관측 |
| --- | --- | --- |
| 현재 잔액16·회수1 | 수집 전 원방 SavePoint | 잔액11·회수0·봉인180/170 복원 |
| 현재 잔액13·회수1 | 수집 후 옆방 SavePoint | 잔액15·회수1·원방 미봉인100/100 유지 |

각 사례에서 load사용1, profile 불변, 두 공개DTO 일치 및 재export→별도 child Continue의 전체 roots 일치를 확인했다. [수집 전 복원](audit/u14_independent_002/load_before001_observations/RESULT.json), [수집 후 복원](audit/u14_independent_002/load_after001_observations/RESULT.json). 첫 기대값은12−1=11, 둘째는12−1−1+8−1−1−1=15로 작성 비용에서 계산했다. 저장·checksum·메모리 변조는 없었다.

소스 검토에서 변경된 세 runtime 파일과 호출/복원 소비자를 직접 확인했다. 기존 비용·유예·자격·보상 owner를 유지하고, 시험명 분기·silent normalization·catch-all 복구·중복 권위·새 저장 fallback을 추가하지 않았다. canonical runtime 계약의 방별 조회 및 보충 상한 설명은 구현과 일치한다.

## 조건별 증거와 재사용 한계

[CONDITION_REVIEW](audit/u14_independent_002/CONDITION_REVIEW.json)에 A01–A31,10차원,10IVA 및 실행 node별 판단을 연결했다. 변경된 공개/봉인/복원 경계는 현재 소스와 새 실행으로 확인하고, 무변경 producer는 이전 독립 판단과 byte 동일한 소스·유효 원시 증거를 재사용했다. 원 감사의 A13/producer-consumer 실패는 이번 대상에서 닫혔다.

[원시 증거 재확인](audit/u14_independent_002/EVIDENCE_RECHECK.json):

- 교정 Core99/99에 이전 focused82개가 모두 포함된다. G11 기본4/4·계측4/4와 서버29/29의 nonzero PASS를 확인했다. coverage의 모든 시험명은 해당 원시 PASS에 연결된다. 이번 감사에서 이 시험들을 재실행하지 않았다.
- 전체 Core의154실패 중138은 실제 baseline 실패 집합이고 나머지16은 현재 Core/G11 PASS에 모두 연결된다. 변경 없는 전체 시험 성공은 해당 계약에 한정해 재사용한다. 전체 suite green을 주장하지 않는다.
- 공식85/확장169방의6표본에서 단계 성공, requested-live-bytes baseline 복귀, 중앙값·heap/RSS 최대치를 원시 값으로 재계산했다. 한도는 변경하지 않았다. 측정 범위는 첫방 현현과 전체 정적 지도이며 최종SLO나 모든방 방문 성능이 아니다.
- 승인 host30+10단계의 실제 IO실패/재시도·미확정/확정 복원,12승 성장 및 XP/보상 producer는 변경 없는 메커니즘 범위에서 이전 증거를 재사용했다. 이전 Normal 승인은 이전 executable identity에 귀속한다. 이번 소비자는 진단이며 새 Normal 승인을 발급하지 않았다.
- U13 `u13_independent_002` Complete/High/open0와 U04/U12 선행 근거를 인수했다. U14가 바꾼 U13 공개 조회·이동/복원 경계는 교정99회귀와 이번 실제 소비 경로로 확인했다.

## 새 실행·재현·종료

[정확한 argv·환경·exit·원문](audit/u14_independent_002/EXECUTION_INDEX.json), [binary identity](audit/u14_independent_002/BINARY.json): release build exit0/82.686초; 공개3사례 exit0/5.782초; 수집 전/후 SavePoint 각1사례 exit0/각0.820초. 기능5사례 실행·5통과·실패0·skip0. 별도 프로세스 Continue는 공개6회, 추가 SavePoint 인수2회 및 이후 child2회다.

저장소 root에서 공개3사례를 재현한다. 출력 경로는 존재하지 않는 새 경로를 사용한다.

```bash
core_enhanced/docs/goals/core_expedition_deterrence_rewards/execution/single_goal/audit/u14_independent_002/u14_reaudit_probe correction-visited /mnt/f/work/simulator/game_resources /tmp/u14-reaudit-repro
```

추가 SavePoint는 `U14_PROBE_INPUTS=/tmp/u14-reaudit-repro`로 같은 binary의 `audit-load /tmp/u14-reaudit-repro/collected/saved.json <새출력> 11 0` 및 `audit-load /tmp/u14-reaudit-repro/collected/returned_saved.json <다른새출력> 15 1`을 사용한다. 원문 output은 audit 디렉터리에 보존했다.

공식 네 고유보스/최종보스 본체, U16 전체 활동, Unity·배포·전체 공식 균형 플레이·최종SLO와 U11 FacilityLayout/Abnormality changed-mod 계측 미지원은 기존 제외/한계로 유지한다. private issuer key는 인수하지 않았다. 범위 안 필수 미실행·열린 차단 finding은 없다.

이번 single Goal의 독립 완료 판정은 끝났다. 별도 GEX-30은 필요하지 않다. 감사 stage-run의 경계에서 종료하며 후속 U15를 선택·초기화·구현하지 않았다. [감사 closure](audit/u14_independent_002/AUDIT_CLOSURE.json)를 현재 완료 근거로 사용한다.
