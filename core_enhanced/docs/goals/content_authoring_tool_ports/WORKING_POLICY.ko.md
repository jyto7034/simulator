# R03 공통 제작 API·CLI·설명·실제 headless 시험 포트

문서 ID `WPD-CATP-20260910`. 사용자 채택 AUTORUN의 R03 범위에서 필요한 P2G를 Handoff까지 이어간다. 이 문서는 원래 artifact의 current 선택을 보여주는 작업 view다.

현재 checkpoint: `WR-CATP-P2G20-001`. P2G-00–60 완료. Current Freeze `FPF-CATP-P2G30-001`, Plan/Receipt `PP-CATP-P2G40-001` / `PVR-CATP-P2G40-001`, Goal Set `GS-CATP-P2G50-001`, Handoff `HOF-CATP-P2G60-001`. 상위 사용자 AUTORUN에서 `(GS-CATP-P2G50-001, GOAL-CATP-01)`을 별도 선택해 GEX-00 인수 완료, GEX-10 상세 설계·구현을 진행한다. 게임 구현·검증·완료 감사는 아직 미완료다. [실행 계획](execution/single_goal/PLAN.md) / [실제 인수](execution/single_goal/GEX_00_RESULT_001.json).

- [초기 Working Revision·raw snapshot](P2G_00_INITIAL_001.json)
- [P2G-00 결과·검증과 실제 선행 인수](P2G_00_RESULT_001.json)
- [상위 사용자 채택 요청](../../roadmaps/content_authoring/AUTORUN_REQUEST.ko.md)
- [선택 R03](../../roadmaps/content_authoring/reviews/2026-09-09-dependency-review-v2.2/roadmap_v2.2-review/goals/R03.ko.md)
- [P2G r2](../../policy_to_goal_pipeline_revised_v2.md), `2026-09-09-r2`

초기 요소·Decision·Open Question map/set은 비어 있다. Scope와 approval reference는 initial Working Revision payload가 소유한다. 이 초기화는 raw input을 보존하며 코드 검토·정책 의미 판정·후속 artifact 또는 게임 구현/검증을 생성하지 않는다. 앞서 수행한 읽기 준비 메모는 정식 source observation이 아니며 후속 P2G-20에서 필요한 current 근거를 별도로 결속한다.

현재 요소·Decision·질문 선택은 successor Working Revision이 소유한다. 초기 empty checkpoint와 raw snapshot은 보존했다. 구체적인 인터페이스·진단·시나리오·trace·reload·예산 설계는 P2G-20에서 current source와 대조하며 필요한 새 material 의미가 확인되면 해당 owner에 반환한다.

- [정책 revision·기존 결정·candidate disposition](P2G_10_POLICY_001.json)
- [P2G-10 정책·참조·coverage 검사](P2G_10_RESULT_001.json)

- [상위 요청의 실제 R01/R02 선행 currentness 인수](PARENT_PREDECESSOR_CHECK_001.json) — 새 완료 verdict나 P2G source binding을 대신하지 않는 parent read-only 관찰.

- [P2G-20 candidate source plan](P2G_20_SOURCE_PLAN_001.json), `RSP-CATP-P2G20-001` — 7개 requirement, current input은 계속 `WR-CATP-P2G10-001`. 초기 candidate로 보존되며 같은 exact plan을 complete Review Batch가 소비했다.

- [코드 검토](CODE_REVIEW_001.ko.md) / [source evidence](P2G_20_SOURCE_EVIDENCE_001.json) / [root 의미 검토](P2G_20_SEMANTIC_REVIEW_001.json)
- [complete Review Batch와 source/binding](P2G_20_REVIEW_001.json) / [P2G-20 결과](P2G_20_RESULT_001.json)

- [Final Policy Freeze](P2G_30_FREEZE_001.json) / [P2G-30 membership·검증](P2G_30_RESULT_001.json)

- [Phase Plan](P2G_40_PHASE_PLAN_001.json) / [공유 계약·순서 설계](P2G_40_PHASE_PLAN_DESIGN_001.ko.md)
- [표준 Validation Receipt](P2G_40_VALIDATION_RECEIPT_001.json) / [P2G-40 결과](P2G_40_RESULT_001.json)

- [Single Goal Set](P2G_50_GOAL_SET_001.ko.md) / [authoring 의미 검토](P2G_50_AUTHORING_REVIEW_001.json) / [P2G-50 결과](P2G_50_RESULT_001.json)

- [Handoff](P2G_60_HANDOFF_001.json) / [P2G-60 검증](P2G_60_VALIDATION_001.json) / [결과](P2G_60_RESULT_001.json)
