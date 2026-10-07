# R02 외부 팩·검증 세대·새 런/저장 결속

문서 ID `WPD-CAPG-20260909`. 사용자 채택 AUTORUN의 R02 범위에서 필요한 P2G를 Handoff까지 이어간다. 이 문서는 원래 artifact의 current 선택을 보여주는 작업 view다.

현재 checkpoint: `WR-CAPG-P2G20-001`. P2G-00–60 완료. Current Freeze `FPF-CAPG-P2G30-001`, Plan/Receipt `PP-CAPG-P2G40-001` / `PVR-CAPG-P2G40-001`, Goal Set `GS-CAPG-P2G50-001`, Handoff `HOF-CAPG-P2G60-001`. P2G는 구현을 승인하지 않았다. 상위 사용자 AUTORUN에 따른 별도 exact child GEX-00 인수가 다음이다.

- [초기 Working Revision·raw snapshot](P2G_00_INITIAL_001.json)
- [P2G-00 결과·검증](P2G_00_RESULT_001.json)
- 상위 요청: `../../roadmaps/content_authoring/AUTORUN_REQUEST.ko.md`
- 기준 P2G: `../../policy_to_goal_pipeline_revised_v2.md`, 2026-09-09-r2

현재 요소/Decision·질문 선택은 P2G_10_POLICY_001.json의 successor Working Revision이 소유한다. 초기 empty checkpoint와 raw snapshot은 보존했다. 구체적인 manifest/hash/budget·타입과 전환 설계는 P2G-20에서 검토한다. 역사적 V4 유지와 현재 latest-only 승인은 scope가 다르며, migration certification을 꾸며내거나 개인 파일을 바꾸지 않는다.

- [정책 revision·기존 결정·candidate disposition](P2G_10_POLICY_001.json)
- [정책 검토·참조/coverage 검사](P2G_10_RESULT_001.json)

- [P2G-20 candidate source plan](P2G_20_SOURCE_PLAN_001.json)
- [기존 읽기 조사 currentness 대조](P2G_20_PRELIMINARY_EVIDENCE_001.json) — formal source observation을 대신하지 않음

- [코드 검토 본문](CODE_REVIEW_001.ko.md)
- [complete Review Batch·source/binding](P2G_20_REVIEW_001.json)
- [P2G-20 검사·canonical Batch Ref 보수 결과](P2G_20_RESULT_002.json)

현재 complete Batch는 `RB-CAPG-P2G20-002`다. 최초 Batch record의 비표준 Artifact Type을 새 Ref로 보수했다. 원본 bytes와 동일 WR·요소·binding·source는 보존했다.

- [Final Freeze](P2G_30_FREEZE_001.json)
- [P2G-30 membership·검증](P2G_30_RESULT_001.json)

- [최소 Phase 설계](P2G_40_PHASE_PLAN_DESIGN_001.ko.md)
- [Phase Plan](P2G_40_PHASE_PLAN_001.json) · [표준 Receipt](P2G_40_PHASE_PLAN_VALIDATION_RECEIPT_001.json) · [결과](P2G_40_RESULT_001.json)

- [Single Goal](P2G_50_GOAL_SET_001.ko.md) · [P2G-50 결과](P2G_50_RESULT_001.json)
- [Handoff](P2G_60_HANDOFF_001.json) · [P2G-60 검증](P2G_60_VALIDATION_001.json) · [결과](P2G_60_RESULT_001.json)

## 상위 AUTORUN의 실행 대상 선택

2026-09-10 KST. 사용자 채택 AUTORUN §§3–6에 따라 parent가 다음 child `goal-run`으로 `(GS-CAPG-P2G50-001, GOAL-CAPG-01)`을 선택했다. R02, `PHASE-CAPG-01`, `PP-CAPG-P2G40-001`, current `HOF-CAPG-P2G60-001`과 연결된다. 별도 GEX-00이 현재 chain·R01 선행·실행 이력/dirty·검증 가능성을 인수한다. P2G 완료가 권한을 만든 것이 아니며 아직 구현/감사 완료가 아니다.
