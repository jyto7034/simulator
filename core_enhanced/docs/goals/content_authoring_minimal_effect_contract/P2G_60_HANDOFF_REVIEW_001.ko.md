# 최소 작성·실행 지원 계약 — P2G-60 인수·인계 검토

2026-09-09 · Codex · NEXT-0008. 이 문서는 P2G의 외부 요청·검토 기록이다. 독립 Exact Artifact Ref, 구현 준비 승인 또는 GEX 감사가 아니다.

## 1. 요청과 배치 인수

사용자 요청 원문:

> core_enhanced/docs/roadmaps/content_authoring/ROADMAP_STATUS.md를 읽고
> 다음 단계를 진행해줘. 첫 인수에서 문서 배치와 P2G/GEX의 적용 버전도
> 확인하고, 작업 결과와 근거를 기록한 뒤 현황 문서를 갱신해줘.

시작 시 ROADMAP_STATUS v1.8 기록 0009의 **NEXT-0008**을 선택했다. `execute / bounded_pipeline`, 기존 `WPD-CAMEC-20260909`의 같은 최소 정책 범위에서 필요한 문서 수리·안전한 진단과 P2G-60 exit까지다. 허용 변경은 Handoff·필수 검토/결과/검증 원본 및 Working Policy/현황/history다. production·RON·store·검증기·canonical 정책·AGENTS·P2G/GEX 규칙 수정, 새 정책/범위/위험/권한 대리 선택, Goal activation·GEX 실행은 제외한다. 현재 사용자 요청에서 다음 GEX 카드까지 실행하지 않는다.

workspace `/mnt/f/work/simulator`, branch `cleanup/simulator-dirty-worktree-20260803`, HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`, dirty=true를 인수했다. 시작 Git 항목은 16,523개, 비교 경로는 1,704개다. [NEXT-0007 종료 검사](NEXT_0007_VERIFICATION.json)와 누적 비교 경로에 변화가 없었다. 관련 문서 디렉터리 및 상위 경로에서 적용 AGENTS는 발견하지 않았다. 기존 dirty 구현의 소유권·실행 권한을 P2G가 새로 부여하지 않는다.

원본 배치는 기존 Topic `docs/goals/content_authoring_minimal_effect_contract`를 유지한다. 현황·최근 요약은 `docs/roadmaps/content_authoring`, 과거 요약은 그 `history`에 둔다. `docs/tools`에는 기존 5개 파일만 있으며 정식 P2G-60 전용 CLI는 없다. [운영 규칙](../../roadmaps/content_authoring/OPERATING_RULES.md)의 분리를 적용하고 새로운 runner·검증기·GEX execution 디렉터리를 만들지 않는다. 기존 Goal ID의 `docs/goals` 참조 검색도 현재 Topic의 P2G 문서 4개뿐이며 별도 GEX 실행 이력은 발견하지 않았다.

적용 [P2G r2](../../policy_to_goal_pipeline_revised_v2.md)는 `2026-09-09-r2`이며 배포 사본과 같은 SHA-256이다. [실제 배치된 GEX](../../goal_execution_pipeline%281%29.md)는 `2026-09-09-gex-r1`, 과거 감사가 사용한 [archive](../../archive/goal_execution_pipeline.md)는 별도 해시로 보존한다. 네 파일은 이전 종료와 동일하다. P2G의 옛 `goal_execution_pipeline.md` 링크 부재와 새 파일의 물리적 배치를 구분한다. runner/외부 host가 이 revision을 실제 적용한다는 검증은 하지 않았다.

기존에 읽은 Reader·§5.1/8.0/8.5와 unchanged 운영 규칙을 재사용하고, P2G-60/§8.3/Handoff template를 확인했다. GEX는 실제 소비 계약 §3의 P2G-60 인수 표와 Reader·GEX-00·§6.6/6.7을 다음 작업 경계 설명용으로 읽었다. 이는 GEX 인수 pass를 실행한 것이 아니다.

## 2. Entry와 이전 결과의 재사용

[P2G-50 complete result](P2G_50_RESULT_001.json)의 current Working Revision/Freeze/Phase Plan/Goal Set과 [표준 구조 검증](P2G_50_VALIDATION_001.json)을 인수했다. Goal Set은 `GS-CAMEC-P2G50-001`, 유일한 Single member는 `GOAL-CAMEC-01`이다. `Authored / Not Activated / Implementation Authorized: No`를 그대로 유지한다. 기존 Handoff는 없으며 최초 작성이다.

검증에 사용한 Goal/Freeze/Plan/Receipt·reached JSON source·도구 세 파일·Node v24.11.0과 기존 31개 review source가 동일하므로 P2G-50의 실제 exit 0 / passed / failures empty 및 추가 acceptance 대조 결과를 재사용한다. P2G-40 Receipt와 그 근거도 unchanged다. 이 재사용을 이번 표준 명령 재실행 또는 새 게임 테스트로 기록하지 않는다. Handoff는 기존 입력을 가리키는 후속 artifact이며 upstream policy/Plan/Goal payload를 바꾸지 않는다.

기존 CRAR Master `Complete / High`와 target 해시 대조 이력은 Review/Goal의 기존 선행 근거다. 새로운 GEX 감사나 이번 실행 eligibility 판단으로 승격하지 않는다. 실제 publication·SkillId·executor 재사용과 AG 공동 전환 → WP 03 통합 검증 순서는 [Code Review §3–5](CODE_REVIEW_001.ko.md) 및 current Plan/Goal을 인수한다. 후반 모든 CA의 코드를 다시 조사하지 않았다.

applicable Change/active obligation은 비어 있고 upstream 의미/Phase/Goal을 수리할 새 blocker는 발견하지 않았다. P2G-00/10/20/30/40/50을 다시 실행하거나 과거 원본을 stale/current로 재설정하지 않는다.

## 3. Handoff 의미와 GEX 소비 대조

[Handoff](P2G_60_HANDOFF_001.json)는 exact Result Composite Basis와 Freeze/Plan/Goal Ref를 인계한다. policy/acceptance·Work Package·Phase·Goal member payload는 각각 원본에서 읽으며 Handoff에 복제하지 않는다. 파일 경로·SHA-256은 탐색과 bytes currentness를 위한 외부 source 식별자이고 P2G Content Digest를 새로 계산한 값이 아니다.

| 소비할 항목 | 인계 판단과 원본 |
| --- | --- |
| policy/Plan/Goal identity | P2G-50 current Composite Basis를 그대로 결속. Handoff 자기 Ref는 payload 바깥에 두며 이전 Handoff는 N/A다. 필수 결과 검증 뒤에만 current 선택한다. |
| 정책에서 Goal까지 추적 | 6개 reviewed PER·6개 DR·12개 acceptance의 source 관계와 Phase primary selector→단일 Goal mapping, final node 2개·IVA 6개의 owner를 대조한다. Handoff는 해당 Goal Set Ref로 이 관계를 인계하며 별도 Matrix를 만들지 않는다. |
| 구현 전 상세 설계·준비 | Freeze KU30-01과 Goal member의 Boundary/Execution Prerequisites/Validation Plan/Command and Changed Input-to-Seam Map을 연결한다. 정확한 타입·문법·locator·지원 조합·실제 fixture 및 producer/consumer 정합성 점검은 미완료 구현 활동으로 남긴다. |
| 검증 근거 재사용 | 이 Topic의 user-approved Shared Validation Baseline Reuse Directive는 N/A. Goal의 기존 input→seam 영향 map 및 GEX §6.4를 소비하며 다른 Topic의 initial/final 횟수 계약을 추가하지 않는다. 기존 성공은 원래 시점/입력에만 결속한다. |
| 정책·상태·외부 제한 | KU30-02–05, Goal out-of-scope·Consumer Boundary와 [P2G-50 작성 검토 §2](P2G_50_AUTHORING_REVIEW_001.ko.md#2-재사용한-entry와-코드-근거)의 저장 버전 문서/actual 문제를 인계한다. 콘텐츠 수치/태그 초안·사본/generation·고급 변형·외부 host/runner·과거 대화 획득을 해결됐다고 표시하지 않는다. |
| 저장 경계 | 기존 V5 writer/V4·V5 reader와 canonical 저장 버전 문구의 불일치는 별도 저장 owner 문제다. 현재 최소 Goal은 저장 포맷 변경·reader 제거·개인 파일 삭제/자동 변환을 제외한다. 이후 해당 경계를 바꿔야 하면 원래 owner/정책 반환 조건을 적용하며 현재 Handoff 작성의 blocker로 확대하지 않는다. |
| 위험·검토·실행 권한 | Goal의 기존 위험/검토 조건과 current GEX가 owner다. Handoff는 Goal activation·구현 승인·production mutation·completion을 승인하지 않는다. GEX-00은 향후 exact goal-run 요청, current dirty/실행 문서/선행·검증 가능성을 다시 인수해야 한다. |

이 검토에서 Handoff 작성 범위를 막는 새 정책·권한·필수 근거 문제는 없었다. 구현 전 조사·설계·게임 시험이 아직 없다는 사실과 Handoff 작성 완료 여부는 구분한다.

## 4. 검증과 종료

P2G-60에는 지침이 요구하는 별도 표준 CLI/Receipt가 없다. 이번 Handoff의 exact reference·source currentness·필수 summary/권한·12개 acceptance trace 관계는 허용된 일회성 문서 진단으로 실제 검사하고 [검증 기록](P2G_60_VALIDATION_001.json)에 방법·입력·결과를 남긴다. 기계 관계 검사와 위 의미 대조는 서로 다른 근거다.

complete result와 current 선택은 [P2G-60 결과](P2G_60_RESULT_001.json)가 소유한다. 최종 dirty/불변 원본·링크 보존은 [NEXT-0008 종료 검사](NEXT_0008_VERIFICATION.json)에 기록한다. 원본 기록 이후 Working Policy/현황을 갱신하고, 오래된 기록 0005를 history로 이동한다.

다음 NEXT-0009는 `(GS-CAMEC-P2G50-001, GOAL-CAMEC-01)`의 GEX `goal-run` 요청용 카드다. 다음 요청에서 GEX-00 인수 후 해당 Goal의 상세 설계·구현·검증과 fresh GEX-20을 분리해 `Complete / High` 또는 중단 조건까지 진행할 대상을 명시한다. 이번에는 그 카드·Goal activation·GEX 실행·게임 테스트·production/RON/store 변경을 수행하지 않는다.
