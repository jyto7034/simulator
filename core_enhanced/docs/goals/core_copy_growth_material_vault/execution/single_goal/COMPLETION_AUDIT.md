# U15 독립 완료 감사 — Complete / High

2026-10-14, GEX-20 stage-run. GS-CGMV-001 / GOAL-CGMV-01 / PHASE-CGMV-01.

**판정: Complete / High. 열린 완료 차단 0건, 확인된 제품 결함 0건.** 21수용 조건·10검증 차원·5통합 조건·3실행 node를 검토했다. 귀환 IO 검증의 증거 오류 1건을 발견했고, 고정 제품에 대한 독립 실행으로 해당 검증 공백을 닫았다. U16은 시작하지 않았다.

## 권한과 대상

사용자의 상시 지시서 선독 후 U15 독립 감사 요청에 따른 별도 감사 pass다. 이번 대화의 실행자는 U15 구현·교정에 참여하지 않았다. 서브에이전트나 자동 위임 없이 수행했으며 모델 전환을 주장하지 않는다. 제품·기존 시험·동결 자료는 수정하지 않았다.

- 기준: FPF-CGMV-001, PP-CGMV-002, [Goal Set](../../P2G_50_GOAL_SET.ko.md)의 Completion Matrix, 현행 상시 지시서, GEX gex-r8의 GEX-20 및 로드맵 §1.9.
- [고정 구현 대상](target/u15_current_001/AUDIT_TARGET_RECORD.json): `u15_current_001`, HEAD `2f725172edbffd27c5078d29000a1a2ba6ddc4d9`.
- SOURCE SHA256 `56a83a8c85ea7d1280f14c539ffba6249160add974285d3d58074b8a05e3e4cb`.
- [감사 대상 기록](audit/independent001/AUDIT_TARGET_RECORD.json), [입구](audit/independent001/ENTRY_CURRENTNESS.json), [출구](audit/independent001/EXIT_CURRENTNESS.json): 소스780·증거751 일치, current / unchanged. 세 production root의 누락 파일도 없다.
- SOURCE archive와 manifest의 전체 bytes·member equality 및250개 delta의 before/after identity를 독립 검산했다. 구현자가 실행한 격리 정역복원은 이 일치 근거로 재사용했다. 작업트리 patch 적용0.

동결 PLAN/EXPERIMENTS/COVERAGE와 GEX10 결과의 “감사 대기”는 구현 인계 당시 상태다. 아래의 증거 오류와 현재 판정은 이 감사 기록이 소유하며 과거 원문을 덮어쓰지 않는다.

## E-U15-001 — 귀환 IO 증거 오류와 closure

[Finding 기록](audit/independent001/FINDINGS.json). **범위 안 — 지원 귀환 명령/실제 저장 IO 장애**에 해당한다.

기존 `test_parts/host_replays001/phase4.json`은 `fail_index:29`와 `reject_indices:[29]`를 함께 지정한다. 29번 요청은 파츠3개 귀환으로 허용량2를 넘으므로 저장 이전에 `InvalidAction`으로 거절된다. 실제 [원시 IO_FAILURE](runs/normal_host001/phase4/IO_FAILURE.json)도 filesystem 오류가 아닌 domain 오류다. 기존 host runner는 오류 종류를 구별하지 않아 이를 IO 성공 검증으로 기록했다.

따라서 원래 host7프로세스/165명령의 다른 결과는 유효하지만, **실제 IO 실패·재시도는 혼합개화와 SavePoint의2건**이다. 기존 문서의3건 주장과 A17의 귀환 IO 근거를 그대로 재사용하지 않았다.

감사 전용 [진행기](audit/independent001/check_return_io.py)는 새 빈 paired store에서 정상 공개 동선을 실행하고, 별도 입력의 `fail_index`를 유효한 귀환 요청30번으로 지정했다. 해당 요청은 파츠2개와 부활촉매1개다. 원래29번 과다 회수 거절 대조는 유지했다. frozen host binary SHA256은 `eaec47bcb5eebd5461363bfaa9468263e248086d1c3fa2e12cfbfa16d1221c68`로 확인했다.

| 독립 실행 | 실제 관측 |
| --- | --- |
| phase1 | 공개 공급·첫 귀환으로 창고를 생성 |
| phase2 | 혼합개화 저장에서 실제 `os error 21`; 모든 memory/disk root 불변; 같은 요청 재시도 성공 |
| phase3 | 새 프로세스 미확정 Continue, SavePoint 복원 저장의 실제 `os error 21`; 불변·재시도 성공 |
| phase4 | 완료 상태의 새 프로세스 Continue, 유효2+1귀환 저장의 실제 `os error 21`; 불변·재시도 `run_complete` 성공 |
| phase5 | 저장된 귀환을 새 프로세스에서 읽고 재대여; 무기투자0·파편투자 유지·한번 환급·반복분쇄 거절 확인 |

[첫4프로세스 결과/argv](audit/independent001/return_io001/RESULT.json), [유효 귀환 실제 IO](audit/independent001/return_io001/phase4/IO_FAILURE.json), [후속 새 프로세스 결과](audit/independent001/return_io001/phase5/RESULT.json), [후속 argv/exit](audit/independent001/return_io001/phase5.process.json). 총5프로세스·133명령, exit0/실패0/skip0, 실제 IO 실패·재시도3건이다. 제품 수정 없이 A17/A20의 공백을 닫았다. 원래 잘못된 사례를 새 IO 성공으로 소급 표기하지 않는다.

## 조건별 검토

상세한 [독립 조건표](audit/independent001/CONDITION_REVIEW.json)에 A01–A21 각각의 소스·판단·원시 증거와10차원/5IVA/3실행 node를 연결했다. 최초 감사이므로 구현자의 결론을 이전 독립 판단으로 취급하지 않았다.

| 조건 | 직접 확인한 계약과 근거 |
| --- | --- |
| A01–A04 / G20 | loader와 실행 모두 무기만 강화. recipe 중복재료 checked 합산, 최대단계, 전체 자원 충족 후 candidate commit. 형태선택/이전/재개화는 추가 투자0. 정비 반복은 입장비용을 재호출하지 않고 Safezone에 정비를 추가하지 않음 |
| A05–A08 / G21 | 원장 재료별 실제 누적합의50%내림. 무기9→4, 파츠5→2/3→1; recipe 변경으로 재평가하지 않음. 장착확인·해제/소멸/보상 원자성, 중복분쇄 거절. 귀환 무기reset·파편 재대여 원장 유지 |
| A09–A13 / G22 | 명시출처가 요구집합·수량과 정확히 일치해야 함. 기존 명령은 소지 전용. query 불변과 실제 결제 일치. 귀환 parts2+catalyst1, 빈 선택·실패/포기 차이. 부활촉매 공통포트까지만 검증 |
| A14–A18 / STORE | 창고 현재잔액+현재원정소비−목표소비. checkpoint의 사본/투자/소지와 함께 복원, 이전소비·무관창고·단조지식 보존. 새 프로세스 미확정/완료 Continue, 실제 IO와 재시도, 구version5 명시거절·원본4파일 보존 |
| A19–A21 / AUTHOR | 공개 외부 RON valid1/작성오류9, Core/정상 승인host의 공개 명령·조회·paired store·재시작. 33개 cost process의396단계와 실제 할당량 회수; 기존 한도 유지 |

`material_economy.rs`와 maintenance의 실제 호출, copy/grant/loadout, lifecycle/SavePoint/Continue, 공개 route/query, profile/active/journal writer 및 실패 경계를 검토했다. 재료 지불·원장·창고의 owner를 분리하고 application candidate가 전체 결과를 수락한다. `equipment_dust`는 RunCraftingResources 단일잔액이며 typed/generic grant와 강화/환급/조회가 연결된다. RunProgression의 native map은 필드별 clone shape로 계량된다.

SavePoint의 장기 소비차이 복원과 정확히 한 번의 미수락 transaction rollback을 구별했다. 전자는 전체 profile을 되돌리지 않고, 후자는 lifecycle 자산과 run을 복원하면서 단조 지식을 유지한다. 정상 호스트는 `ApprovedPersistence::publish`에서 paired 저장 수락 뒤 prepared candidate를 commit한다.

기존 inventory/copy/UUID/loadout/저장 authority를 재사용한다. 시험명 전용 제품 분기, 비용 자동완화, 묵시 출처대체, 구투자 추정·default 이행, 새 중복 자산엔진이나 catch-all 복구는 확인되지 않았다. 현재 writer16/profile6/journal4 및 공개 포트는 canonical runtime/data 계약과 일치한다.

## 재사용·한계와 재현

[원시 증거 검산](audit/independent001/EVIDENCE_RECHECK.json)은 기존 Rust 로그의 실제 시험명·결과, host 명령, 환급 산술, 외부 제작 결과, 구저장 archive 및 비용 원시값을 확인했다.

- Core 고유 통과2180은 전체 실행의2097 library+72 integration에 교정6/G11기본4/socket1을 결속한 coverage다. focused37은 중복 합산하지 않았다. 전체 원실행은 실패6을 포함했고 그6개 모두 focused에서 통과했다. 기존 ignore6을 유지하며 단일 전체 green 실행을 주장하지 않는다.
- 별도 native G11 4/4와 server29/29, 공개 consumer6프로세스, 기존 host7프로세스의 유효 부분을 재사용했다. Rust suite와 비용 실행을 감사 단계 이름만으로 반복하지 않았다.
- 비용 raw9+18+6개 모두12단계 성공, 최종 requested-live-bytes가 baseline으로 복귀했다. 실제 heap/시간·RSS와 내부 native/retained 회계를 구분했다. dev 진단·유한 확장 입력이며 release/device SLO나 공식 전체 플레이 완료가 아니다.
- U14 independent002는 역사적 선행 판정이다. U15의 copy/schema/lifecycle/dust 변경은 이번 직접 검토와 현재 U14 회귀·host·비용 증거로 확인했다. 이전 U14 감사가 U15를 자동 승인한 것으로 표기하지 않는다.
- 외부 임의 저장/checksum/메모리 변조 전용 사례는 추가 finding이나 필수시험으로 만들지 않았다. 지원 제작 오류·공개 명령 오용·실제 IO 장애는 계속 검증했다.
- 감사용 로그 집계 첫 실행은 `should panic` 라벨12개를 빠뜨렸다. raw 성공 수를 바꾸지 않고 parser만 교정하여2180을 확인했다. [실패 보존과 교정 근거](audit/independent001/AUDIT_TOOL_CORRECTION.json).

재현 명령은 저장소 루트에서 실행한다. 첫 명령의 output은 존재하지 않는 새 경로여야 한다.

```bash
python3 core_enhanced/docs/goals/core_copy_growth_material_vault/execution/single_goal/audit/independent001/check_return_io.py /tmp/u15-return-io-new-output
python3 core_enhanced/docs/goals/core_copy_growth_material_vault/execution/single_goal/audit/independent001/recheck_evidence.py /tmp/u15-evidence-recheck-new.json
```

후속 phase5는 `return_io001/phase5.process.json`의 argv에 기록했다. 재현 시 새 실행의 stores/output 경로를 사용한다. 정상 host 실행에는 동결 admission config가 참조하는 기존 승인 환경이 필요하며 private issuer 키는 감사 증거에 포함하지 않는다.

D02 최종 밸런스, U17 실제 부화, Unity/고유 최종보스 및 배포장치 SLO는 범위 밖이다. 이 single-phase Goal은 별도 GEX-30 없이 이번 판정으로 완료한다. 사용자 요청은 U15 감사에서 종료하며 U16 선택·초기화·구현 권한을 확대하지 않는다.
