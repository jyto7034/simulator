# R05 타입화 자원·owner×lifetime·원자적 소비 — Goal Set

2026-09-12 · P2G-50 · Goal Quality Contract 2. Ref 표시는 typed Artifact ID이며 Content Digest는 N/A다. Goal ID는 이 Goal Set의 stable member key다. 새 구현·게임 시험·감사는 아직 수행하지 않았다.

## Goal Set

- Goal Set Ref: `GS-CARL-P2G50-001`
- Predecessor Goal Set Ref: N/A — initial R05 Goal Set.
- Exact Prerequisite Composite Basis: Working Revision Ref `WR-CARL-P2G20-001`; Final Policy Freeze Ref `FPF-CARL-P2G30-001`; Phase Plan Ref `PP-CARL-P2G40-002`; Goal Set Ref N/A; Authoritative Change Obligation Declaration Ref Set `[]`.
- Goal Structure Fact: `single_phase`
- Phase Plan Ref: `PP-CARL-P2G40-002`
- Final Policy Freeze Ref: `FPF-CARL-P2G30-001`
- Phase Plan Validation Receipt Ref: `PVR-CARL-P2G40-002`
- Complete Goal Member Set: `GOAL-CARL-01` — 아래 Single member 전체 payload 하나.
- Applicable P2G-50 Obligation Key Set: explicit empty `[]`.
- Goal Quality Contract Version: `2`
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No
- User-Approved Validation Baseline Reuse Directive Ref: N/A — 별도 initial/final full 횟수 directive 없음. 채택 AUTORUN의 변경 영향에 비례한 검증·정확히 같은 evidence의 재사용과 GEX currentness를 적용한다.

입력은 [Freeze](P2G_30_FREEZE_001.json), [Plan](P2G_40_PHASE_PLAN_002.json), passed [Receipt](P2G_40_VALIDATION_RECEIPT_002.json), [코드 검토](CODE_REVIEW_001.ko.md)다. P2G가 사용자 요청의 별도 GEX 권한을 만들거나 실행 target을 활성화하지 않는다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| PHASE-CARL-01 | GOAL-CARL-01 | AG-CARL-01, WP-CARL-04 | 1 Phase → 1 Single Goal |

### Integration Acceptance-to-Goal Mapping

Plan의12 integration member를 그대로 인계한다. 모든 durable seam 구현과 실제 시험은 이 Goal의 WP04 소유다. source 수정은 AG로 반환하며 감사자는 production/test를 고치지 않는다.

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |
| IVA-CARL-01 / Bound Freeze member selector/acceptance (1-based): C01; E01 all; E08.1; CRA-CARL-P2G20-001 exact C sections | assigned_execution_node | GOAL-CARL-01 / WP-CARL-04 | Actual two copies of the same equipment definition, two employees, source provider/executor distinction, same copy redeployed. Invalid/missing/mismatched persistent owner refuses without touching another owner. Compare observed owner and value with actual UUIDs. Exact source hash/toolchain/seed/input/command/exit/actual nonzero count and expected-versus-actual assertions/raw output. Audit is a separate read-only pass after stabilization; failure returns to implementation. |
| IVA-CARL-02 / Bound Freeze member selector/acceptance (1-based): C02; E02.1/4; CRA-CARL-P2G20-001 exact C sections | assigned_execution_node | GOAL-CARL-01 / WP-CARL-04 | External declarations for Run/Attempt/Deployment: consume, withdraw, redeploy, battle end/retry; Deployment resets while Attempt/Run persist at their approved boundaries. Assert real consumers and registry entries/leases removed at their own end. Exact source hash/toolchain/seed/input/command/exit/actual nonzero count and expected-versus-actual assertions/raw output. Audit is a separate read-only pass after stabilization; failure returns to implementation. |
| IVA-CARL-03 / Bound Freeze member selector/acceptance (1-based): C02/C04; E02.2/3/4; CRA-CARL-P2G20-001 exact C sections | assigned_execution_node | GOAL-CARL-01 / WP-CARL-04 | Actual accepted/cancelled cast plus released projectile/area/queued completion: Activation ends at control terminal; Delivery lives until actual payload end. End twice/stale timer/new attempt same diagnostic ordinal does not affect new state. No uncreated child survives cancellation. Exact source hash/toolchain/seed/input/command/exit/actual nonzero count and expected-versus-actual assertions/raw output. Audit is a separate read-only pass after stabilization; failure returns to implementation. |
| IVA-CARL-04 / Bound Freeze member selector/acceptance (1-based): C02/C09; E02.2/4; E09.1; CRA-CARL-P2G20-001 exact C sections | assigned_execution_node | GOAL-CARL-01 / WP-CARL-04 | Real trigger/cast/delayed child chain maintains root context across source replacement and closes after finite descendants. Independent roots isolate; no log inference; queue budget/refund/cancel paths reclaim exactly once. Full Flow/relic join is not the test substitute. Exact source hash/toolchain/seed/input/command/exit/actual nonzero count and expected-versus-actual assertions/raw output. Audit is a separate read-only pass after stabilization; failure returns to implementation. |
| IVA-CARL-05 / Bound Freeze member selector/acceptance (1-based): C03/C04; E03 all; CRA-CARL-P2G20-001 exact C sections | assigned_execution_node | GOAL-CARL-01 / WP-CARL-04 | Authored explicit initial/cap and grant/decay/consume boundary values; requested versus actual delta. Reject missing/invalid/nonfinite/overflow/unknown resource and owner/lifetime context. Existing resonance/tactical behavior unchanged; fixture values not official balance. Exact source hash/toolchain/seed/input/command/exit/actual nonzero count and expected-versus-actual assertions/raw output. Audit is a separate read-only pass after stabilization; failure returns to implementation. |
| IVA-CARL-06 / Bound Freeze member selector/acceptance (1-based): C04/C09; E04 all; E09.2; CRA-CARL-P2G20-001 exact C sections | assigned_execution_node | GOAL-CARL-01 / WP-CARL-04 | Multiple resource costs with second insufficient; reserve competition, commit/abort repeated and stale epoch; injected admission/clone/receipt/queue fault. Compare entire authoritative state/RNG/UUID/revision/queue/observations before and after. Prior committed timestamp prefix remains; truthful Cost/ResourceChange source. Exact source hash/toolchain/seed/input/command/exit/actual nonzero count and expected-versus-actual assertions/raw output. Audit is a separate read-only pass after stabilization; failure returns to implementation. |
| IVA-CARL-07 / Bound Freeze member selector/acceptance (1-based): C05; E05 all; E06.5; CRA-CARL-P2G20-001 exact C sections | assigned_execution_node | GOAL-CARL-01 / WP-CARL-04 | Actual starter/grant/shop/reward/recruit/admin acquisition, same-item unequip/transfer/enhance retains balance; successful sale/dismantle deletes only its resource and yields existing reward; rejected operations preserve copy/store. Employee status change retaining UUID preserves its Run resource; no unrelated records lost. Exact source hash/toolchain/seed/input/command/exit/actual nonzero count and expected-versus-actual assertions/raw output. Audit is a separate read-only pass after stabilization; failure returns to implementation. |
| IVA-CARL-08 / Bound Freeze member selector/acceptance (1-based): C06; E06.1/2/3; CRA-CARL-P2G20-001 exact C sections | assigned_execution_node | GOAL-CARL-01 / WP-CARL-04 | Normal/event battles, victory/defeat/result retry: capture overlay before battle deletion; CompleteCombatResult twice and failure then retry apply resource exactly once. Entry/retreat restores value/RNG/UUID; unit withdrawal does not rollback Run. Floor transition preserves values; new Run does not inherit old authority. Exact source hash/toolchain/seed/input/command/exit/actual nonzero count and expected-versus-actual assertions/raw output. Audit is a separate read-only pass after stabilization; failure returns to implementation. |
| IVA-CARL-09 / Bound Freeze member selector/acceptance (1-based): C06/C07; E06.1/4; E07 all; CRA-CARL-P2G20-001 exact C sections | assigned_execution_node | GOAL-CARL-01 / WP-CARL-04 | New public application export/continue round-trip with same content/contract restores actual owner/value/progress; duplicate/missing owner/key/cap violation/checksum/mismatched fingerprint/same ID different resource contract/old version reject with prior app/profile/input unchanged. Preserve loads_used/run peak exceptions. Exact source hash/toolchain/seed/input/command/exit/actual nonzero count and expected-versus-actual assertions/raw output. Audit is a separate read-only pass after stabilization; failure returns to implementation. |
| IVA-CARL-10 / Bound Freeze member selector/acceptance (1-based): C03/C08; E08 all; E03.1; CRA-CARL-P2G20-001 exact C sections | assigned_execution_node | GOAL-CARL-01 / WP-CARL-04 | Independent external RON pack through default production API/CLI; mutate initial/cap/operation/new ID and observe actual trial gameplay/receipts for six lifetimes and real copy owners. Descriptor/view parity, negative reference/context/opcode inputs with source diagnostics, official pack load/build and literal-edit regression. Exact source hash/toolchain/seed/input/command/exit/actual nonzero count and expected-versus-actual assertions/raw output. Audit is a separate read-only pass after stabilization; failure returns to implementation. |
| IVA-CARL-11 / Bound Freeze member selector/acceptance (1-based): C04/C09; E09.1/2; CRA-CARL-P2G20-001 exact C sections | assigned_execution_node | GOAL-CARL-01 / WP-CARL-04 | Realistic and adversarial state/reservation/lease/queue/result/receipt workloads: measured finite retained/native/public clone/traversal/output bounds and typed fault unchanged candidate. Same seed/input with altered container/pack iteration and log on/off/eviction preserves approved gameplay/RNG. Registry/ticket memory reclaimed. Exact source hash/toolchain/seed/input/command/exit/actual nonzero count and expected-versus-actual assertions/raw output. Audit is a separate read-only pass after stabilization; failure returns to implementation. |
| IVA-CARL-12 / Bound Freeze member selector/acceptance (1-based): C09; E09.3/4; all freeze acceptances; CRA-CARL-P2G20-001 exact C sections | assigned_execution_node | GOAL-CARL-01 / WP-CARL-04 | Durable completion matrix across all R05 card/T05/T20/T23/T29 and 33 frozen acceptance members with exact source/toolchain/commands/raw outputs/nonzero counts. Required direct-predecessor regression and integration/build checks. Stable exact source/evidence manifest permits separate fresh implementation-nonparticipant audit; no production change delegated to auditor. Exact source hash/toolchain/seed/input/command/exit/actual nonzero count and expected-versus-actual assertions/raw output. Audit is a separate read-only pass after stabilization; failure returns to implementation. |

### Single Goal Member — GOAL-CARL-01

- Goal ID: `GOAL-CARL-01`
- Goal Kind: single
- Phase ID: `PHASE-CARL-01`
- Assigned Final Execution Node Set: `AG-CARL-01`, `WP-CARL-04` (AG members WP01/WP02/WP03 are not separately assigned execution nodes).
- Covered Policy Element Revision Ref Set: `PER-CARL-E01-REVIEWED-001`, `PER-CARL-E02-REVIEWED-001`, `PER-CARL-E03-REVIEWED-001`, `PER-CARL-E04-REVIEWED-001`, `PER-CARL-E05-REVIEWED-001`, `PER-CARL-E06-REVIEWED-001`, `PER-CARL-E07-REVIEWED-001`, `PER-CARL-E08-REVIEWED-001`, `PER-CARL-E09-REVIEWED-001`.
- Covered Decision Revision Ref Set: `DR-CARL-E01-001`, `DR-CARL-E02-001`, `DR-CARL-E03-001`, `DR-CARL-E04-001`, `DR-CARL-E05-001`, `DR-CARL-E06-001`, `DR-CARL-E07-001`, `DR-CARL-E08-001`, `DR-CARL-E09-001`.
- Derived Observable Policy Acceptance Coverage: all33 acceptance members of the exact nine Freeze members, projected unchanged in the Acceptance Completion Matrix below; IVA01–12 and WP04 own actual verification.
- Objective / Contract-Visible Result: 외부 RON으로 선언한 typed resource가 실제 소유자와 여섯 수명을 따라 실행되고, 예약/비용은 원자적으로 처리되며, Run 값이 실제 결과·후퇴·체크포인트·장비 사본 전환에서 승인된 의미대로 보존/폐기되는 core와 public 작성 포트를 완성한다.
- User-Visible Problem: 현재 공명·전술 포인트 외 자원의 공통 작성·수명·사본 owner·Run 저장 경계가 없다. 작성자가 같은 장비 두 사본을 별도로 추적하거나 배치/도전/실행/전달/연쇄별 잔량을 선언하고 실제 결과와 복원을 검증할 수 없다.
- User-Visible Completion Condition Set: C01 실제 두 사본/직원 owner의 값·source가 구별되고 부정 binding은 거부된다; C02 여섯 lifetime의 실제 작성/소비/종료/취소·stale 작업이 검증된다; C03 cap/grant/decay/consume와 reserve/commit/abort가 부분 지출·이중 지출 없이 정확한 before/after를 낸다; C04 같은 사본의 유지/판매·분해와 직원 상태 변화가 승인된 보존/폐기 정책을 지킨다; C05 Run writer/reader/validator/entry/result/rollback/층 이동이 함께 작동하고 결과를 한 번 반영한다; C06 production 외부 RON/API/CLI/descriptor가 같은 실제 기능과 부정 입력 진단을 제공한다; C07 현실/adversarial 입력의 비용·clone·queue·receipt·결정론·실패 복원을 실제 증거로 충족한다; C08 전체 frozen acceptance/R05 카드/T05/T20/T23/T29/직접 회귀·canonical 문서와 별도 fresh 감사가 완료된다. 각 조건은 개별 불충족을 식별하며 모두 충족해야 Goal 전체 완료다.
- In-Scope: Frozen E01–09, Plan WP01–04/AG01/SC01–03/IVA01–12. 새 immutable resource 정의, typed actual owner, 여섯 lifetime registry/epoch/lease, 값·예약·receipt, 실제 battle hooks·causal context, world Run/owner/result/entry/save/clone/검증, external RON/public/default-release/official pack 경로, durable tests/probes·측정·필수 회귀·문서·fresh audit.
- Out-of-Scope: 다른 R 전체 기능, R07 fragment-copy 집계·유물/변신/full Flow/fork-join gameplay, 서버/Unity/remote deployment/editor UI/global configuration. 새 보상/비용/확률/공식 cap/상속·환불, 저장 자동변환/삭제, definition-ID 소유자 추정, placeholder 실행기와 지원 성공 fallback. P2G/GEX/검증기/AUTORUN/AGENTS/global 지침 변경.
- Reviewed Current Behavior / Implementation Delta: CRA-CARL-P2G20-001 C01–09의44파일 exact source를 입력으로 한다. 실제 장비/직원 UUID가 draft/item/source까지 전달되고 action-control/finite-delivery 종료·candidate transaction·latest V6 save와 strict external pack loader가 존재한다. generic resource와 여섯 상태·예약·persistent Run delta는 없다. TriggerAbility/new cast source는 causal context를 reset하며 terminal core가 result 완료 전에 제거된다. 기존 경계를 재사용하면서 C01–09의 store/registry/copy binding/terminal payload/AT05/production authoring/cost를 공동 전환한다. source가 변하면 기대된 realization과 premise 변경을 구별한다.
- Goal-Local Source of Truth / Owning Layer: 정책 의미는 exact Freeze의 PER/Decision, 구조는 exact Phase Plan이다. immutable catalog는 선언 authority, mutable resource value/reservation/lifetime authority는 world/battle 기존 candidate owner다. inventory/roster actual UUID graph가 영구 owner를 식별한다. RuntimeFactSource와 관찰 receipt는 출처 전달·설명이지 spending authority가 아니다. Result transaction만 Run battle overlay를 반영하며 entry/checkpoint owner가 승인된 timeline을 복원한다.
- Goal-Local Invariant Set / Source Refs: I01 actual owner/definition/lifetime 분리(E01/C01/SC01); I02 자기 lifetime 종료·stale 차단 및 released payload 독립성(E02/C02/SC01); I03 explicit initial/finite cap·실제 grant/decay·부족 전체 거부(E03/C03); I04 reservation/cost와 queue/RNG/UUID/revision/관찰 동일 candidate commit(E04/C04/SC01); I05 사본 보존·실제 삭제의 동일 transaction과 보상 불변(E05/C05/SC02); I06 Run result exactly-once·entry/retreat/checkpoint/floor/terminal 일관성(E06/C06/SC02); I07 latest content/save 계약·owner graph 검증과 미지원 거부(E07/C07/SC02); I08 실제 외부 작성→생산자→공개 소비자와 유한 결정론적 상태/출력(E08–09/C08–09/SC03). Source Ref는 FPF-CARL-P2G30-001의 해당 member와 PP-CARL-P2G40-002 SC selector이며 새 정책을 추가하지 않는다.
- Code/Data/Schema/Store/Transport/Consumer Boundary: WP01 game/ability·data resource 선언과 battle core/resource registry/candidate/queue/sim/triggers/cast/projectile/area/death/withdraw를 연결한다. WP02 world application/grant/owner graph, combat terminal/staged result, RunState floor transfer, entry/checkpoint active_run writer/reader/validator와 CloneShape를 연결한다. WP03 content_pack raw/resolver/assemble/semantic identity/live validation/database/packaging, target usefulness/effect executor, authoring describe/view/API/CLI를 연결한다. C01–09/SC01–03은 제안 구조이며 실제 새 API가 아니다. GEX-10에서 각 producer→state→consumer 상세 설계·실패/종료 순서와 finite admission 준비를 확인한 뒤 AG를 구현한다. 파일명/enum/index/grammar/limit/version 수치는 기술 재량이나 공유 owner/lifetime/저장/Phase 경계를 바꾸면 P2G owner로 반환한다.
- Consumer Verification Boundary: Workspace-confirmed required는 actual equipment/employee constructors·battle executor/queue/지연 payload·world result/entry/save/floor·selected pack loader/official pack/authoring public API/CLI/borrowed view다. 이 전체를 실제 입력/상태/결과로 검증하며 아래 Consumer Boundary에서 구분한다. 외부 배포본과 서버/Unity는 O01 N/A이고 compatible로 판정하지 않는다. 존재나 상태가 unknown인 외부 소비자에 대한 성공을 주장하지 않는다.
- Structural Completion / Legacy and Fallback Disposition: 하나의 resource mutation/Run result authority와 실제 validated catalog 경로를 둔다. 새 resource에 hardcoded/test-only 실행, 로그 replay authority, guessed owner, implicit zero/unlimited, old-version dual reader/자동변환이 없어야 한다. 기존 공명/전술 포인트·action/delivery facts는 승인된 원래 역할을 보존하며 삭제/일반화 자체를 목표로 하지 않는다. future full Flow/relic 기능만 위한 미사용 추상화는 넣지 않는다. AG가 끝나기 전 partial resource capability를 지원 완료로 표시하지 않는다.
- Canonical Documentation Obligation: [Core Runtime Contract](../../core_runtime_contract.md)에 실제 resource owner/lifetime/예약·원자성/Run 결과·복원/유한 비용과 공개 관찰을 반영한다. [Data Loading Contract](../../data_loading_contract.md)에 실제 새 RON schema/capability/strict validation/official pack/latest save identity와 검증된 CLI 예제를 반영한다. [Skill Target Contract](../../skill_target_contract.md)는 자원-only 효과의 actual targeting 및 controller/released delivery 경계가 달라지는 부분만 갱신한다. 기존 승인 gameplay 문구와 역사적 evidence를 소급 변경하지 않는다. 예시는 실제 production 소비자로 검증한다.
- Goal-Local Dependencies / Execution Prerequisites: 이 Goal Set 안 선행 Goal은 없다. R01–R04 실제 완료는 intake/PREDECESSOR_REUSE_001.json의 scoped currentness·역사적 hash mismatch 영향평가를 GEX-00의 then-current source와 대조해 인수한다. exact Handoff·사용자 권한·실행 자격 확인 전 구현하지 않는다. 한 writer가 기존 dirty baseline을 보존한다. AG01 공유 계약의 상세 설계/소스·저장/비용 준비를 완료하고 atomic cutover한 뒤 WP04 실제 전체 증거와 fresh audit로 닫는다. 시작 prototype만으로 Phase 완료를 주장하지 않는다.
- Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan: 별도 shared baseline directive N/A. 기존 R01–R04 PASS는 exact source/fixture/toolchain/lock/claim이 같고 영향 밖인 부분만 재사용한다. P2G 정적 검사나 기존 resonance probe는 새 R05 PASS가 아니다. production 변경은 test 파일 불변이어도 해당 seam evidence를 invalid한다. fixture/probe/harness/serde/schema/clone/lock 변경은 영향 집합을 다시 실행한다. 아래 map대로 focused 검증하고 final integrated core/all-target/default-release/직접 외부 소비자 및 실제 finite cost를 검증한다. 날짜/Phase 진입만으로 full suite 반복을 요구하지 않는다.
- Assigned Durable Integration Validation Seam Coverage: IVA-CARL-01부터 IVA-CARL-12까지 전부 GOAL-CARL-01 / WP-CARL-04 / PHASE-CARL-01 소유다. 각 exact Plan row의 actual input/outcome/error/evidence contract를 인계한다. Tests/probes/negative fixtures와 측정 harness는 구현 중 작성하고 실행한다. 읽기 전용 감사에 source 생성/수정을 남기지 않는다.
- Applicable Migration/Compatibility/Rollout/Rollback: Freeze의 applicable transition disposition을 인수한다. 새 Run state가 생길 때 writer/reader/validator/entry/result/clone/rollback을 AT05로 함께 전환한다. 미지원 이전 버전/다른 fingerprint/owner는 원본 파일·application/profile 무변경 거부; 자동 변환/삭제 없음. 결과 retry는 한 번 반영, retreat는 pre-entry 복원, unit 재배치는 Run/Attempt 값을 되감지 않으며 floor는 같은 Run을 보존한다. 같은 copy의 해제/이전/강화 보존과 실제 판매/분해 자원 폐기를 지킨다. 외부 deployment/host save path/Unity wire migration은 N/A다.
- Goal-Local Known Unknown / Blocker / Stop Condition: KU-CARL-01 internal type/grammar/version/budget 선택은 GEX 상세 설계, KU-CARL-02 새 actual test/cost/audit는 WP04 및 fresh reviewer, KU-CARL-03 새 source의 predecessor 영향/currentness는 GEX 인수/완료 owner다. 이는 구현 의무이며 현재 성공 근거가 아니다. 미승인 gameplay/owner/lifetime/손실/호환 선택은 가장 이른 P2G owner로 반환하며 위임 조건 밖이면 사용자 결정 후 진행한다. 필수 source/실행/reviewer/evidence 부재나 진짜 환경 한계는 완료로 숨기지 않는다. 선택 R05 밖 새 작업으로 전환하지 않는다.
- Applicable Risk / Reviewer Requirement and Evidence: 위험은 같은 정의 사본 합침, re-equip/floor의 잔량 재초기화, 부족 비용의 부분 지출, candidate 밖 reservation/causal mutation, 끝난 activation이 delivery를 잘못 취소, chain context reset/stale generation alias, result 중복/소실, save owner graph/clone 필드 누락, unbounded queue/receipt와 unsupported public 성공이다. C01–09/SC01–03/IVA01–12가 대응 경계다. 구현·필수 시험을 멈추고 안정된 exact source/criteria/receipts를 구현에 참여하지 않은 별도 reviewer가 fresh 감사해야 한다. 같은 모델/컨텍스트 분리를 사실대로 기록한다. 자체 구현 요약을 독립 감사로 대신하지 않으며 필수 reviewer가 없으면 audit 미완료다.

#### Consumer Boundary

| 분류 | 실제 경계 | 완료 근거 |
| --- | --- | --- |
| workspace-confirmed / required | inventory/roster/starter/grant/recruit/loadout → actual player draft/runtime item/source | 실제 UUID의 값·receipt·유지/삭제가 다른 사본과 분리 |
| workspace-confirmed / required | battle cast/trigger/queue/projectile/area/control terminal/death/withdraw | 여섯 lifetime actual consumer, causal child 유지/자기 종료/stale 차단 |
| workspace-confirmed / required | world application/terminal result/CompleteCombatResult/entry/floor/checkpoint | source-owner 결과 한 번 반영·실패 복원·새 application save round-trip |
| workspace-confirmed / required | selected external pack parser/resolver/compile/identity/official package + authoring API/CLI/describe/view | 외부 RON 값 변화→실제 결과, 부정 입력 오류/무변경·동일 최신 계약 |
| excluded / N/A | 서버/Unity/외부 배포 compatibility, host 저장파일 population/OS path | O01/SR10–11에 의해 인증하지 않음; core bytes/public 소비 제외 아님 |

#### Acceptance Completion Matrix

각 exact PER의 acceptance를 그대로 projection한다. 모든 row의 실제 owner는 WP04이며 위 C01–08 전체 완료 조건과 함께 판정한다. 아래 index는1부터 시작한다.

| Exact member acceptance | Approved observable acceptance | Validation owner / seam |
| --- | --- | --- |
| `PER-CARL-E01-REVIEWED-001[1]` | 같은 정의의 장비 두 사본과 서로 다른 직원의 값·사용기록이 분리된다. 같은 사본의 서로 다른 배치도 같은 영구 owner로 추적된다. | IVA-CARL-01, IVA-CARL-10 / WP-CARL-04 |
| `PER-CARL-E01-REVIEWED-001[2]` | 자원 변경·비용 결과가 실제 source owner로 귀속되고 definition ID 또는 transient unit ID로 다른 사본과 합쳐지지 않는다. | IVA-CARL-01, IVA-CARL-06 / WP-CARL-04 |
| `PER-CARL-E01-REVIEWED-001[3]` | 실재하지 않거나 서로 맞지 않는 owner/binding은 명시적으로 거부되며 다른 소유자의 상태를 변경하지 않는다. | IVA-CARL-01, IVA-CARL-06 / WP-CARL-04 |
| `PER-CARL-E02-REVIEWED-001[1]` | 같은 사본의 Deployment 상태는 철수·재배치로 종료/초기화되고 Attempt 기록은 같은 전투 도전 안에서 유지된다. Run 상태는 Attempt·Deployment 종료만으로 초기화되지 않는다. | IVA-CARL-02, IVA-CARL-08 / WP-CARL-04 |
| `PER-CARL-E02-REVIEWED-001[2]` | Activation/Delivery/Chain 각각의 실제 시작·종료·취소 경계에서 상태와 예약/lease가 누출되지 않으며 stale handle/timer와 중복 종료는 새 인스턴스에 영향을 주지 않는다. | IVA-CARL-03, IVA-CARL-04 / WP-CARL-04 |
| `PER-CARL-E02-REVIEWED-001[3]` | 이미 방출한 전달체는 현재 승인된 독립 수명을 지킨다. controller 취소가 모든 released payload의 무조건 취소나 미생성 후속 효과의 존속으로 바뀌지 않는다. | IVA-CARL-03 / WP-CARL-04 |
| `PER-CARL-E02-REVIEWED-001[4]` | 여섯 수명 중 실제 consumer와 종료 검증이 없는 수명을 구현 완료로 표시하지 않는다. 후속 full Flow·branch/join 기능을 R05 완료의 대용으로 만들지 않는다. | IVA-CARL-02, IVA-CARL-03, IVA-CARL-04, IVA-CARL-10 / WP-CARL-04 |
| `PER-CARL-E03-REVIEWED-001[1]` | 유효 초기값은 0 이상 상한 이하이며 잘못된 타입·범위·누락·비유한 값·미지원 수명/owner 조합은 설명 가능한 거부다. 누락을 0 또는 무제한으로 메우지 않는다. | IVA-CARL-05, IVA-CARL-10 / WP-CARL-04 |
| `PER-CARL-E03-REVIEWED-001[2]` | grant/decay 결과는 요청량과 실제 변화량을 구별한다. 상한 초과분이나 없는 잔량을 환불·부채·다른 자원으로 변환하지 않는다. | IVA-CARL-05 / WP-CARL-04 |
| `PER-CARL-E03-REVIEWED-001[3]` | 상한보다 큰 잔량, 산술 overflow 또는 잘못된 비용 입력이 성공으로 게시되지 않는다. 실행 안전 예산과 authored gameplay cap을 구별한다. | IVA-CARL-05, IVA-CARL-06, IVA-CARL-11 / WP-CARL-04 |
| `PER-CARL-E03-REVIEWED-001[4]` | 기존 공명/전술 자원 동작은 보존한다. 시험 입력의 초기량·상한·소비량은 공식 밸런스 결정이 아니다. | IVA-CARL-05, IVA-CARL-12 / WP-CARL-04 |
| `PER-CARL-E04-REVIEWED-001[1]` | 여러 비용 중 하나가 부족하거나 중간 처리가 실패하면 부분 지출·다른 사본 지출·성공 receipt·ID/RNG/queue 누출이 없다. | IVA-CARL-06 / WP-CARL-04 |
| `PER-CARL-E04-REVIEWED-001[2]` | 동일 reservation의 중복 commit/abort 또는 stale revision/epoch는 추가 지출·환불·재생성을 일으키지 않는다. 예약된 잔량을 다른 소비가 다시 지출할 수 없다. | IVA-CARL-06 / WP-CARL-04 |
| `PER-CARL-E04-REVIEWED-001[3]` | Cost/ResourceChange 결과는 실제 before/after와 소유자·자원을 나타내며 로그 보존 여부가 원자성/권한/상태의 authority가 되지 않는다. | IVA-CARL-06, IVA-CARL-11 / WP-CARL-04 |
| `PER-CARL-E04-REVIEWED-001[4]` | 이미 완료된 이전 timestamp 전체를 rollback하는 것으로 과장하지 않고 현재 공개 transaction의 실패 경계를 검증한다. | IVA-CARL-06 / WP-CARL-04 |
| `PER-CARL-E05-REVIEWED-001[1]` | 동일 item UUID의 해제·다른 직원 장착·강화 전후에 Run 자원 잔량이 같다. 다른 직원/장비의 자원과 섞이지 않는다. | IVA-CARL-07 / WP-CARL-04 |
| `PER-CARL-E05-REVIEWED-001[2]` | 정상 판매·분해가 성공할 때 그 사본 전용 자원과 binding이 정리된다. 거부된 판매·분해에서는 사본과 자원이 모두 보존된다. | IVA-CARL-07 / WP-CARL-04 |
| `PER-CARL-E05-REVIEWED-001[3]` | 자원 잔량 때문에 판매/분해 보상량이 늘거나 다른 사본에 자원이 지급되지 않는다. 기존 판매/분해 정책의 보상·비용은 그대로다. | IVA-CARL-07 / WP-CARL-04 |
| `PER-CARL-E06-REVIEWED-001[1]` | 자원이 도입되는 시점에 save writer/reader/validator, entry capture/restore, result delta와 rollback이 함께 작동한다. 별도 후속 R을 기다려야 하는 저장 공백이 없다. | IVA-CARL-08, IVA-CARL-09 / WP-CARL-04 |
| `PER-CARL-E06-REVIEWED-001[2]` | 동일 owner의 전투 중 변화가 정상 승리·패배 결과 확정에서 한 번만 적용되고 같은 결과의 재적용은 자원 증감 중복을 만들지 않는다. | IVA-CARL-08 / WP-CARL-04 |
| `PER-CARL-E06-REVIEWED-001[3]` | retreat/retry 시 자원·attempt·RNG/UUID 등 기존 rollback 범위를 복원한다. RuntimeUnit 철수·재배치는 Run 자원을 되감지 않는다. | IVA-CARL-02, IVA-CARL-08 / WP-CARL-04 |
| `PER-CARL-E06-REVIEWED-001[4]` | 체크포인트 round-trip은 owner identity/잔량/세대 결속을 보존한다. 기존 loads_used·run peak·개인 profile 등 rollback 예외를 새 자원 복원에 섞어 되감지 않는다. | IVA-CARL-09 / WP-CARL-04 |
| `PER-CARL-E06-REVIEWED-001[5]` | 직원 상태 변화가 직원 identity를 제거하지 않는 한 해당 직원 Run 상태를 임의 소멸시키지 않는다. 다른 사본·독립 진행·보상 기록을 함께 삭제하지 않는다. | IVA-CARL-07 / WP-CARL-04 |
| `PER-CARL-E07-REVIEWED-001[1]` | 새 application에서 같은 유효 콘텐츠·계약의 새 Run 저장을 읽어 동일한 자원 owner/값/진행을 얻는다. compiled index/pointer가 영구 identity가 되지 않는다. | IVA-CARL-09 / WP-CARL-04 |
| `PER-CARL-E07-REVIEWED-001[2]` | 동일 정의 ID라도 fingerprint/자원 계약이 다르거나 저장 owner가 실재하지 않으면 기존 런·profile·입력 파일을 바꾸지 않고 실패한다. | IVA-CARL-09 / WP-CARL-04 |
| `PER-CARL-E07-REVIEWED-001[3]` | 기존 R02 latest-only 정책의 미지원 버전 거부를 유지하고 이전 저장을 암묵적으로 0 자원으로 변환하지 않는다. | IVA-CARL-09 / WP-CARL-04 |
| `PER-CARL-E08-REVIEWED-001[1]` | 같은 엔진에서 새 ID/같은 정의 두 사본의 외부 팩을 읽고 실제 authoring trial과 게임 실행으로 값·비용·수명 결과를 관찰한다. | IVA-CARL-01, IVA-CARL-10 / WP-CARL-04 |
| `PER-CARL-E08-REVIEWED-001[2]` | 지원하지 않는 자원 kind/owner/lifetime/operation 또는 잘못된 참조는 public validator에서 사실대로 거부되며 빈 executor/성공 fallback이 없다. | IVA-CARL-05, IVA-CARL-10 / WP-CARL-04 |
| `PER-CARL-E08-REVIEWED-001[3]` | 새 자원 결과는 직접 소비자·설명 포트에 일관되게 전달된다. 기존 공식 RON·schema 소비자와 같은 최신 경로를 사용한다. | IVA-CARL-10 / WP-CARL-04 |
| `PER-CARL-E09-REVIEWED-001[1]` | owner/container/pack 순회 순서와 진단 on/off가 승인된 게임 의미·RNG 권한을 바꾸지 않는다. stale/중복/부족 비용·취소·실패/restore를 실제 nonzero 시험으로 검증한다. | IVA-CARL-02, IVA-CARL-03, IVA-CARL-04, IVA-CARL-06, IVA-CARL-11 / WP-CARL-04 |
| `PER-CARL-E09-REVIEWED-001[2]` | 새 state/clone/receipt/queue 경계의 현실 입력과 adversarial 입력을 측정하고 경계 초과 시 typed fault와 transaction 무변경을 검증한다. 0 tests/skip/검증기 통과를 게임 성공으로 세지 않는다. | IVA-CARL-06, IVA-CARL-11 / WP-CARL-04 |
| `PER-CARL-E09-REVIEWED-001[3]` | T05/T20/T23/T29 및 R05 카드의 여섯 수명·실제 owner/저장 전환을 요구별 근거로 대조한다. 기존 단위 PASS나 작은 내부 store test만으로 전체 완료를 주장하지 않는다. | IVA-CARL-12 / WP-CARL-04 |
| `PER-CARL-E09-REVIEWED-001[4]` | 구현을 멈춘 별도 fresh 감사 pass와 적용 reviewer 조건을 충족한다. 구현에 참여하지 않은 reviewer의 감사 기록을 남기고 필수 미완료를 complete로 쓰지 않는다. | IVA-CARL-12 / WP-CARL-04 |

#### Command and Changed Input-to-Seam Map

명령은 GEX 실행 후보이며 cwd·toolchain·lock·실제 selector/count를 실행 시 고정한다. 새 R05 focused test/probe 이름은 구현 후 실재하는 것으로 대조한다. 아래 source/test/harness 변경은 관련 evidence를 invalid한다.

| 변경 입력 | 검증 command / actual seam | 범위·증거 |
| --- | --- | --- |
| resource/store/owner/reservation/lifecycle/queue/cast/trigger | `cargo test -p game_core --test skill_runtime_contract`; affected battle/lib tests plus new real six-lifetime and reservation tests | IVA01–06/11; before/after entire candidate and delayed payload lifecycle·RNG/IDs |
| inventory/roster/grant/deploy/sale/enhance/result/Run/entry/floor/save | `cargo test -p game_core --lib game::world`; `cargo test -p game_core --test live_item_skill_activation`; new public checkpoint/result resource fixtures | IVA01/02/07–09 and actual source-copy/currentness; content/hash and profile exceptions |
| schema/raw/resolver/compiler/identity/descriptor/executor/CLI | actual default-release external RON probe and negative packs; affected data/authoring tests; official pack produce/consume | IVA05/10; production load→execute→borrowed observation·error parity, not mocks |
| new state/source/clone/queue/result/receipt cost | native/public realistic/adversarial workload, plain release and separate allocator measurement where needed | IVA06/11; true time/retained/peak/reclaimed/count/source/fixture and typed failure unchanged candidate |
| shared schema/serde/clone/runtime harness/dependency or integrated final | `cargo test -p game_core --all-targets`; `cargo build -p game_core --release --bins`; direct production external consumer and assigned integration suite | IVA01–12/final integrated full proof; actual count and exit, no 0 tests/skip pass |
| docs or positive unchanged evidence | exact hash/claim/fixture/toolchain/lock identity, relevant public parser/example | No broad rerun only for date; implementation changes cannot reuse affected old runtime PASS |

#### Validation Dimension Matrix

| Dimension | Disposition | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |
| --- | --- | --- | --- |
| success | required | IVA01–10 actual external authored resource/store/owner/lifecycle/save/consumer | C01–06 each normal result is real and agrees with frozen policy |
| invalid_input | required | IVA01/05/06/09/10 invalid owner/ref/value/context/contract/schema | Typed rejection, no partial value/IDs/RNG/queue/receipt or original-file mutation |
| failure_rollback | required | IVA06/08/09/11 injected intermediate/admission/clone/restore failure | C03/C05/C07 same candidate atomicity, earlier committed timestamp prefix retained |
| retry_idempotency | required | IVA03/04/06/08 repeated reservation/end/result, stale epoch and failure then retry | No double spending/refund/result, old token cannot reach new instance |
| ordering_concurrency | required | IVA02–04/06/11 competing reservations/same-time events/root-child dispatch/owner iteration | Existing deterministic ordering and one writer; correct released child and life termination |
| boundary_empty_max | required | IVA05/06/09/11 empty/missing/zero/exact cap/max/overflow/finite admission | No implicit zero/unlimited, overflow success or unbounded queue/tombstone |
| disconnect_restart_persistence | required | IVA02/07–09 live clone/query, entry/retreat/floor and fresh process/application checkpoint | Same Run value/owner restored; invalidated runtime capability not serialized authority; profile exceptions |
| live_data | required | IVA05/10 external actual RON/default loader/official pack/invalid startup | Authored declaration reaches executor and supports truthful diagnostics/descriptors |
| producer_consumer | required | IVA01–12 actual source/owner/state/result/save/pack → public core/API/CLI/view | Real outputs and round-trip, not type existence or mock agreement |
| legacy_fallback_absence | required | IVA05/09/10/12 source audit and strict loader negative examples | No duplicate resource authority, implicit default/old-version conversion, log-backed Chain or empty executor |

## Policy–Phase–Goal Traceability

아래는 Freeze member와 Plan primary selector, 위 Phase-to-Goal 관계에서 계산한 검증기 입력용 조회다. 독립 정책 authority가 아니다.

| Stable Element ID | Exact Policy Element Revision | Decision Revision | Primary Phase | Goal Member |
| --- | --- | --- | --- | --- |
| CARL-E01 | PER-CARL-E01-REVIEWED-001 | DR-CARL-E01-001 | PHASE-CARL-01 | GOAL-CARL-01 |
| CARL-E02 | PER-CARL-E02-REVIEWED-001 | DR-CARL-E02-001 | PHASE-CARL-01 | GOAL-CARL-01 |
| CARL-E03 | PER-CARL-E03-REVIEWED-001 | DR-CARL-E03-001 | PHASE-CARL-01 | GOAL-CARL-01 |
| CARL-E04 | PER-CARL-E04-REVIEWED-001 | DR-CARL-E04-001 | PHASE-CARL-01 | GOAL-CARL-01 |
| CARL-E05 | PER-CARL-E05-REVIEWED-001 | DR-CARL-E05-001 | PHASE-CARL-01 | GOAL-CARL-01 |
| CARL-E06 | PER-CARL-E06-REVIEWED-001 | DR-CARL-E06-001 | PHASE-CARL-01 | GOAL-CARL-01 |
| CARL-E07 | PER-CARL-E07-REVIEWED-001 | DR-CARL-E07-001 | PHASE-CARL-01 | GOAL-CARL-01 |
| CARL-E08 | PER-CARL-E08-REVIEWED-001 | DR-CARL-E08-001 | PHASE-CARL-01 | GOAL-CARL-01 |
| CARL-E09 | PER-CARL-E09-REVIEWED-001 | DR-CARL-E09-001 | PHASE-CARL-01 | GOAL-CARL-01 |

## Goal Authoring Summary

유일 Phase의 두 Final Execution Node를 Single Goal member 하나에 배정했다. 9 PER/9 Decision/33 acceptance, 12 integration ownership, 10 required validation dimensions와8 독립 완료 조건을 인계했다. initial Goal Set으로 correction/repair는 없다. P2G-50 complete 검증 후에만 current로 선택한다. 다음 P2G-60은 사용자에게 이미 승인된 범위에서 exact Handoff를 작성하며 실제 GEX 인수/activation/구현은 별도 owner다. 이 작성은 R05 게임 구현·감사 완료를 뜻하지 않는다.
