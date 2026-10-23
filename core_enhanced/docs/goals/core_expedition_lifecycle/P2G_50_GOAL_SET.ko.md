# U12 튜토리얼·원정 시작·귀환과 개인 상태 수명 — Single Goal Set

## Goal Set

- Goal Set Ref: GS-CEXL-001
- Predecessor Goal Set Ref: N/A
- Lifecycle Disposition: current
- Exact Prerequisite Composite Basis: WR-CEXL-008; FPF-CEXL-001; PP-CEXL-001; PVR-CEXL-001. Exact digests는 P2G_30_FREEZE_001.json / P2G_40_RESULT_001.json에 결속한다. Changes/obligations explicit empty.
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CEXL-001
- Final Policy Freeze Ref: FPF-CEXL-001
- Complete Goal Member Set: GOAL-CEXL-01
- Applicable P2G-50 Obligation Key Set: explicit empty
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

P2G 문서 작성 상태다. 문서 자체는 구현 승인이 아니다. 구조 검증과 의미 검토를 닫은 complete Goal Set을 인계하고, 기존 직접 사용자 연속 권한으로 별도 GEX를 실행한다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| PHASE-CEXL-01 | GOAL-CEXL-01 | AG-CEXL-LIFECYCLE, WP-CEXL-PUBLIC, WP-CEXL-VERIFY | 1 |

### Shared Validation Baseline Reuse Contract

- User-Approved Directive Ref: PIS-CEXL-002 및 CONTINUOUS_EXECUTION_INSTRUCTIONS_REVISED.ko.md의 직접 연속 지시. 현재 수용 조건과 실제 변경 영향만 재검증하고 유효 선행 근거를 재사용한다.
- Baseline Evidence Key: owning Goal/command/selector/fixture/input/relevant product/test/config/toolchain/lock/raw result와 정확 claim을 함께 결속한다.
- Initial Full Baseline Owner / Timing / Command Set: GOAL-CEXL-01은 U11 u11_current_001/pass001 Complete/High/open0와 RESUME_20261012_001의 4,678 source exact 일치를 인수하고 mutation 전 recoverable source 기준을 고정한다. 역사적 Full 필드명은 새 전체 검사 지시가 아니다.
- Final Full Baseline Owner / Timing / Command Set: GOAL-CEXL-01/WP-CEXL-VERIFY가 최종 current source의38수용 항목/10차원/19통합과 영향받는 U03/U04/U07M/U11을 닫는다. 실제 명령/selector/입력은 독립 author와 PLAN/EXPERIMENTS에서 고정하고 Sol 한 명이 실행한다.
- Production Input Invalidation / Focused Revalidation Map: mode/start/employee/schema→PS01/02,PE01–06/13; vault/copy/terminal→PE07–12; Gate/exhaustion/recruit/recovery→PE04/08/10/11; profile/CAS/Continue/journal→PE01/07/09/11–13와 U03/U04; loader/capability/normal prefix→PE02/03/13와 U11; clone/layout 비용→U07M.
- Test/Fixture/Probe Change Invalidation / Rebaseline Rule: 변경된 oracle/fixture/입력 seam은 current source에서 nonzero 실행한다. 과거 실패 원본을 보존하고 직원 수 축소, tutorial 완료 선설정, expected 완화로 fresh-public 요구를 우회하지 않는다.
- Shared Harness/Schema/Persistence/Transport/Dependency Invalidation Rule: reader/writer/current/backup/nested SavePoint/converter, server approved/unbound/fallback/journal과 public DTO 직접·간접 소비자를 따라 영향 범위를 확대한다. 시험 파일이 같아도 제품 변경 영향은 재검증한다.
- Phase-Start Rule: 세션·역할·단계 변경만으로 재실행하지 않는다. 실행 중 관련 source/test/fixture/tool/config writer0과 동일 입력 sole executor를 유지한다.
- Normal Full-Validation Count: 고정 전체 suite 횟수는 요구하지 않는다. initial 유효 선행 계약과 final 필수 현재 범위를 닫고 실제 수정/실패/미확인 영향만 재검증한다.
- Failure/Zero-Execution/Post-Run-Change Retry Rule: build/0시험/skip/중단/환경 실패를 functional PASS로 기록하지 않는다. raw 실패와 한계를 보존하고 source 변경 뒤 영향 seam을 갱신한다. 자원/성능은 기능 실행과 격리한다.

### Integration Acceptance-to-Goal Mapping

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |
| IVA-CEXL-PS01 / FPF-CEXL-001 PE-CEXL-PS01: every exact Observable Policy Acceptance item; CODE_REVIEW mapped current row and T sections | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal/COVERAGE.json PE-CEXL-PS01; independent test parts and raw prepared execution; Each exact acceptance item has independently authored nonzero current execution or explicit relevant unchanged evidence. Public claims require normal producer/consumer, not helper-only path. |
| IVA-CEXL-PS02 / FPF-CEXL-001 PE-CEXL-PS02: every exact Observable Policy Acceptance item; CODE_REVIEW mapped current row and T sections | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal/COVERAGE.json PE-CEXL-PS02; independent test parts and raw prepared execution; Each exact acceptance item has independently authored nonzero current execution or explicit relevant unchanged evidence. Public claims require normal producer/consumer, not helper-only path. |
| IVA-CEXL-01 / FPF-CEXL-001 PE-CEXL-01: every exact Observable Policy Acceptance item; CODE_REVIEW mapped current row and T sections | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal/COVERAGE.json PE-CEXL-01; independent test parts and raw prepared execution; Each exact acceptance item has independently authored nonzero current execution or explicit relevant unchanged evidence. Public claims require normal producer/consumer, not helper-only path. |
| IVA-CEXL-02 / FPF-CEXL-001 PE-CEXL-02: every exact Observable Policy Acceptance item; CODE_REVIEW mapped current row and T sections | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal/COVERAGE.json PE-CEXL-02; independent test parts and raw prepared execution; Each exact acceptance item has independently authored nonzero current execution or explicit relevant unchanged evidence. Public claims require normal producer/consumer, not helper-only path. |
| IVA-CEXL-03 / FPF-CEXL-001 PE-CEXL-03: every exact Observable Policy Acceptance item; CODE_REVIEW mapped current row and T sections | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal/COVERAGE.json PE-CEXL-03; independent test parts and raw prepared execution; Each exact acceptance item has independently authored nonzero current execution or explicit relevant unchanged evidence. Public claims require normal producer/consumer, not helper-only path. |
| IVA-CEXL-04 / FPF-CEXL-001 PE-CEXL-04: every exact Observable Policy Acceptance item; CODE_REVIEW mapped current row and T sections | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal/COVERAGE.json PE-CEXL-04; independent test parts and raw prepared execution; Each exact acceptance item has independently authored nonzero current execution or explicit relevant unchanged evidence. Public claims require normal producer/consumer, not helper-only path. |
| IVA-CEXL-05 / FPF-CEXL-001 PE-CEXL-05: every exact Observable Policy Acceptance item; CODE_REVIEW mapped current row and T sections | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal/COVERAGE.json PE-CEXL-05; independent test parts and raw prepared execution; Each exact acceptance item has independently authored nonzero current execution or explicit relevant unchanged evidence. Public claims require normal producer/consumer, not helper-only path. |
| IVA-CEXL-06 / FPF-CEXL-001 PE-CEXL-06: every exact Observable Policy Acceptance item; CODE_REVIEW mapped current row and T sections | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal/COVERAGE.json PE-CEXL-06; independent test parts and raw prepared execution; Each exact acceptance item has independently authored nonzero current execution or explicit relevant unchanged evidence. Public claims require normal producer/consumer, not helper-only path. |
| IVA-CEXL-07 / FPF-CEXL-001 PE-CEXL-07: every exact Observable Policy Acceptance item; CODE_REVIEW mapped current row and T sections | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal/COVERAGE.json PE-CEXL-07; independent test parts and raw prepared execution; Each exact acceptance item has independently authored nonzero current execution or explicit relevant unchanged evidence. Public claims require normal producer/consumer, not helper-only path. |
| IVA-CEXL-08 / FPF-CEXL-001 PE-CEXL-08: every exact Observable Policy Acceptance item; CODE_REVIEW mapped current row and T sections | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal/COVERAGE.json PE-CEXL-08; independent test parts and raw prepared execution; Each exact acceptance item has independently authored nonzero current execution or explicit relevant unchanged evidence. Public claims require normal producer/consumer, not helper-only path. |
| IVA-CEXL-09 / FPF-CEXL-001 PE-CEXL-09: every exact Observable Policy Acceptance item; CODE_REVIEW mapped current row and T sections | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal/COVERAGE.json PE-CEXL-09; independent test parts and raw prepared execution; Each exact acceptance item has independently authored nonzero current execution or explicit relevant unchanged evidence. Public claims require normal producer/consumer, not helper-only path. |
| IVA-CEXL-10 / FPF-CEXL-001 PE-CEXL-10: every exact Observable Policy Acceptance item; CODE_REVIEW mapped current row and T sections | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal/COVERAGE.json PE-CEXL-10; independent test parts and raw prepared execution; Each exact acceptance item has independently authored nonzero current execution or explicit relevant unchanged evidence. Public claims require normal producer/consumer, not helper-only path. |
| IVA-CEXL-11 / FPF-CEXL-001 PE-CEXL-11: every exact Observable Policy Acceptance item; CODE_REVIEW mapped current row and T sections | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal/COVERAGE.json PE-CEXL-11; independent test parts and raw prepared execution; Each exact acceptance item has independently authored nonzero current execution or explicit relevant unchanged evidence. Public claims require normal producer/consumer, not helper-only path. |
| IVA-CEXL-12 / FPF-CEXL-001 PE-CEXL-12: every exact Observable Policy Acceptance item; CODE_REVIEW mapped current row and T sections | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal/COVERAGE.json PE-CEXL-12; independent test parts and raw prepared execution; Each exact acceptance item has independently authored nonzero current execution or explicit relevant unchanged evidence. Public claims require normal producer/consumer, not helper-only path. |
| IVA-CEXL-13 / FPF-CEXL-001 PE-CEXL-13: every exact Observable Policy Acceptance item; CODE_REVIEW mapped current row and T sections | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal/COVERAGE.json PE-CEXL-13; independent test parts and raw prepared execution; Each exact acceptance item has independently authored nonzero current execution or explicit relevant unchanged evidence. Public claims require normal producer/consumer, not helper-only path. |
| IVA-CEXL-PUBLIC / Approved official data→normal local/server start→actual battle/activity→Gate/exhaustion return→fresh-process Continue/story and repeated expedition; no approval bypass | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal test_parts, COVERAGE and finalization/audit; Approved official data→normal local/server start→actual battle/activity→Gate/exhaustion return→fresh-process Continue/story and repeated expedition; no approval bypass |
| IVA-CEXL-ATOMIC / Profile/vault/active exact ownership and CAS across preaccept/accepted/recovery_pending/restart, including checkpointNone ack and no-change stale revision | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal test_parts, COVERAGE and finalization/audit; Profile/vault/active exact ownership and CAS across preaccept/accepted/recovery_pending/restart, including checkpointNone ack and no-change stale revision |
| IVA-CEXL-REGRESSION / Changed draft/clone/persistence/version/capability consumers U03/U04/U07M/U11; finite cost/retained state including vault; explicit nonimpact reuse | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal test_parts, COVERAGE and finalization/audit; Changed draft/clone/persistence/version/capability consumers U03/U04/U07M/U11; finite cost/retained state including vault; explicit nonimpact reuse |
| IVA-CEXL-STRUCTURE / Psychology/old-mode silent fallback absence, final38acceptance/10dimension/node coverage, source/evidence currentness and isolated exact forward/reverse restoration, nonparticipant whole-Goal audit | assigned_execution_node | GOAL-CEXL-01 / WP-CEXL-VERIFY | execution/single_goal test_parts, COVERAGE and finalization/audit; Psychology/old-mode silent fallback absence, final38acceptance/10dimension/node coverage, source/evidence currentness and isolated exact forward/reverse restoration, nonparticipant whole-Goal audit |

## Single Goal Member `GOAL-CEXL-01`

- Goal ID: GOAL-CEXL-01
- Goal Kind: single
- Phase ID: PHASE-CEXL-01
- Assigned Final Execution Node Set: AG-CEXL-LIFECYCLE, WP-CEXL-PUBLIC, WP-CEXL-VERIFY
- Covered Policy Element Revision Ref Set: PER-CEXL-PS01-REVIEWED-001, PER-CEXL-PS02-REVIEWED-001, PER-CEXL-01-REVIEWED-001, PER-CEXL-02-REVIEWED-001, PER-CEXL-03-REVIEWED-001, PER-CEXL-04-REVIEWED-001, PER-CEXL-05-REVIEWED-001, PER-CEXL-06-REVIEWED-001, PER-CEXL-07-REVIEWED-001, PER-CEXL-08-REVIEWED-001, PER-CEXL-09-REVIEWED-001, PER-CEXL-10-REVIEWED-001, PER-CEXL-11-REVIEWED-001, PER-CEXL-12-REVIEWED-001, PER-CEXL-13-REVIEWED-001
- Covered Decision Revision Ref Set: DR-CEXL-001, DR-CEXL-01-001, DR-CEXL-02-001, DR-CEXL-03-001, DR-CEXL-04-001, DR-CEXL-05-001, DR-CEXL-06-001, DR-CEXL-07-002, DR-CEXL-08-001, DR-CEXL-09-001, DR-CEXL-10-001, DR-CEXL-11-001, DR-CEXL-12-001, DR-CEXL-13-001
- Derived Observable Policy Acceptance Coverage: FPF-CEXL-001의15 exact member가 소유한38항목을 그대로 파생한다. IVA 각 요소가 그 요소의 모든 항목을 소유하며 PUBLIC/ATOMIC/REGRESSION/STRUCTURE 4통합을 추가 결속한다. 새 수용 의미를 만들지 않는다.
- Objective / Contract-Visible Result: 플레이어가 독립 튜토리얼을 선택/반복/skip하거나 즉시 원정을 시작하고,6중3 직원과 최대2보관 사본을 선택해 실제 전투·활동/유료 채용을 거쳐 합법 귀환한다. 개인 성격/condition과 사본 정체성은 해당 수명 동안 유지되고, 귀환/실패/포기 및 최초 이야기 결과가 정상 저장/재시작에서 정확히 유지된다. 기억·신뢰 시스템은 지원 표면에서 완전히 제거한다.
- User-Visible Problem: 현재 Standard/Endless는 tutorial 완료 gate와 부분 초기화를 사용하고 보관함/정확 반출/첫귀환 이야기가 없다. 기존 profile OR/max merge는 대여처럼 감소하는 사본 소유를 처리할 수 없으며 기억·신뢰 필드와 묵인하는 legacy reader가 남아 있다.
- User-Visible Completion Condition Set: (1) 실제3층 고정 튜토리얼이 영구 profile에 영향을 주지 않고 반복/skip/완료된다. (2) fresh 원정이 역할별2/총6중3,기본세트/예산을 정확 한 번 받고 새 identity로 시작한다. (3)0–2실물 대여와 현재 소유 사본의 합법 귀환/실패 손실이 정확하다. (4)유료 max2 채용과 실제 회복 행동/실패 판단,depth3 Gate 및 genuine exhaustion 선택이 실제 경로로 작동한다. (5)종료 인력/첫이야기/Continue와 profile+active crash 복구가 일관되며 구형 입력 원본을 보호한다. (6)normal 승인 local/server와38항목/10차원/관련 회귀가 독립 Complete/High/open0로 닫힌다.
- In-Scope / Out-of-Scope: FPF의15요소 및 PS01/PS02 완전 제거, 실제 fixed tutorial/minimal authored 개발 원정, 직원/채용/예산/사본 vault/귀환/회복/종료/story 및 공개 승인·저장 전환/문서/시험을 포함한다. U13 전체graph/U14 전체 억제력·Breakdown·황금가지,개별 최종보스,실제Unity UI/VN,배포/출시 세이브 migration은 제외한다. 기억 보존 모델,구형 필드 silent drop/alias,획득 기록 재발급,임의 vault gameplay cap은 허용하지 않는다.
- Reviewed Current Behavior / Implementation Delta: CRA-CEXL-002 및 CODE_REVIEW T01–T07는 실제 현재와 제안을 구분한다. PreparedApplication exclusive draft,copy inventory/fragment forms,seed queue/loadout grant,fixed3층/actor override,paired intent/CAS,U11 approved facade를 재사용한다. 전체 run reset/새 instance,vault exact lifecycle,real return/exhaustion,priced2hire/meaningful recovery,terminal/story 및 strict schema가 새 delta다.
- Goal-Local Source of Truth / Owning Layer: GameCore draft는 원정 자원/직원/실제 행동과 terminal을 소유한다. profile는 vault 사본·원정 sequence/active binding·story 및 별도 monotonic knowledge를 소유한다. authored run/employee/map 정책이 roles/가격/깊이/튜토리얼 수치를 소유한다. paired accepted journal이 profile+active publication을, U11 issuer/host가 승인된 data generation을 소유한다. DTO/history/UI는 사본 재발급이나 귀환 authority가 아니다.
- Goal-Local Invariant Set / Source Refs: FPF-CEXL-001과 CODE_REVIEW T01–T07. tutorial 영구 비영향; same scenario와 new instance 분리; 기본 예산/장비 한 번; vault∪active UUID 단일 소유; totalloan≤2; maxroster10/deploy8; 실제 현재 depth/source 기반 귀환;0전이 receipt 한 번; terminal 원자 정산과 pending story; same-instance rollback에서 최신 profile 비되돌림; exact CAS/accepted recovery; old nested psychology 명시 거절과 원본 보존.
- Code/Data/Schema/Store/Transport/Consumer Boundary: 공개 start/phase capability→현재 공식 role/loadout/run/facility data→Core draft mode/instance/run state→선발/대여→공용 실제 activity/battle→Gate/exhaustion/recovery/terminal→profile exact vault/story+active checkpoint→normal approved local/server paired store→fresh Continue/query. Employee/default/node-advance/clone/DTO와 reader/writer/converter/version을 한 atomic cutover에 연결한다. 터미널 뒤 start guard와 seed-only 검증 소비자도 함께 바꾼다.
- Consumer Verification Boundary: workspace-confirmed Core public commands/query/authoring pack,정상 approved local CLI와 game_server handler/store/intent/backup/Continue를 실제 사용한다. default-release fresh-profile external probe와 fixed tutorial 실전 경로가 필요하다. internal fixtures는 보조이며1명 fixture나 선완료 tutorial은 정상6중3 증거가 아니다. actual Unity/deployed/released는 명시 범위 밖이고 required local consumer 접근 불가 항목은 현재 없다.
- Goal-Local Dependencies / Execution Prerequisites: U11 u11_current_001/pass001 및 현재 source4678일치와 관련 U03/U04/U07M 계약을 인수한다. passed PVR/P2G50/P2G60→GEX00 recoverable baseline 뒤 제품을 쓴다. Main Astra high 계획/설계/실제 코드·품질,Sol6.1 xhigh 제품+sole runner,별도 Astra high test/fixture/oracle,비참여 Astra high final audit. 제품은 구체 module/API/version/tuning을 짧게 명시하고 shared 파일은 전체 단위로 순차 소유한다.
- Validation Plan / Completion Matrix: 38exact 항목을 독립 oracle에 연결해 fresh/repeat tutorial/expedition,roles/loan/copy invalidity,authored depth2/3와 exhaustion receipt,paid2hire queue/price/roster,combined/futile recovery,return/fail/abandon/story/save를 검증한다. same seed 다른 원정 identity, same-instance SavePoint after knowledge/ack 양성,checkpointNone stale CAS,preintent fail/accepted crash/restart,valid-checksum nested legacy reject와 original protection 음성을 포함한다. 실제 approved official pack→local/server까지 수행하고 changed clone/persistence/capability 및 자원 비용의 영향 선행 회귀를 선별한다.
- Structural Completion / Legacy and Fallback Disposition: Employee memory/trust/default/advance/query/codec/fixture와 전용 입력/capability가 남거나 빈 null shape로 대체되지 않아야 한다. old Standard/Endless를 조용히 새 의미로 읽지 않고 unsupported current/backup/nested SavePoint/V7V8 converter를 명시 거절한다. OR/max로 vault 소유 부활,history reissue,튜토리얼 영구 leak,부분 reset/중복 budget,category-only 회복,stale no-change CAS 우회,normal 승인 우회가 없어야 한다. personality/condition/growth/knowledge 및 allocator memory/trusted-host 용어는 별개다.
- Canonical Documentation Obligation: docs/core_runtime_contract.md,docs/data_loading_contract.md 및 현재 공개 API/CLI/server·세이브 문서에 실제 Tutorial/Expedition modes/시작/선발/채용/대여/귀환/종료/story query·ack,strict version/unsupported 원본 보호,exact profile/active intent와 승인 콘텐츠 소비를 반영한다. 실제 제품 지원만 기록하고 roadmap 완료 표기는 독립 감사 뒤 갱신한다.
- Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan: 위 공유 계약을 상속한다. mode/roles→fresh public와공식loader;copy/profile→loan/terminal/CAS/restart;pendingGate/exhaustion/recruit→phase/Continue/SavePoint;psychology→nested codec/clone/query;approved schema→U11 actual preparation/save;clone/work retained→U07M. 기존 유효 authority 알고리즘은 명시 비영향 재사용하며 원시 실패는 보존한다.
- Assigned Durable Integration Validation Seam Coverage: PP의19IVA를 GOAL-CEXL-01/WP-CEXL-VERIFY가 소유한다. 제품/공식 데이터/일반 사용 문서는 Sol,시험/fixture/public probe/oracle/comparator는 Astra가 작성한다. prepared 실행 교정은 승인 범위 내 담당자가 처리하고 실제 입력/소유 변화를 기록한다.
- Applicable Migration/Compatibility/Rollout/Rollback: 새 지원 mode/schema/version을 모든 reader/writer/current/backup/preflight/converter에 동시에 적용한다. 구형 세이브는 명시 미지원으로 거절하고 원본 bytes를 보존한다. same instance run-local SavePoint/퇴각은 최신 vault/story를 되돌리지 않는다. preintent 실패는 draft/원본 불변,accepted 이후는 기존 recovery_pending과 정확 accepted next state 복구를 유지한다. source 정·역 복원은 격리 경로에서 증명한다.
- Goal-Local Known Unknown / Blocker / Stop Condition: 현재 policy/source gap은 없다. concrete API/version/모듈/가격·초기 억제력은 기존 정책이 위임한 구현 상세이고 runtime correctness/비용은 미완료 GEX다. 새로운 의미 변경/불가능한 정상 consumer/원본 보호 계약 충돌이면 earliest P2G owner에 반환한다. 일반 compile/fixture/명확한 결함은 담당자가 자율 교정하고 사용자에게 재확인하지 않는다.
- Applicable Risk / Reviewer Requirement and Evidence: 가장 큰 위험은 vault union복제와 lost update,동일seed UUID 재사용,tutorial profile leak,실제불가능 회복 false positive,exhaustion reward중복,old nested silentdrop,accepted recovery뒤 재정산이다. 본 Goal의 계획·제품·시험 비참여 Astra high가15요소/38항목/10차원/3node/19IVA와 실제 source/raw evidence/관련 선행/복원을 독립 검토해 Complete/High/open0를 판정해야 한다.

### Validation Dimension Matrix

| Dimension | Disposition | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |
| --- | --- | --- | --- |
| success | required | actual fixed tutorial plus fresh6of3 expedition→loan→paid recruit→battle/activity→legal return→story/next start | 정상 공개 경로로 자원·직원·보관·story의 수명 완료 |
| invalid_input | required | wrong role/selection/copy ownership/dormant loan/unsupported mode/version/valid checksum nested old fields | 정확 진단과 wallet/vault/UUID/live/disk 원본 불변 |
| failure_rollback | required | grant/debit/loan/terminal prepare 실패, preintent rename 실패와 accepted crash | 미수락 불변과 수락된 exact next state/recovery_pending 구분 |
| retry_idempotency | required | duplicate start/select/hire/return/ack,0유지 재입장,Continue/SavePoint receipt | 추가예산·가격중복·copy복제·보상·이야기 재소비 없음 |
| ordering_concurrency | required | profile→active CAS 경쟁,stale checkpointNone/no-change ack,approved required generation | 정확 once accepted 소유와 stale 거절;최신 knowledge/story 비되돌림 |
| boundary_empty_max | required | loan0/2/3,roster10/deploy8,hire0/1/2/3,depth2/3,sequence overflow,all dead | 정책 경계 실제 적용과 불가능 회복/fake terminal 거절 |
| disconnect_restart_persistence | required | fresh process current/backup/intent/Continue and pending story, same-instance SavePoint after ack | 최신 exact vault/story와 원정 identity 유지,old/different instance 원본 보호 |
| live_data | required | official external RON→pack loader→normal approved local/server,typed six roles and authored Gate/HQ | 실제 지원3층 tutorial/개발 원정 경로와 잘못된 authored data 거절 |
| producer_consumer | required | Core actual state→prepared profile/checkpoint→accepted journal→server DTO/query→reload | 같은 instance/copy/receipt/version의 end-to-end 소비 |
| legacy_fallback_absence | required | psychology/default/clone/query/capability removal and mode/version/converter audit | silent alias/drop,history reissue,ORvault merge,approval bypass 없이 retained condition/personality 보호 |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| PE-CEXL-PS01 | PER-CEXL-PS01-REVIEWED-001 | DR-CEXL-001 | PHASE-CEXL-01 | GOAL-CEXL-01 |
| PE-CEXL-PS02 | PER-CEXL-PS02-REVIEWED-001 | DR-CEXL-001 | PHASE-CEXL-01 | GOAL-CEXL-01 |
| PE-CEXL-01 | PER-CEXL-01-REVIEWED-001 | DR-CEXL-01-001 | PHASE-CEXL-01 | GOAL-CEXL-01 |
| PE-CEXL-02 | PER-CEXL-02-REVIEWED-001 | DR-CEXL-02-001 | PHASE-CEXL-01 | GOAL-CEXL-01 |
| PE-CEXL-03 | PER-CEXL-03-REVIEWED-001 | DR-CEXL-03-001 | PHASE-CEXL-01 | GOAL-CEXL-01 |
| PE-CEXL-04 | PER-CEXL-04-REVIEWED-001 | DR-CEXL-04-001 | PHASE-CEXL-01 | GOAL-CEXL-01 |
| PE-CEXL-05 | PER-CEXL-05-REVIEWED-001 | DR-CEXL-05-001 | PHASE-CEXL-01 | GOAL-CEXL-01 |
| PE-CEXL-06 | PER-CEXL-06-REVIEWED-001 | DR-CEXL-06-001 | PHASE-CEXL-01 | GOAL-CEXL-01 |
| PE-CEXL-07 | PER-CEXL-07-REVIEWED-001 | DR-CEXL-07-002 | PHASE-CEXL-01 | GOAL-CEXL-01 |
| PE-CEXL-08 | PER-CEXL-08-REVIEWED-001 | DR-CEXL-08-001 | PHASE-CEXL-01 | GOAL-CEXL-01 |
| PE-CEXL-09 | PER-CEXL-09-REVIEWED-001 | DR-CEXL-09-001 | PHASE-CEXL-01 | GOAL-CEXL-01 |
| PE-CEXL-10 | PER-CEXL-10-REVIEWED-001 | DR-CEXL-10-001 | PHASE-CEXL-01 | GOAL-CEXL-01 |
| PE-CEXL-11 | PER-CEXL-11-REVIEWED-001 | DR-CEXL-11-001 | PHASE-CEXL-01 | GOAL-CEXL-01 |
| PE-CEXL-12 | PER-CEXL-12-REVIEWED-001 | DR-CEXL-12-001 | PHASE-CEXL-01 | GOAL-CEXL-01 |
| PE-CEXL-13 | PER-CEXL-13-REVIEWED-001 | DR-CEXL-13-001 | PHASE-CEXL-01 | GOAL-CEXL-01 |
