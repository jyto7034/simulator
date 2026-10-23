# U10 실제 사건 기반 유물 연쇄와 장비 효과 통합 — Working Policy

- Working Policy Document ID: WPD-CRCE-20261009
- Policy Directory Name: core_relic_causal_chains
- Current Working Revision Ref: WR-CRCE-003
- Current Policy Intake Snapshot Ref: PIS-CRCE-001
- Current stage: P2G-60 complete; GEX-00 baseline intake
- Requested Result: 승인된 연속 범위의 P2G-00→60, 이후 별도 GEX 구현·검증·독립 감사와 로드맵 후속
- Target repository: simulator / `/mnt/f/work/simulator`

초기 immutable payload와 exact refs는 [P2G_00_INITIAL_001.json](P2G_00_INITIAL_001.json)에 있다. [원문 보관](INTAKE_SOURCE_001.tar.gz)은 사용자 제공 locator 및 후속 정책·코드 검토에 필요한 원문 bytes를 보존한다. [초기화 결과](P2G_00_RESULT_001.json)는 생성 경계만 기록한다.

U09 target038 전체 독립 Complete/High/open0와 Root 완료 게시를 선행 입력으로 인수했다. 사용자 연속 지시는 단계 전환의 별도 승인 대기 없이 다음 로드맵 범위의 정책·계획·구현·시험·독립 감사를 허용한다. 이 초기화 자체가 제품 구현 준비나 정책 의미 검토 완료를 뜻하지 않는다.

승인 범위는 활성 ROADMAP U10/C07/C08의 실제 사건 기반 유물 인과·장비 효과 통합이다. 구체 in/out 항목과 승인 근거는 초기 Working Revision의 Scope Items에 있다. Core 우선·Unity 후속·개별 최종보스 제외, 기존 유효 증거·실패 원본 보존과 변경 영향에 따른 검증 원칙을 유지한다.

현재 14개 정책 요소와 결정은 [P2G-10 immutable 산출물](P2G_10_POLICY_001.json), 17개 candidate 처리와 정확한 근거·종료 검토는 [P2G-10 결과](P2G_10_RESULT_001.json)에 있다. 미결 질문은 없다. 같은 사본/효과 방문, fork/실제 join, 독립 주기/파생 지연, 로그 독립 수명, 장비 counter/death/suppression, 공명 잠금, 쐐기 실제 strike, 자원·공개 RON·복원을 포함한다.

쐐기의 미정 유효 적중은 H04의 범위 내 gameplay 위임으로 **최종 HP 감소가 양수인 실제 직접 strike**로 결정했다. 완전 보호막 흡수·무적·0 피해는 누적을 소비하지 않는다. 이번 유효 타격부터 n×r을 적용하고 같은 strike의 여러 segment는 한 번만 센다. 이는 사용자 직접 답변이나 확정 밸런스가 아닌 AI 위임 결정이다. r=10%p는 검증 예시로 유지한다. 기존 초기 revision과 원문은 보존한다.

[실제 코드 검토](CODE_REVIEW.ko.md)와 [P2G-20 complete result](P2G_20_RESULT_002.json)를 발행했다. 14개 요소의 statement/acceptance를 유지한 code_reviewed successor 및 하나의 WR003에 모든 binding을 결속했다. 현재 구현의 부재를 delta로 기록했고 제품·시험 코드 변경과 시험 실행은 없다. 다음은 P2G-30 정책 동결이다.

P2G-20 current batch는 RB-CRCE-002다. 기존 비정규 finding 분류 이름을 `implementation_finding`으로 바로잡아 동일 WR003 정책과 현재 source bytes를 재결속했다. 원본 RB001과 검토 증거는 보존하며 실행 재검사는 하지 않았다.

Current planning Freeze: FPF-CRCE-001. [동결 결과](P2G_30_RESULT_001.json)는 WR003/RB002의 14개 current member·binding과 28개 acceptance 문장을 그대로 선택한다. 다음은 구현 경계와 독립 검증을 묶는 P2G-40이다.

[Phase Plan 결과](P2G_40_RESULT_001.json)는 실제 생성·검증 exit0/passed/failures0다. 하나의 Phase·3개 실행 node·19개 통합 검증 owner를 인수했다. Single Goal candidate 작성 후 P2G-50 구조 검증을 진행한다.

[P2G-50](P2G_50_RESULT_001.json) 구조 검증과 [P2G-60](P2G_60_RESULT_001.json) 인계를 완료했다. 별도 기존 사용자 연속 권한으로 [GEX 실행 계획](execution/single_goal/PLAN.md)의 복원 기준선을 인수하고 구현·독립 시험을 이어간다. 제품 동작은 아직 미검증이다.
