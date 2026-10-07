# CA01 최소 작성·실행 지원 계약 — Working Policy

Document Identity: `WPD-CAMEC-20260909`  
Policy Directory: `content_authoring_minimal_effect_contract`  
Current Working Revision / Checkpoint: `WR-CAMEC-P2G20-001`  
Current Stage: P2G-60 completed; NEXT-0008 종료  
Current Planning Freeze: `FPF-CAMEC-P2G30-001` (selected; lifecycle `current`)  
Current Phase Plan: `PP-CAMEC-P2G40-002` (lifecycle `current`); Receipt `PVR-CAMEC-P2G40-002` passed  
Current Goal Set: `GS-CAMEC-P2G50-001` (lifecycle `current`); Single member `GOAL-CAMEC-01`; `Authored / Not Activated / Implementation Authorized: No`  
Current Handoff Artifact: `HOF-CAMEC-P2G60-001` (lifecycle `current`; GEX 인수·구현 권한은 부여하지 않음)  
P2G-10 Completion Predicate: true (P2G-10 결과 참조)

## 요청·범위·원본

현재 요청은 현황 문서를 읽고 다음 단계를 진행해 달라는 사용자 요청이며, 시작 시 ROADMAP_STATUS v1.8 기록 0009의 NEXT-0008을 선택했다. Last User Stage Request와 종료점/권한은 [P2G-60 결과](P2G_60_RESULT_001.json)의 request 및 [인계 검토 §1](P2G_60_HANDOFF_REVIEW_001.ko.md#1-요청과-배치-인수)가 소유한다. 같은 최소 정책 범위의 `bounded_pipeline`으로 P2G-60 Handoff 작성·검증까지 완료했다. 이번 권한은 Topic 문서·안전한 진단·현황 갱신이며 검증기·게임 코드/RON/store·canonical 정책·P2G/GEX 규칙 수정은 포함하지 않는다. Goal activation·구현 승인·GEX는 수행하지 않았다.

승인된 정책 범위와 NEXT-0003 당시 요청·선택 카드는 [초기 Working Revision payload](P2G_00_INITIAL_001.json)의 Approved Metadata and Scope가 소유한다. 그 Requested Result/O03의 P2G-20 제한은 당시 요청의 기록이다. 이후 요청은 같은 정책 범위에서 NEXT-0004의 P2G-30, NEXT-0005/0006의 P2G-40, NEXT-0007의 P2G-50, NEXT-0008의 P2G-60으로 실행 종료점을 확장했다. 기존 Working Revision·Decision·acceptance는 변경하지 않았다. 원문 상태를 의미 판정 없이 고정한 `PIS-CAMEC-P2G00-001`도 같은 파일에 둔다.

[초기화 결과](P2G_00_RESULT_001.json): 미점유 경로·미사용 identity, 필수 payload·참조·명시적 빈 map/set 검사 통과. P2G-00 시점에는 Policy Element·Decision·Question이 없었다. 기존 CA00 및 CRAR Topic/감사 기록은 재초기화하지 않았다.

## 지침·변경 보존

적용 P2G는 [저장소 r2](../../policy_to_goal_pipeline_revised_v2.md), revision `2026-09-09-r2`다. 배포 사본과 SHA-256이 같음을 확인했다. GEX는 [배치된 gex-r1](../../goal_execution_pipeline%281%29.md)을 관찰했고 기존 감사 basis는 [archive](../../archive/goal_execution_pipeline.md)에 있다. 이번은 P2G 작업이며 GEX 실행은 하지 않는다.

workspace `/mnt/f/work/simulator`, branch `cleanup/simulator-dirty-worktree-20260803`, HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`와 기존 dirty 변경을 인수했다. NEXT-0008 시작 Git 항목 16,523개, 비교 경로 1,704개이며 이전 종료 대비 변화는 없었다. 문서 배치는 기존 Topic 원본·현황/history view를 유지하고 적용 AGENTS는 발견하지 않았다. [NEXT-0007 종료 검사](NEXT_0007_VERIFICATION.json)의 Goal·도구·source 근거를 재사용했다. 이번 source·지침·도구 currentness와 보존 결과는 [P2G-60 결과](P2G_60_RESULT_001.json)와 [NEXT-0008 종료 검사](NEXT_0008_VERIFICATION.json)가 소유한다. 새 GEX execution 디렉터리나 실행 이력은 만들지 않았다.

## 현재 정책 basis

[정책·결정 원본](P2G_10_POLICY_001.json)과 [P2G-10 분류·검증 결과](P2G_10_RESULT_001.json)는 6개 요소의 결정 의미와 15개 candidate disposition 이력을 소유한다. 현재 요소 revision map은 아래 P2G-20 Result Working Revision이 선택한다. 기존에 사용자 결정이라고 명시된 U01/U02/U04/U07 및 공통 정책의 효과 태그·새 효과 출처 부분만 인수했다. 과거 대화 원본을 독립 획득한 것은 아니다.

R/F 콘텐츠 초안, CA01/A02 권장 자료구조와 COV의 `합의` 라벨은 포괄 정책 승인으로 사용하지 않았다. 미정 수치·적중 인정·carrier 변형 gameplay·전체 selector 지원은 확정하지 않았다. P2G-20은 기존 상수 Damage/Heal/Instant를 쓰는 최소 계약의 현재 경계와 구현 제안을 검토했다.

## 인수한 코드 검토

[검토 본문](CODE_REVIEW_001.ko.md), [Review Batch·source/evidence·binding](P2G_20_REVIEW_001.json), [P2G-20 complete result](P2G_20_RESULT_001.json)를 선택한다. 6개 요소는 `code_reviewed`이고, 7개 source requirement(필수 workspace 6개, 범위상 N/A 1개)를 평가했다. 31개 source 파일의 명시된 구간을 읽고 source/current reference 및 claim/binding coverage를 검증했다. 원래 Decision·Observable Acceptance는 변경하지 않았다.

NEXT-0003에서 문서 작성과 정적 코드 검토를 완료했다. 기존 상수 실행 경로, 공유 carrier, HP-loss gate/signed Heal, direct consumer와 저장 영향은 검토 본문이 소유한다. 당시 [작업공간 기준선](WORKSPACE_BASELINE_001.json)과 [종료 검사](NEXT_0003_VERIFICATION.json)는 그대로 보존했다. NEXT-0004는 31개 source 및 결정 근거의 해시·구간, exact revision/binding을 대조하여 검토 재사용 조건을 확인했다. P2G-10/20을 새로 실행하지 않았다.

## 인수한 동결 결과

[Final Policy Freeze](P2G_30_FREEZE_001.json) `FPF-CAMEC-P2G30-001`과 [P2G-30 complete result](P2G_30_RESULT_001.json)를 인수했다. 동결 당시 basis는 `WR-CAMEC-P2G20-001` + 이 Freeze였다. 현재는 아래 P2G-40의 current Phase Plan도 선택한다. 6개 member의 intrinsic status는 기존 `code_reviewed` 그대로이고 membership은 complete result에 기록했다. 적용되는 Change/Obligation 및 P2G-30 Resolution Record set은 명시적으로 비어 있다.

동결 대상은 기존 사용자 결정 기록에서 인수한 **6개 최소 정책 요소와 그 12개 수용 기준**이다. 콘텐츠 문서 전체·미정 수치/태그 목록·R/L 구현 제안의 확정이 아니다. acceptance를 추가하거나 변경하지 않았으며, exact member/Decision/Review/Source/Binding의 일치, 29개 요소·필수 claim 조합, 6개 binding과 수용 기준별 검토/검증 경계를 확인했다. 잘못된 후보 6개를 거부하는 문서 검사도 통과했다.

전환 disposition과 남은 unknown은 Freeze 원본이 소유한다. 정확한 작성 문법·타입·source 주소와 구현 시험은 후속 계획/구현 owner, 미정 gameplay는 해당 PG와 P2G-10, 새 사본/세대의 영구 상태는 해당 CA owner가 다룬다. 현재 최소 metadata 계약에 새 저장 포맷을 도입하지 않으며, 기존 V4/V5 reader 존재를 미래 호환 보장으로 해석하지 않는다. 과거 대화 원본의 독립 획득과 외부 환경/runner 적용은 확인하지 않았다.

NEXT-0004는 이 동결 결과에서 종료했다. 당시 [검사 기록](NEXT_0004_VERIFICATION.json)과 불변 artifact는 그대로 보존한다.

## NEXT-0005의 계획 후보와 중단 이력

[계획 후보](P2G_40_PHASE_PLAN_CANDIDATE_001.ko.md)는 Work Package 3개·Atomic Group 1개·최종 실행 node 2개·Phase 1개다. compiler/최신 catalog/runtime의 동시 전환과 durable 통합 fixture의 구현 owner를 배정하고, 6개 selector·12개 acceptance의 검증 연결과 중간 상태를 검토했다. 새 코드나 fixture를 구현한 결과는 아니다.

[표준 생성 시도](P2G_40_VALIDATOR_ATTEMPT_001.json)는 실제 exit 1 / `not_written` / `failed`다. 기존 JSON Freeze를 Markdown으로 파싱하고 stable ID를 `PE-` 형식으로 가정하여 `REF_FREEZE`, `FREEZE_MEMBERS`, `WP_SELECTOR_EMPTY`가 발생했다. Receipt는 생성되지 않았고 기본 검증도 실행하지 않았다. 보조 문서 진단의 관계/graph 검사로 이 필수 gate를 대체하지 않는다.

[중단 원인·교정 범위](P2G_40_BLOCKER_001.ko.md)에 재현 근거와 필요한 권한을 기록했다. NEXT-0005가 검증기 수정을 제외하므로 그 요청에서는 교정하지 않았다. 당시 마지막 유효 Composite Basis는 Working Revision + Freeze였으며 current Phase Plan/Goal은 없었다. 기존 Freeze를 stale로 만들거나 정책 ID를 개명하지 않았다.

이 실패 원본과 [당시 종료 검사](NEXT_0005_VERIFICATION.json)는 현재도 그대로 보존한다.

## NEXT-0006의 P2G-40 결과와 종료

사용자 승인에 따라 최소 자동 검사와 별도 의미 검토의 경계를 [도구 설명](../../tools/README.md)에 정리했다. JSON artifact의 실제 Stable Element ID·명시적 관계를 읽는 공통 해석을 도입하고, 기존 Markdown 처리를 호환 adapter로 분리했다. P2G-40의 생성/검증과 P2G-50의 직접 소비자가 같은 입력 해석을 사용한다. P2G/GEX 규칙과 Goal Quality Contract는 변경하지 않았다.

[JSON Phase Plan](P2G_40_PHASE_PLAN_002.json) `PP-CAMEC-P2G40-002`와 [표준 Receipt](P2G_40_PHASE_PLAN_VALIDATION_RECEIPT_002.json) `PVR-CAMEC-P2G40-002`를 current 결과로 선택했다. 구조·ownership은 이전 후보와 같다: WP 3개, 동시 전환 group 1개, final node 2개, Phase 1개, integration row 6개. 6개 정책의 primary 담당과 12개 acceptance 참조를 보존했다. 그때는 Goal Set이 없었다.

[실행 근거](P2G_40_TOOLING_VALIDATION_001.json)에 회귀 검사 37/37 통과와 실제 표준 생성·기본 검증 `exit 0 / passed / failures []`를 기록했다. 기존 Receipt 4개와 완성된 과거 Goal Set도 도구 호환성 검사에서 통과했다. 게임 시험이나 이 Topic의 P2G-50 실행 결과는 아니다. 설계 의미·current source 확인은 [도구 검토](P2G_40_TOOLING_REVIEW_001.ko.md)와 [complete result](P2G_40_RESULT_002.json)가 소유한다.

B40-001은 해소됐으며 당시 요청의 P2G-40 종료점에 도달했다. 당시 NEXT-0007은 카드만 기록했고 실행하지 않았다. 그 요청에서 게임 구현·RON/store 변경·게임 테스트·GEX 감사는 수행하지 않았다.

## NEXT-0007의 P2G-50 결과와 종료

[Goal Set](P2G_50_GOAL_SET_001.ko.md) `GS-CAMEC-P2G50-001`을 [complete result](P2G_50_RESULT_001.json)에서 current로 선택했다. 기존 Plan의 유일한 `PHASE-CAMEC-01`을 Single Goal `GOAL-CAMEC-01`에 연결했으며 담당 node는 `AG-CAMEC-01`과 `WP-CAMEC-03`이다. 상태는 `Authored / Not Activated / Implementation Authorized: No`다. 당시 Handoff는 없었다.

6개 reviewed PER와 그 Decision·12개 acceptance, IVA 01–06의 검증 owner를 그대로 인계했다. 기존 코드 동작과 목표 delta, 실제 선행·직접 소비자·공유 계약, 상세 설계/준비 점검·실행 이후 교정/재검증을 Goal의 기존 필드에 작성했다. [작성 검토](P2G_50_AUTHORING_REVIEW_001.ko.md)는 이 내용의 의미 대조를 소유한다. 정책·Phase 재분할이나 P2G-10/20/30/40 재실행은 없었다.

[실제 구조 검증](P2G_50_VALIDATION_001.json)은 `p2g50-goal-set-validator/1.3.0 --quality-contract 2`, exit 0 / passed / failures empty다. 1 Goal·2 final node·6개 정책 연결·6 integration·10 validation dimension 및 12개 acceptance member의 완전성을 확인했다. 검증기 회귀 37개는 이전 unchanged 근거를 인수했고 이번에는 재실행하지 않았다. 게임 테스트·GEX 감사 결과가 아니다.

새 API/정확한 타입·문법·locator·실제 fixture 결과는 후속 구현 owner에게 남아 있다. 콘텐츠 초안의 미정 정책/수치와 사본·generation·고급 변형, 외부 consumer/runner actual도 기존 범위를 유지한다. 추가로 읽은 canonical의 저장 버전 서술과 기존 actual의 불일치는 [작성 검토 §2](P2G_50_AUTHORING_REVIEW_001.ko.md#2-재사용한-entry와-코드-근거)에 별도 저장 owner 문제로 기록했다. 현재 최소 Goal은 저장 포맷 변경을 제외한다.

NEXT-0007은 P2G-50 종료점에서 멈췄다. 당시 NEXT-0008은 다음 요청용 카드였으며 실행하지 않았다. 그 요청에서 게임 코드/RON/store·검증기·canonical/지침 변경·P2G-60·GEX 실행은 없었다.

## 현재 P2G-60 결과와 종료

[Handoff](P2G_60_HANDOFF_001.json) `HOF-CAMEC-P2G60-001`을 [complete result](P2G_60_RESULT_001.json)에서 current로 선택했다. P2G-50의 exact current Working Revision/Freeze/Plan/Goal Set basis를 그대로 인계했다. 정책·Phase·Goal member 원문은 복제/수정하지 않았고 별도 Change/repair는 없다.

[인계 검토](P2G_60_HANDOFF_REVIEW_001.ko.md)는 GEX 소비 계약과의 대조 및 남은 상세 설계·실제 시험·consumer/저장/권한 한계의 의미를 소유한다. Handoff의 8개 원본 연결 요약은 KU30-01–05, 새 API/fixture와 검증 미완료, 기존 저장 버전 문서/actual 불일치, 향후 GEX 인수·권한 경계를 유지한다. 상태는 계속 `Authored / Not Activated / Implementation Authorized: No`다.

[실제 문서 검증](P2G_60_VALIDATION_001.json)은 Handoff 필수 필드·exact basis·참조 순환 부재·6개 정책/12개 acceptance→단일 Goal 연결·6개 integration owner·남은 요약·권한 경계를 대조해 exit 0 / passed를 얻었다. 기존 P2G-50 표준 pass는 exact 입력/도구/Node가 같아 재사용했으며 이번에 재실행하지 않았다. P2G-60 전용 표준 CLI/Receipt를 새로 만들지 않았다.

다음 NEXT-0009는 `(GS-CAMEC-P2G50-001, GOAL-CAMEC-01)`의 GEX `goal-run` 요청용 카드다. 다음 요청에서 현재 dirty·선행·실행 문서·검증 가능성을 GEX-00이 인수한 뒤 필요한 상세 설계·구현·검증과 fresh GEX-20을 분리해 진행한다. 그 준비/권한/성공을 이번 P2G가 미리 판정하지 않는다. NEXT-0008은 Handoff 작성 종료점에 도달했으며 새 카드·Goal activation·GEX·게임 시험·production/RON/store 변경은 실행하지 않았다.
