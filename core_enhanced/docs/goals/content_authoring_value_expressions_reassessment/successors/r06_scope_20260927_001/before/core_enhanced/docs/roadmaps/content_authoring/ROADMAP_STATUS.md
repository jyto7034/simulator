# Simulator 개발 현황 — 단일 진입점

2026-09-27 · v1.92 · 현재 개발 주작업: **R06**.

개발을 재개할 때 이 문서에서 시작한다. 다음 작업과 전체 진행표는 여기서 안내하고, 상세 설계·실행·감사 원본은 링크로 확인한다.
문서 정리나 후속 기능 논의를 했다는 이유로 개발 주작업을 바꾸지 않는다. 사용자 최신 지시가 우선한다.

## 현재 작업

| 항목 | 현재 기준 |
| --- | --- |
| 개발 주작업 | R06 · ValueExpr/집계 · WP02 부분 구현 |
| 이번 문서 작업 | 모델을 구현하며 개선하는 기준안으로 명시하고 작성 형식/내부 모델의 경계를 보완. 게임 구현·제품 시험 없음 |
| 다음 개발 작업 | **스킬 모델 기준안과 기존 동결 계약의 차이 대조 → R06 잔여 D04 범위·계획 조정** |
| 진행 방식 | 이 대화에서 검토·설계, 실제 구현은 별도 대화 |
| 후속 후보 | 실험실·모드 검증·안전 설정/관측. 별도 계획을 보관하며 현재 착수 대상으로 선택하지 않음 |

## 다음 작업

사용자는 스킬 모델을 설계·검토하되, 현재 안을 최종 동결하지 않고 실제 구현·검증 과정에서 계속 개선하기로 했다.
[스킬 모델 기준안](../../skills/skill_authoring_model.ko.md)에 구성·실행 의미·전용 기능 계약과 대표 7종의 정적 대조를 모았다.
v0.2는 15개 분해 축과 대응하며, 직원은 고정 배치하고 전술 이동·교대로만 위치를 바꾸는 사용자 결정을 반영한다.
v0.3은 구현하면서 개선하는 기준안이라는 지위를 명시한다. 전체 모델의 영구 확정을 구현 선행 조건으로 추가하지 않는다.
다음에는 모델과 기존 동결 계약의 차이를 기록하고, D04의 입력·값/소유자·집계 기준·평가 시점·소비 경로를 대조한다.
필요한 범위·계획 변경을 반영한 뒤 구현 지시서를 작성한다. 이번 모델 설계는 새 구현 지시나 D04 완료 판정이 아니다.
이후 D07 preview/explain, WP03·WP04 및 전체 검증이 남는다.
다른 R이나 실험실 작업으로 자동 전환하지 않는다. MS01 완료는 R06 재개의 선행 조건이 아니다.

### R06 재개 근거

1. [현재 계획 인계](../../goals/content_authoring_value_expressions_reassessment/PLAN_HANDOFF.ko.md): `GS-CAVER-P2G50-001 / GOAL-CAVER-01`, `HA-CAVER-P2G60-001`. **계획 당시** 기록이며 최신 구현 상태는 아래 근거와 함께 읽는다.
2. [D04 구현 인계·잔여 범위](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/segments/wp02_d04_resolved_target_001/REVIEW_HANDOFF.ko.md).
3. [D04-F01 교정 독립 검토](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/reviews/wp02_d04_f01_correction_001_review_001/REVIEW.md): 교정 기능 PASS. WP02/R06 전체 완료가 아니다.
4. [이후 비용 정책 변경](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/R06_COST_POLICY_UPDATE_20260927.ko.md): 약9MB concurrent peak 초과만의 개발 차단을 경고·관찰로 전환. 당시 실패와 감사 판정은 보존한다.
5. [상세 설계 원본](../../goals/content_authoring_value_expressions_reassessment/CODE_REVIEW_003.ko.md): 잔여 설계와 현재 구현의 차이를 확인할 때 사용한다.
6. [콘텐츠 작성 체계 검토·후속 공개 범위 합의](../../goals/content_authoring_value_expressions_reassessment/execution/single_goal/handoffs/AUTHORING_SYSTEM_CONTENT_REVIEW_20260927.ko.md): §9–10이 후속 사용자 합의다. 본편 전용 기능을 허용하고 모드의 작성 보장은 공개 기능 범위로 한정한다. 기존 공식 콘텐츠 교체 권한은 유지한다.
7. [스킬 작성·실행 모델](../../skills/skill_authoring_model.ko.md): 현재 모델 설계의 기준 문서. 래티씨아는 공격 가능한 친구만 공격·소비하고 나머지는 유지하는 사용자 결정을 반영했다. 예시는 설계용이며 현 parser/실행기 지원을 뜻하지 않는다.

구현·검토 근거의 시험 결과는 당시 target에 귀속한다. cleanup과 후속 문서 검토는 제품 시험·동결 source 현재성·선행 감사를 재실행하지 않았다.

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
| R06 | [ValueExpr·집계·Attack/SpellPower와 무기 성장](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R06.ko.md) | **진행 중 · WP02 부분 구현** | R01, R04, R05 | — | [재개 근거](#r06-재개-근거) |
| R07 | [파편 사본·개화 선택·정상 획득/장착/저장 동시 전환](reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R07.ko.md) | 후속 범위 · 완료 미확인 | R01, R05 | — | — |
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

스킬 모델 우선 설계 순서를 채택해 모델 기준안 한 개를 작성했다. 대표 7종과 경계 시나리오 18개를 정적으로 대조하고,
래티씨아의 무대상 node 보존을 사용자에게 확인했다. 다음 작업은 동결 계약과의 차이 대조이며 추가 제품 구현은 진행하지 않았다.

콘텐츠 설계 노트 전체를 검토하고 본편 전용 기능과 공개 작성 기능을 분리하는 사용자 합의를 기록했다.
상세 근거는 위 재개 근거 6번에 연결한다. R21/R22의 표현력·재조합 인수 기준도 향후 이 경계에 맞춰 구체화한다.
R 순서·의존성과 동결 계획/증거는 이번 문서 변경으로 수정하지 않는다.

로드맵 cleanup으로 개발 주작업/이번 문서 작업/다음 개발 작업을 구분하고 R06 재개를 복구했다.
R 순서·의존성·동결 자료는 보존하고, 배포용 파일 6개를 아카이브했으며 과거 기록 12건을 이력으로 옮겼다.
[검증 결과](archive/cleanup_20260927/VALIDATION.json)는 링크·보존·문서 구조 확인이며 제품 시험 결과가 아니다.
