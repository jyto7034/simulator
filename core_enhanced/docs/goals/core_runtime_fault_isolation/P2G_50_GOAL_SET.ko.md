# U03 오류 전투 격리 — Single Goal Set

## Goal Set

- Goal Set Ref: GS-CRFI-001
- Predecessor Goal Set Ref: N/A
- Lifecycle Disposition: current
- Exact Prerequisite Composite Basis: WR-CRFI-007; FPF-CRFI-003; PP-CRFI-002; PVR-CRFI-002; COD-CRFI-SCHEMA-001 and COD-CRFI-SCHEMA-002 (all owner obligations resolved in P2G_40_RESULT_002.json)
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CRFI-002
- Final Policy Freeze Ref: FPF-CRFI-003
- Complete Goal Member Set: GOAL-CRFI-01
- Applicable P2G-50 Obligation Key Set: explicit empty
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

P2G 산출물 자체는 구현 권한을 만들지 않는다. 원 사용자의 연속 요청이 별도로 GEX 구현·독립 감사·교정을 승인했다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| PHASE-CRFI-01 | GOAL-CRFI-01 | AG-CRFI-RUNTIME, WP-CRFI-VERIFY | 1 |

### Shared Validation Baseline Reuse Contract

- User-Approved Directive Ref: PIS-CRFI-001 IN-001의 기존 계획/유효 증거 재사용·불필요 반복 금지. 별도 고정 횟수 directive는 N/A; 필수 검증/독립 감사가 우선한다.
- Baseline Evidence Key: command/selector, fixture/input, source/data/test/config, toolchain/lock/dependency, raw result와 owning Goal.
- Reuse Rule: exact input identity와 evidence contract가 유지된 근거만 재사용. evidence/PREDECESSOR_BASELINE_001.json의359개 제품 bytes와 U02 독립 Master002 종료를 비교해 initial baseline 인수. 이후 source의 PASS로 가장하지 않는다.
- Production Input Invalidation / Focused Revalidation Map: application/fault/origin/reset→V01–04,V11,V12; actor/session/writers→V05–09,V12; observation terminal→V10,V12와 U02 PE08–12; docs-only 변경은 해당 계약/링크 검증.
- Test/Fixture/Probe Change Invalidation / Rebaseline Rule: 바뀐 seam을 current source에서 nonzero 실행. 과거 실패와 입력은 보존하고 새 receipt를 발행.
- Shared Harness/Schema/Persistence/Transport/Dependency Invalidation Rule: affected consumer set과 모든 공유 seam을 명시적으로 invalidation; test file이 그대로라는 이유로 runtime source 변경의 증거를 재사용하지 않는다.
- Phase-Start Rule: 단순 단계 시작만으로 재실행하지 않는다. 이 Goal의 current impact map이 필요한 재검증을 소유한다.
- Initial Full Baseline Owner / Timing / Command Set: GOAL-CRFI-01, production mutation 전 exact U02 final baseline 인수. Core default/test-fixtures1701/1701/observation1707, host77/84, 별도 allocator 및 실제 공개 소비자 원본을 currentness와 함께 연결한다.
- Final Full Baseline Owner / Timing / Command Set: GOAL-CRFI-01 WP-CRFI-VERIFY; current integrated source의 Core default/test-fixtures/runtime-observation, host default/observation, fmt/check/clippy와 실제 public API/headless host 및 필요한 inherited seams.
- Normal Full-Validation Count: initial은 exact 검증된 U02 baseline을 재사용하고 final integrated set은 한 번 성공 고정한다. 변경/실패/zero-execution으로 무효화된 검증은 필요한 만큼 교정·재실행하며 고정 횟수 때문에 필수 검증을 생략하지 않는다.
- Failure/Zero-Execution/Post-Run-Change Retry Rule: incomplete/failed/changed-input 실행은 PASS로 세지 않고 원인 교정 후 영향을 받은 seam을 다시 결속한다. R05 raw101 경고와 OBS001/20분 제외는 그대로다.

### Integration Acceptance-to-Goal Mapping

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |
| IVA-CRFI-V01 / CODE_REVIEW_001.ko.md V01; exact frozen PE acceptance mapping in review | assigned_execution_node | GOAL-CRFI-01 / WP-CRFI-VERIFY | execution/single_goal/COVERAGE.json V01; actual nonzero Core/public/host execution appropriate to seam; exact source/config/input/seed/toolchain/raw exit/expected-vs-actual; preserved negative receipts |
| IVA-CRFI-V02 / CODE_REVIEW_001.ko.md V02; exact frozen PE acceptance mapping in review | assigned_execution_node | GOAL-CRFI-01 / WP-CRFI-VERIFY | execution/single_goal/COVERAGE.json V02; actual nonzero Core/public/host execution appropriate to seam; exact source/config/input/seed/toolchain/raw exit/expected-vs-actual; preserved negative receipts |
| IVA-CRFI-V03 / CODE_REVIEW_001.ko.md V03; exact frozen PE acceptance mapping in review | assigned_execution_node | GOAL-CRFI-01 / WP-CRFI-VERIFY | execution/single_goal/COVERAGE.json V03; actual nonzero Core/public/host execution appropriate to seam; exact source/config/input/seed/toolchain/raw exit/expected-vs-actual; preserved negative receipts |
| IVA-CRFI-V04 / CODE_REVIEW_001.ko.md V04; exact frozen PE acceptance mapping in review | assigned_execution_node | GOAL-CRFI-01 / WP-CRFI-VERIFY | execution/single_goal/COVERAGE.json V04; actual nonzero Core/public/host execution appropriate to seam; exact source/config/input/seed/toolchain/raw exit/expected-vs-actual; preserved negative receipts |
| IVA-CRFI-V05 / CODE_REVIEW_001.ko.md V05; exact frozen PE acceptance mapping in review | assigned_execution_node | GOAL-CRFI-01 / WP-CRFI-VERIFY | execution/single_goal/COVERAGE.json V05; actual nonzero Core/public/host execution appropriate to seam; exact source/config/input/seed/toolchain/raw exit/expected-vs-actual; preserved negative receipts |
| IVA-CRFI-V06 / CODE_REVIEW_001.ko.md V06; exact frozen PE acceptance mapping in review | assigned_execution_node | GOAL-CRFI-01 / WP-CRFI-VERIFY | execution/single_goal/COVERAGE.json V06; actual nonzero Core/public/host execution appropriate to seam; exact source/config/input/seed/toolchain/raw exit/expected-vs-actual; preserved negative receipts |
| IVA-CRFI-V07 / CODE_REVIEW_001.ko.md V07; exact frozen PE acceptance mapping in review | assigned_execution_node | GOAL-CRFI-01 / WP-CRFI-VERIFY | execution/single_goal/COVERAGE.json V07; actual nonzero Core/public/host execution appropriate to seam; exact source/config/input/seed/toolchain/raw exit/expected-vs-actual; preserved negative receipts |
| IVA-CRFI-V08 / CODE_REVIEW_001.ko.md V08; exact frozen PE acceptance mapping in review | assigned_execution_node | GOAL-CRFI-01 / WP-CRFI-VERIFY | execution/single_goal/COVERAGE.json V08; actual nonzero Core/public/host execution appropriate to seam; exact source/config/input/seed/toolchain/raw exit/expected-vs-actual; preserved negative receipts |
| IVA-CRFI-V09 / CODE_REVIEW_001.ko.md V09; exact frozen PE acceptance mapping in review | assigned_execution_node | GOAL-CRFI-01 / WP-CRFI-VERIFY | execution/single_goal/COVERAGE.json V09; actual nonzero Core/public/host execution appropriate to seam; exact source/config/input/seed/toolchain/raw exit/expected-vs-actual; preserved negative receipts |
| IVA-CRFI-V10 / CODE_REVIEW_001.ko.md V10; exact frozen PE acceptance mapping in review | assigned_execution_node | GOAL-CRFI-01 / WP-CRFI-VERIFY | execution/single_goal/COVERAGE.json V10; actual nonzero Core/public/host execution appropriate to seam; exact source/config/input/seed/toolchain/raw exit/expected-vs-actual; preserved negative receipts |
| IVA-CRFI-V11 / CODE_REVIEW_001.ko.md V11; exact frozen PE acceptance mapping in review | assigned_execution_node | GOAL-CRFI-01 / WP-CRFI-VERIFY | execution/single_goal/COVERAGE.json V11; actual nonzero Core/public/host execution appropriate to seam; exact source/config/input/seed/toolchain/raw exit/expected-vs-actual; preserved negative receipts |
| IVA-CRFI-V12 / CODE_REVIEW_001.ko.md V12; exact frozen PE acceptance mapping in review | assigned_execution_node | GOAL-CRFI-01 / WP-CRFI-VERIFY | execution/single_goal/COVERAGE.json V12; actual nonzero Core/public/host execution appropriate to seam; exact source/config/input/seed/toolchain/raw exit/expected-vs-actual; preserved negative receipts |

## Single Goal Member `GOAL-CRFI-01`

- Goal ID: GOAL-CRFI-01
- Goal Kind: single
- Phase ID: PHASE-CRFI-01
- Assigned Final Execution Node Set: AG-CRFI-RUNTIME, WP-CRFI-VERIFY
- Covered Policy Element Revision Ref Set: PER-CRFI-01-REVIEWED-003, PER-CRFI-02-REVIEWED-003, PER-CRFI-03-REVIEWED-003, PER-CRFI-04-REVIEWED-003, PER-CRFI-05-REVIEWED-003, PER-CRFI-06-REVIEWED-003, PER-CRFI-07-REVIEWED-003, PER-CRFI-08-REVIEWED-003, PER-CRFI-09-REVIEWED-003, PER-CRFI-10-REVIEWED-003, PER-CRFI-11-REVIEWED-003, PER-CRFI-12-REVIEWED-003
- Covered Decision Revision Ref Set: DR-CRFI-01-002, DR-CRFI-02-002, DR-CRFI-03-002, DR-CRFI-04-002, DR-CRFI-05-002, DR-CRFI-06-002, DR-CRFI-07-002, DR-CRFI-08-002, DR-CRFI-09-002, DR-CRFI-10-002, DR-CRFI-11-002, DR-CRFI-12-002
- Derived Observable Policy Acceptance Coverage: FPF-CRFI-003의 위12 exact member가 소유한 acceptance 전부. 아래 조건은 실행 해설이며 의미를 변경하지 않는다.
- Objective / Contract-Visible Result: 실제 runtime fault가 발생한 공개 live battle을 명시적 오류 상태로 종료하고 tick/명령/reconnect/저장 재개를 차단한다. 최초 원인을 bounded하게 조회하고 기존 명시적 새 세션/유효 checkpoint Continue로 안전 복구한다.
- User-Visible Problem: 현재 오류는 candidate rollback과 timer 취소로 끝나 같은 actor가 재접속/수동 명령으로 다시 실행되거나 관리/stop 저장 경로를 통과할 수 있다.
- User-Visible Completion Condition Set: 첫 실제 한도/내부/수식 실패 후 마지막 root와 동일 first fault 보존; 모든 후속 gameplay/admin/관리 우회 거절; reconnect 오류 조회; active/profile 원본 bytes 보호; 명시적 Quit/new actor/기존 Continue; 정상 다른 player 진행; 정상 입력/조회/전달 오류와 분류; bounded 진단/관측 종료. 각각 V01–12로 독립 판정한다.
- In-Scope / Out-of-Scope: AG-CRFI-RUNTIME의 Core/host/store/observation terminal 연결과 WP-CRFI-VERIFY. 제외: 정상 결과·보상·SavePoint U04 정책, 데이터 삭제/자동 변환, 전 Run rollback/같은 전투 재시작 보장, skill auto-cancel, 서버 process kill, U11 mod 등록, Unity UI,20분 시험·임의 gameplay/latency/RSS cap.
- Reviewed Current Behavior / Implementation Delta: CODE_REVIEW_001.ko.md RQ01–08/TD01–08. 외부 CoreApplication fault owner와 raw typed origin 분류, public gate/bounded query, actor timer/session/writer 전체 연결, 별도 observer terminal clear. 기존 GameCore sole commit과 first-fault budget은 유지한다.
- Goal-Local Source of Truth / Owning Layer: CoreApplication만 오류 전투 identity/최초 원인을 소유한다. GameCore는 마지막 accepted gameplay root/commit, host는 session/scheduler/store, observer는 별도 bounded 계측을 소유한다. host에서 오류 정책을 복제하지 않는다.
- Goal-Local Invariant Set / Source Refs: FPF-CRFI-003 PE01–12; PP-CRFI-002 intermediate-state audit. 실패 rollback으로 terminal flag나 work를 지우지 않음; 오류는 정상 BattleResult 아님; 일반 입력/read-only/postcommit 오류와 구분; original bytes/권리/다른 player 보존; private speculative retry/취소 유지.
- Code/Data/Schema/Store/Transport/Consumer Boundary: application raw GameError→bounded typed capsule/query/result; all public mutate/prepare/rollback/export ports와 reset Result; actor manual/admin/tick/Ensure/Attach/Detach/stop→실제 active/profile writer gates; websocket snapshots; observer committed versus terminal state. 새 fault는 save schema 밖 application 수명. CODE_REVIEW TD01–08의 producer/consumer/order/failure/cutover 상세를 따르며 private runtime oracles를 public host 검증으로 대체하지 않는다.
- Consumer Verification Boundary: workspace-confirmed: 공개 CoreApplication API, authoring trial/API/CLI, PlayerGameActor/Session, current active/profile stores, runtime observer/exporter. release 실제 local HTTP/websocket host가 required. Unity는 canonical consumer만 알려진 후속 UI이며 이 Goal의 required 구현/배포 claim이 아니다. 원격 deployed source는 조사/호환 claim 대상이 아니다.
- Goal-Local Dependencies / Execution Prerequisites: U02 독립 Master002 Complete/High 및359개 source currentness, exact current Freeze003/Plan002/receipt002/Goal/Handoff와 두 schema Change closure. GEX before archive에 사용자 dirty 포함. Core→host/observer의 실제 API 준비 뒤 atomic unit 내 통합; 독립 완료를 분할하지 않는다. fixed-size capsule·origin/gate enumeration·writer/stop mapping을 구현 직전 확정한다.
- Validation Plan / Completion Matrix: CODE_REVIEW V01–12와 integration mapping 전체. 실제 native limit·legal dynamic expression/internal failure와 일반 거절 대조, last commit/root/RNG/event, 모든 공개 gate, diagnostic failure, 실제 timer/late callback/reconnect/duplicate, file bytes/stop, two-player, Quit/recreate/Continue, observer off/on/failure parity, authoring/private/opaque capability, final Core/host 전체와100 predecessor 조건/31 R06 mapping을 source-bound nonzero receipts로 닫는다.
- Structural Completion / Legacy and Fallback Disposition: 단일 Core fault authority, typed origin, precommit과 postcommit 분리. host-only flag/숨은 정상 패배 정산/자동 retry/스킬 cancel/저장 삭제 fallback 없음. public private-root escape와 fixture-only production branch 없음. 불필요 미래 schema/abstraction 없음; 기존 normal no-fault path는 동일 authority를 유지한다.
- Canonical Documentation Obligation: core_runtime_contract.md에 fault 수명·분류·조회·mutable/save port·명시적 recovery 경계; host 소비 문서 및 기존 관측 계약에 terminal cleanup 설명을 연결한다. 과거 audited metric 문서는 원본 보존하고 필요하면 successor 문서/링크를 쓴다. 로드맵 완료 현황은 독립 감사 종료 뒤만 반영한다.
- Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan: 위 공통 계약. current input-to-seam map과12 U02+88 U01/R06/R07 조건을 각각 재사용/영향/의도된 U03 전환/새 증거로 기록. 실제 public live retry/terminal observation 변화는 U03 정책으로 명시하고 private speculative retry는 유지한다.
- Assigned Durable Integration Validation Seam Coverage: IVA-CRFI-V01–V12 모두 WP-CRFI-VERIFY / GOAL-CRFI-01. Durable fixture/probe를 구현자가 준비하고 감사자에게 누락 seam 구현을 넘기지 않는다.
- Applicable Migration/Compatibility/Rollout/Rollback: no-fault JSON shape와 existing save format/Continue/Load 비용은 유지. 새 bounded fault query/snapshot field와 reset Result의 실제 caller를 함께 전환. 기존 파일 삭제/일괄 변환 없음. before bytes/archive→forward/reverse patch 복원을 실제 실행. 실패 candidate root rollback과 explicit Quit/new owner 복구는 서로 다른 수명이며 전체 Run rollback을 약속하지 않는다.
- Goal-Local Known Unknown / Blocker / Stop Condition: GEX에서 concrete helper명/정상 loader negative fixture/실제 observed counts 확정; required source/meaning unknown 없음. 구현 증거는 아직 없음. 새 의미/경계 premise 발견 시 가장 이른 P2G owner로 successor 처리. 중단은 사용자 stop, 승인 로드맵 완료, 현재 권한·환경으로 해결 불가 실제 장애뿐.
- Applicable Risk / Reviewer Requirement and Evidence: implementation-nonparticipant GPT-6 Astra / High가 실제 raw source/receipts/currentness/restore, classifier의 일반 거절, 숨은 writer/late callback, root/first-fault lifetime와100 inherited 조건을 독립 감사한다. finding 교정·재감사 완료 전 U04 구현 금지. 이 authoring은 reviewer activation이나 GEX gate 실행이 아니다.

### Validation Dimension Matrix

| Dimension | Disposition | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |
| --- | --- | --- | --- |
| success | required | V01,V05,V08,V09 real API/host | fault 조회/안전 종료/기존 Continue 성공과 정상 다른 player 진행 |
| invalid_input | required | V02,V04,V08 typed negative inputs | 일반 authority/action/profile/query/invalid save 거절은 명시적이며 잘못된 fatal/저장 변경 없음 |
| failure_rollback | required | V01–03,V07 actual failure and diagnostics | 마지막 root/RNG/event·원본 파일 보존, first-fault 중단은 rollback 밖, 진단 실패에도 gate 유지 |
| retry_idempotency | required | V01,V05,V07 repeated commands and prepared failure retry | 같은 first fault, 재실행/정산/쓰기 없음; 미게시 candidate의 기존 retry 보존 |
| ordering_concurrency | required | V05,V06,V09 late callback/session replacement/two players | 기존 session authority와 순서 유지; fault actor만 중단, 다른 actor 진행 |
| boundary_empty_max | required | V01,V02,V04,V10 limits/large diagnostics/no observer | hard limit·empty/missing·large cause가 bounded, 같은 fault 보존, 관측 포화가 gameplay와 무관 |
| disconnect_restart_persistence | required | V06–08 live reconnect/quit/recreate/store hashes | 같은 actor fault 유지; stop writes차단; 새 actor의 valid Continue만 복구 |
| live_data | required | V01,V02,V12 actual validated external/embedded content | 실제 production loader로 시작해 nonzero runtime limit/expression failure, fixture만으로 완료 아님 |
| producer_consumer | required | V04–10 actual public API/actor/websocket/worker | Core typed state가 실제 query/host/gates/selected cleanup까지 일치 |
| legacy_fallback_absence | required | V06,V07,V11,V12 source and negative routes | host 별도 fault authority/자동 재개/가짜패배/저장 삭제/관측 feedback 및 공개 root escape 없음 |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| PE-CRFI-01 | PER-CRFI-01-REVIEWED-003 | DR-CRFI-01-002 | PHASE-CRFI-01 | GOAL-CRFI-01 |
| PE-CRFI-02 | PER-CRFI-02-REVIEWED-003 | DR-CRFI-02-002 | PHASE-CRFI-01 | GOAL-CRFI-01 |
| PE-CRFI-03 | PER-CRFI-03-REVIEWED-003 | DR-CRFI-03-002 | PHASE-CRFI-01 | GOAL-CRFI-01 |
| PE-CRFI-04 | PER-CRFI-04-REVIEWED-003 | DR-CRFI-04-002 | PHASE-CRFI-01 | GOAL-CRFI-01 |
| PE-CRFI-05 | PER-CRFI-05-REVIEWED-003 | DR-CRFI-05-002 | PHASE-CRFI-01 | GOAL-CRFI-01 |
| PE-CRFI-06 | PER-CRFI-06-REVIEWED-003 | DR-CRFI-06-002 | PHASE-CRFI-01 | GOAL-CRFI-01 |
| PE-CRFI-07 | PER-CRFI-07-REVIEWED-003 | DR-CRFI-07-002 | PHASE-CRFI-01 | GOAL-CRFI-01 |
| PE-CRFI-08 | PER-CRFI-08-REVIEWED-003 | DR-CRFI-08-002 | PHASE-CRFI-01 | GOAL-CRFI-01 |
| PE-CRFI-09 | PER-CRFI-09-REVIEWED-003 | DR-CRFI-09-002 | PHASE-CRFI-01 | GOAL-CRFI-01 |
| PE-CRFI-10 | PER-CRFI-10-REVIEWED-003 | DR-CRFI-10-002 | PHASE-CRFI-01 | GOAL-CRFI-01 |
| PE-CRFI-11 | PER-CRFI-11-REVIEWED-003 | DR-CRFI-11-002 | PHASE-CRFI-01 | GOAL-CRFI-01 |
| PE-CRFI-12 | PER-CRFI-12-REVIEWED-003 | DR-CRFI-12-002 | PHASE-CRFI-01 | GOAL-CRFI-01 |
