# R06 계획 전용 종료점 변경 인수 — 2026-09-13

현재 사용자 요청은 고성능 모델로 P2G 검토·구현 계획을 수립하고, 그 결과를 인계한 뒤 자동 종료하여 사용자가 다른 모델에 별도로 구현을 요청할 수 있도록 문서를 수정하는 것이다. 사용자는 수정안을 확인한 뒤 “응 그러면 수정해줄래?”라고 승인했다. 이번 문서 편집 요청의 완료 뒤 P2G 실행을 자동 재개하지 않는다.

## 현재 범위와 이전 요청의 대체

- 선정 R은 기존 **R06 — ValueExpr·집계·Attack/SpellPower와 무기 성장**으로 유지한다. 원래 요청은 [REQUEST_INTAKE_001](REQUEST_INTAKE_001.ko.md)에 보존한다.
- 현재 요청 기준은 [AUTORUN single-r-v4-plan-only](../../roadmaps/content_authoring/AUTORUN_REQUEST.ko.md)와 [운영 v1.7](../../roadmaps/content_authoring/OPERATING_RULES.md)이다. 기존 single-r-v3의 R 전체 완료·GEX 연속 구현/감사 위임은 이번 사용자 변경 지시로 대체된다. R06의 기능·정책·수용 기준은 유지한다.
- 향후 계획 재개 범위는 R06 P2G `bounded_pipeline`이며, 종료 결과는 필요한 P2G-60의 필수 검증과 AUTORUN §6·§9·§10의 설계·기준선·계획 인계 요건 충족이다. 종료 시 `P2G 계획·인계 완료 / 구현 요청 대기`를 보고하고 최종 응답으로 반환한다. 이는 요청한 계획 결과의 정상 완료이며 R06 전체 완료가 아니다.
- GEX-00 인수·Goal activation·production/test/RON 구현 변경·GEX-20/30 감사는 별도 사용자 요청이 필요하다. 일반적인 “계속”, 자동 재개·컨텍스트 압축·현황 갱신·모델 변경으로 이 경계를 넘지 않는다. 사용자가 “이 계획으로 구현해줘”라고 명시하면 exact Handoff·Goal과 실제 구현 범위·종료점을 연결한다.
- 계획을 만든 고성능 모델과 구현 모델의 교체는 사용자가 수행한다. 문서가 모델·실행 중인 다른 세션·시스템 Goal을 자동 전환했다고 주장하지 않는다.

## 구현자에게 전달할 내용

설계 품질과 기준선 원본 요건은 AUTORUN §6·§9가 소유한다. R06에서는 parser/validation/compiler/evaluator, snapshot/live, stat/무기 합성, resource/result, preview/explain/save/cache의 변경 책임과 타입·입출력·처리 순서·오류·취소·저장 및 직접 시험 기대값을 연결한다. 핵심 알고리즘·계약·시험 oracle을 구현자에게 미정 상태로 일괄 넘기지 않는다. 필수 상세 설계나 선행 구현이 남으면 정확한 위치·owner·준비 조건을 기록하며 P2G-60 current를 구현 준비 완료로 바꾸지 않는다.

계획 완료 시 exact Freeze·Phase Plan·Goal Set·Handoff와 검증 결과, Git branch/HEAD/status, 관련 파일과 계획 산출물의 SHA-256, manifest 해시 및 복원 가능한 원본 위치를 인계한다. 그 시점의 관찰을 구현 직전 baseline으로 가장하지 않는다. 별도 구현 요청을 받은 모델이 mutation 직전에 차이와 영향 범위를 확인하고 실제 baseline을 연결한다. 해시만으로 원본 복원을 주장하지 않는다.

앞서 논의한 구현 후 고성능 검토를 위한 후속 요청은 exact Goal의 `goal-run`에 GEX-10 구현·자체 검증·Audit Target Record 고정 직후 반드시 중단하는 사용자 checkpoint를 명시한다. 구현 종료 후 GEX-20은 별도 감사 요청을 기다리며 Goal 전체 완료로 표시하지 않는다. GEX-00과 GEX-10을 단계별로 요청한다면 각각 별도 `stage-run`이다. 필수 선행 감사가 남은 다음 Phase는 실행하지 않으며, 실제 reviewer 독립성·교정 pass·currentness 규칙은 GEX를 따른다. 이 안내는 현재 구현/감사 실행 승인이 아니다.

## 현재 증거와 다음 작업

- 마지막 완료가 검증된 formal checkpoint는 `WR-CAVE-P2G10-001`이며 9 elements/9 Decisions/0 questions다. 기존 P2G-00/10 JSON·판정은 당시 원본을 유지한다. 실행 요청 변경을 정식 정책 내용 변경으로 소급 기록하지 않는다.
- [CODE_REVIEW_001](CODE_REVIEW_001.ko.md)과 [P2G_20_SOURCE_EVIDENCE_001](P2G_20_SOURCE_EVIDENCE_001.json)이 존재한다. 이 문서 개정에서는 내용을 수정하거나 complete Review Batch 검증을 수행하지 않았다. 존재만으로 P2G-20 완료를 선언하지 않는다.
- 다음 계획 owner는 P2G-20이다. 기존 review/source evidence의 유효성을 확인해 재사용하고, 요청 범위 metadata에 successor가 필요하면 실제 P2G 규칙으로 처리한다. 해당 단계의 complete batch와 code_reviewed checkpoint 검증 뒤 P2G-30/40/50/60으로 이어간다.
- 문서 개정 시점에는 exact R06 Phase Plan·Goal Set·Handoff와 계획 종료 manifest가 아직 없다. 미리 파일/Ref/해시를 만들어 인계 완료로 표시하지 않는다.

## 이번 문서 편집의 보존·검증 범위

변경 대상은 AUTORUN_REQUEST, OPERATING_RULES, ROADMAP_STATUS, R06 mutable WORKING_POLICY와 이 새 인수 기록이다. 기존 [single-r-v3 원문](../../roadmaps/content_authoring/records/plan_handoff_boundary_20260913/before/AUTORUN_REQUEST.ko.md)과 [운영 v1.6 원문](../../roadmaps/content_authoring/records/plan_handoff_boundary_20260913/before/OPERATING_RULES.md)을 보존했다. P2G/GEX 본문·검증기·원래 요청 인수·formal artifact·과거 verdict·게임 source/RON은 변경 대상이 아니다.

현재 요청문/운영 규칙의 내용 변경은 과거 해시와 같다고 주장하지 않는다. 과거 감사의 target·판정은 당시 그대로 보존하고, 다음 P2G/GEX owner가 직접 소비하는 governance 변경과 근거의 현재성만 필요한 범위에서 확인한다. 문서 편집을 새 감사 통과나 선행 source 재검증으로 보고하지 않는다.

문서 편집 검증은 [DOCUMENT_VALIDATION](../../roadmaps/content_authoring/records/plan_handoff_boundary_20260913/DOCUMENT_VALIDATION.json)에 기록한다. 이는 문서 경계·보존·링크 검사이며 P2G-20/60 완료, 게임 테스트 또는 감사 증거가 아니다.


## 2026-09-13 후속 요청 — 원본 보존 재검토·계획 인계

위 본문은 당시 인수 상태로 그대로 보존한다. 이후 사용자가 별도 identity의 P2G-00 재검토를 명시했으며, 현재 결과는 [새 Working Policy](../content_authoring_value_expressions_reassessment/WORKING_POLICY.ko.md)와 [계획 인계](../content_authoring_value_expressions_reassessment/PLAN_HANDOFF.ko.md)가 소유한다. exact `FPF-CAVER-P2G30-001 / PP-CAVER-P2G40-001 / GS-CAVER-P2G50-001 / HA-CAVER-P2G60-001`; 27 acceptance 보존, P2G-00→60 재검토 완료. 원문/비교·단계별 검증·설계·branch/HEAD/status·파일별 hash·manifest 자체 hash·실제 복원 사본은 인계에 연결했다. 기존 WPD-CAVE와 formal artifact는 그대로 보존한다. **R06 P2G 재검토·계획 인계 완료 / 구현 요청 대기**; GEX 인수·activation·구현·감사는 별도 명시적 구현/감사 요청 전 시작하지 않는다.
