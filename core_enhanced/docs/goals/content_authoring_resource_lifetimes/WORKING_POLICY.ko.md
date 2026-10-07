# R05 타입화 자원·owner×lifetime·원자적 소비 — Working Policy

2026-09-12 · P2G-60 complete; GEX-00 completed; GEX-10 implementation/validation closed; fresh audits pending. 이 문서는 mutable Working Revision view이며 구현·정책 완료 증거가 아니다.

## Metadata와 승인 경계

- Working Policy Document ID: `WPD-CARL-20260911`
- Policy Topic: R05 타입화 자원·owner×lifetime·원자적 소비
- Target Documentation Root: `core_enhanced/docs`
- Policy Directory Name: `content_authoring_resource_lifetimes`
- Physical Path: `core_enhanced/docs/goals/content_authoring_resource_lifetimes/WORKING_POLICY.ko.md`
- Current Working Revision / Checkpoint: `WR-CARL-P2G20-001`; Successor, predecessor `WR-CARL-P2G10-001`; predecessor Freeze N/A. Initial `WR-CARL-P2G00-001` preserved.
- Policy Intake Snapshot: `PIS-CARL-P2G10-001` (successor; initial snapshot preserved).
- Current Stage: P2G-60 completed; separate GEX-00 accepted; GEX-10 implementation/validation closed and fresh audits pending.
- P2G-10 Completion Predicate: true, proven by [15 exit checks and exact-reference validation](P2G_10_VALIDATION_001.json). Current 9 elements/9 Decisions; 10 candidate dispositions; 0 user questions. 정적 code-informed 검토는 [19종료조건 검증](P2G_20_RESULT_001.json)으로 완료했다. 게임 구현·시험·감사는 미완료다.
- Requested Result / Progression: R05 `bounded_pipeline`, 필요한 P2G-60 Handoff까지. [single-r-v2 인수](REQUEST_INTAKE_001.ko.md)로 R05 하나를 고정한다. 통상적 정책은 위임 조건으로 결정하며 사용자 선택이 필요한 정책만 질문 후 보류한다.
- Stage Selection Basis: Authorized Endpoint Prerequisite under user-adopted [AUTORUN](../../roadmaps/content_authoring/AUTORUN_REQUEST.ko.md) sections3/5/6/7/8. Actual R04 predecessor closure is recorded in [parent acceptance](PARENT_PREDECESSOR_CHECK_001.json).
- Allowed Actions: 필요한 P2G 문서·source 검토·구조 검사. GEX는 실제 Goal/Handoff와 자격을 확인한 별도 child 실행이다. 위임 조건을 만족하는 policy 의미는 P2G-10에서 확정·기록할 수 있다. production/test/RON/canonical mutation은 P2G pass에서 수행하지 않는다.

## Approved initial scope

[Current payload](P2G_20_REVIEW_001.json)의 approved metadata/scope를 아래에 표시한다. 원래 기능 I01–I05/O01–O05는 유지하며 I06의 실행 방식은 채택된 single-r-v2로 갱신했다. 각 항목의 exact user approval reference는 같은 payload가 소유한다.

| ID | Disposition | Scope |
| --- | --- | --- |
| I01 | in | R05 typed resource store with separate owner and Run/Attempt/Deployment/Activation/Delivery/Chain lifetime; reserve/commit/abort, cap/grant/decay and epoch/lease boundaries. |
| I02 | in | Actual persistent employee/equipment ownership and battle binding, including same-definition distinct copies and deployment-versus-attempt state; no definition-ID substitute for actual copy identity. |
| I03 | in | Introduced Run state writer/restore/checkpoint/entry/result-delta/source-owner exactly-once application and required failure/retreat/retry/redeployment/stale work tests; AT05 remains indivisible for each implemented family. |
| I04 | in | Actual directly connected producer/consumer and authoring surface for resource operations; any new RON/schema/opcode requires production parser-validation-compiler-executor and negative fixtures, truthful publication and finite budgets. |
| I05 | in | R05 applicable T05/T20/T23/T29, actual current R04 and affected direct/predecessor consumers, state/clone/determinism/cost validation and eventual fresh independent audit. Unresolved families remain incomplete. |
| I06 | in | R05 P2G-60까지 연속 진행. 통상적 정책은 위임 조건으로 결정하고 사용자 선택 정책만 질문 후 대기. GEX 권한은 별도 사용자 요청 근거. |
| O01 | out | Server/Unity/client, external service deployment, RON editor UI, remote publication and global installation/configuration. |
| O02 | out | Unapproved new gameplay/balance/UX/owner-lifetime/data-loss/compatibility choices. Pending family behavior is not decided by defaults, implementation convenience or elapsed time. |
| O03 | out | R06-R24 full content implementations outside R05 required producer/consumer/state boundary, including R07 fragment-copy binding and later revival/transform/Flow/relic features; their parent requirements remain outstanding. |
| O04 | out | Personal store deletion/automatic conversion, content-name executors, unused kernel or fixture-only false completion, silent success/fallback and fabricated future identities. |
| O05 | out | Changes to AUTORUN/P2G/GEX/validators/AGENTS/runner/global settings or authority created by artifacts. |

## Current policy와 다음 owner

Initial [P2G-00](P2G_00_INITIAL_001.json)과 [결과](P2G_00_RESULT_001.json)는 과거 원본으로 보존한다. 이번 요청/개정/기록된 실제 사용자 답변을 [successor intake](P2G_10_INTAKE_001.json)에 보존하고 [entry revision](P2G_10_ENTRY_001.json)에서 정책 검토를 시작했다.

[검토 결과 payload](P2G_20_REVIEW_001.json)가 current reviewed element/Decision 선택을 소유하고 [Triage](P2G_10_TRIAGE_001.json)는 그 조회용이다. E01 identity/binding, E02 여섯 수명, E03 자원 선언/증감, E04 원자성, E05 장비 유지/소멸, E06 Run timeline, E07 저장 계약, E08 실제 작성/실행, E09 검증/감사를 포함한다. E03/E06은 다섯 조건을 기록한 AI 위임 결정이며 E05는 기존 기록0066의 직접 승인이다.

[제한적 사실 확인](intake/POLICY_FACTS_001.json)은 정책 판단의 근거이고 P2G-20 Code Review Binding이 아니다. [선행 대조](intake/PREDECESSOR_CURRENTNESS_001.json)의20불일치/고유5경로는 [영향 판정](intake/PREDECESSOR_REUSE_001.json)으로 설명한다. 실제 source는 감사 archive와 세 곳 줄바꿈만 다르고, 나머지는 승인된 지침 개정이다. 기존 gameplay 증거를 재사용하되 과거 exact manifest가 현재 일치한다고 주장하지 않는다.

[코드 검토](CODE_REVIEW_001.ko.md)는44파일의 exact source에서9요소의 owner/수명/작성/원자성/AT05/관찰 경계를 확인했다. [Complete result](P2G_20_RESULT_001.json)가11개 source requirement(필수9 confirmed, 외부2 N/A),9 current binding,400 typed reference와 acceptance 보존을 검증했다. [원시 source 근거](P2G_20_SOURCE_EVIDENCE_001.json)와 source archive는 현재 bytes를 고정하며 새 게임 시험 pass를 뜻하지 않는다.

Current planning Freeze는 `FPF-CARL-P2G30-001`이다. [동결 원문](P2G_30_FREEZE_001.json)과 [완료 결과](P2G_30_RESULT_001.json)가9 member/33 acceptance의 정확한 projection과 source/binding coverage를 확인했다. Current Phase Plan은 `PP-CARL-P2G40-002`다. [Phase 요약](PHASE_PLAN_002.ko.md) / [완료 결과](P2G_40_RESULT_001.json)가4WP·1AG·1Phase·12IVA와33 acceptance coverage를 확인했다. Current Goal Set은 `GS-CARL-P2G50-001` / `GOAL-CARL-01`이다. [Goal 원문](P2G_50_GOAL_SET_001.ko.md)과 [작성 결과](P2G_50_RESULT_001.json)가 Quality Contract2/Single member/12IVA/10차원을 검증했다. Current Handoff는 `HA-CARL-P2G60-001`이다. [인계 원문](P2G_60_HANDOFF_001.json) / [완료 결과](P2G_60_RESULT_001.json)가 exact chain과 남은 구현/검증/감사 의무를 전달한다. 기존 사용자 승인에 따른 [GEX-00 인수](execution/single_goal/GEX_00_RESULT_001.json)가 끝났다. [실행 PLAN](execution/single_goal/PLAN.md)의 GEX-10 구현/검증 pass를 닫고 source mutation을 멈췄다. typed 선언·store/registry·실제 battle/Run AT05/catalog/public 관찰을 연결했다. 최종102 original guards,103 전체1410tests,104/105 default-release,106 공개15cases,107 기존소비자13묶음,108 진단10commands,109 음성13cases,110 예산8cases,111 순서/부족,112–116 비용 plain1025/allocator615samples가 통과했다. [수용 대조](execution/single_goal/ACCEPTANCE_EVIDENCE_001.md)가33acceptance/12IVA/10차원을 연결한다. [실행 기록](execution/single_goal/EXPERIMENTS.md)은 실패와 재검증을 보존한다. 필수 R01→R02→R03→R04 fresh 감사 복원·R05 GEX20·parent reconciliation은 남아 있으며 R05 전체는 미완료다. R06은 다음 후보이고 이번 요청에서 착수하지 않는다. source plan은 [RSP-CARL-P2G20-001](P2G_20_SOURCE_PLAN_001.json)이며 조사 중 [진행 기록](P2G_20_PROGRESS_001.ko.md)은 역사적 non-current 메모로 보존한다.
