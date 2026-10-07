# R03 공통 제작 API·CLI·설명·실제 시험 — Goal Set

2026-09-10 KST · P2G-50 · Goal Quality Contract 2. Goal 작성 결과다. 구현·실제 검증·완료 감사는 별도 eligible GEX가 소유한다. Typed Ref의 Artifact ID를 표시하며 producer Content Digest는 N/A다. Goal Set Ref는 identity envelope, Goal ID는 member key다.

## Goal Set

- Goal Set Ref: `GS-CATP-P2G50-001`
- Predecessor Goal Set Ref: N/A — 최초 R03 Goal Set.
- Exact Prerequisite Composite Basis: Working Revision Ref `WR-CATP-P2G20-001`; Final Policy Freeze Ref `FPF-CATP-P2G30-001`; Phase Plan Ref `PP-CATP-P2G40-001`; Goal Set Ref N/A; Authoritative Change Obligation Declaration Ref Set `[]`.
- Goal Structure Fact: `single_phase`
- Phase Plan Ref: `PP-CATP-P2G40-001`
- Final Policy Freeze Ref: `FPF-CATP-P2G30-001`
- Complete Goal Member Set: 아래 Single Goal Member `GOAL-CATP-01` 전체 payload 하나.
- Applicable P2G-50 Obligation Key Set: `[]`
- Goal Quality Contract Version: `2`
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No
- User-Approved Validation Baseline Reuse Directive Ref: N/A — 별도 shared initial/final 횟수 directive 없음. AUTORUN의 변경 영향에 따른 검증·unchanged evidence 재사용과 GEX currentness를 적용한다.

입력: [Freeze](P2G_30_FREEZE_001.json), [Plan](P2G_40_PHASE_PLAN_001.json), passed [Receipt](P2G_40_VALIDATION_RECEIPT_001.json), `CRA-CATP-P2G20-001`의 [코드 검토](CODE_REVIEW_001.ko.md). 이 작성 문서는 상위 사용자의 별도 GEX 권한이나 실제 GEX 자격 인수를 대신하지 않는다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| PHASE-CATP-01 | GOAL-CATP-01 | AG-CATP-01, WP-CATP-03 | 1 Phase → 1 Single Goal |

### Integration Acceptance-to-Goal Mapping

Plan의 9개 integration row를 그대로 인계한다. 상세 prerequisite/fixture/evidence는 각 owning Plan member가 소유한다.

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |
| IVA-CATP-01 / V01; Freeze Binding Completeness Evidence / acceptance-to-seam projection | assigned_execution_node | GOAL-CATP-01 / WP-CATP-03 | 같은 독립 external pack을 default-release public library와 CLI로 validate/describe/explain. 실제 token/fingerprint/source/지원 의미 일치; current R01 editable 조합 positive/unsupported negative; 다른 kind/compiled graph 설명 isolated reproducible external input, public API/CLI or focused owner assertions; exact source/workload/toolchain/command/nonzero count/exit/receipt. No personal store or fixture-only production. Source corrections return to AG; predecessor/final fresh audits are separate read-only passes. |
| IVA-CATP-02 / V02; Freeze Binding Completeness Evidence / acceptance-to-seam projection | assigned_execution_node | GOAL-CATP-01 / WP-CATP-03 | syntax/unknown field/malformed manifest·contract/capability/ambiguous winner/final dangling reference/summoned-boss compile/generated policy-wave·custom boss source/trial invalid authority의 같은 입력. API/CLI code/phase/source/related/available expected-actual parity, unknown precision 부재, zero candidate publication isolated reproducible external input, public API/CLI or focused owner assertions; exact source/workload/toolchain/command/nonzero count/exit/receipt. No personal store or fixture-only production. Source corrections return to AG; predecessor/final fresh audits are separate read-only passes. |
| IVA-CATP-03 / V03; Freeze Binding Completeness Evidence / acceptance-to-seam projection | assigned_execution_node | GOAL-CATP-01 / WP-CATP-03 | source 사본의 actual effect 수치만 바꾼 A/B를 일반 public scenario로 준비하고 nonzero advance/실제 effect execution까지 호출. 관찰 HP/resource/result delta와 actual source/fingerprint 결속, actual commands/nonzero counts/exit와 raw receipt 저장 isolated reproducible external input, public API/CLI or focused owner assertions; exact source/workload/toolchain/command/nonzero count/exit/receipt. No personal store or fixture-only production. Source corrections return to AG; predecessor/final fresh audits are separate read-only passes. |
| IVA-CATP-04 / V04; Freeze Binding Completeness Evidence / acceptance-to-seam projection | assigned_execution_node | GOAL-CATP-01 / WP-CATP-03 | 같은 initial/player/profile provenance/content/seed/template+resolved input/clock 재실행; trace on/off와 repeated/delayed/area, log eviction/terminal source, semantic identity가 log correlation에 의존하지 않음. 동일 gameplay/RNG/next decision 및 정확한 gap 표시 isolated reproducible external input, public API/CLI or focused owner assertions; exact source/workload/toolchain/command/nonzero count/exit/receipt. No personal store or fixture-only production. Source corrections return to AG; predecessor/final fresh audits are separate read-only passes. |
| IVA-CATP-05 / V05; Freeze Binding Completeness Evidence / acceptance-to-seam projection | assigned_execution_node | GOAL-CATP-01 / WP-CATP-03 | exact definition/effect/origin/compiled graph 설명과 readonly Preview 반복 전후 공개 snapshot/reconnect/revision, 이어지는 같은 command/clock 결과 일치. 누락 정의·unsupported query는 명시 거절, 가짜 forecast 없음 isolated reproducible external input, public API/CLI or focused owner assertions; exact source/workload/toolchain/command/nonzero count/exit/receipt. No personal store or fixture-only production. Source corrections return to AG; predecessor/final fresh audits are separate read-only passes. |
| IVA-CATP-06 / V06; Freeze Binding Completeness Evidence / acceptance-to-seam projection | assigned_execution_node | GOAL-CATP-01 / WP-CATP-03 | 실제 A→B 성공; bad syntax/compile/generated B 및 valid token+실제 trial prep 실패 B를 각각 반복; time-zero/출력 한계/취소 실패 A token/state/seed/input/receipt 불변; 이후 valid B 회복. 실패 단계 앞의 committed 성공 보존; 소유 참조 drop 확인 isolated reproducible external input, public API/CLI or focused owner assertions; exact source/workload/toolchain/command/nonzero count/exit/receipt. No personal store or fixture-only production. Source corrections return to AG; predecessor/final fresh audits are separate read-only passes. |
| IVA-CATP-07 / V07; Freeze Binding Completeness Evidence / acceptance-to-seam projection | assigned_execution_node | GOAL-CATP-01 / WP-CATP-03 | 빈/최소/정확 cap/cap+1/잘못된 nested request/binding/clock overflow, load-generated setup과 valid-token trial startup에서 각각 낮은 fuel/취소, same-time 반복·command/deferred fan-out·queue growth의 작은 fuel 한계와 취소. 한 호출 내부에서 실제 중단·명시 오류·rollback, trace-on만 optional cap을 넘는 입력에서도 trace-off와 commit/fuel/continuation/RNG 일치 및 gap, mandatory receipt 오류는 양쪽 후보 미게시 isolated reproducible external input, public API/CLI or focused owner assertions; exact source/workload/toolchain/command/nonzero count/exit/receipt. No personal store or fixture-only production. Source corrections return to AG; predecessor/final fresh audits are separate read-only passes. |
| IVA-CATP-08 / V08; Freeze Binding Completeness Evidence / acceptance-to-seam projection | assigned_execution_node | GOAL-CATP-01 / WP-CATP-03 | 기존 meaningful core suite, 관련 R01 effect/provenance/delayed·R02 loader/pin/V6/profile/failed-candidate invariants, all-target/default release와 실제 독립 positive API/negative privacy consumers. Source 변화로 침범한 선행 currentness·fresh audit 복구 isolated reproducible external input, public API/CLI or focused owner assertions; exact source/workload/toolchain/command/nonzero count/exit/receipt. No personal store or fixture-only production. Source corrections return to AG; predecessor/final fresh audits are separate read-only passes. |
| IVA-CATP-09 / V09; Freeze Binding Completeness Evidence / acceptance-to-seam projection | assigned_execution_node | GOAL-CATP-01 / WP-CATP-03 | 새 load/source inventory/trace/candidate clone/fuel의 실제 시간·할당·peak/retained/generation count와 비용을 source/toolchain/workload별 기록. Plain latency와 allocator timing을 분리. 기존 baseline/guards는 actual impact를 대조하여 재실행하거나 exact non-impact reuse 근거를 제시 isolated reproducible external input, public API/CLI or focused owner assertions; exact source/workload/toolchain/command/nonzero count/exit/receipt. No personal store or fixture-only production. Source corrections return to AG; predecessor/final fresh audits are separate read-only passes. |

### Single Goal Member — GOAL-CATP-01

- Goal ID: `GOAL-CATP-01`
- Goal Kind: `single`
- Phase ID: `PHASE-CATP-01`
- Assigned Final Execution Node Set: `AG-CATP-01`, `WP-CATP-03`
- Covered Policy Element Revision Ref Set: `PER-CATP-E01-REVIEWED-001`, `PER-CATP-E02-REVIEWED-001`, `PER-CATP-E03-REVIEWED-001`, `PER-CATP-E04-REVIEWED-001`, `PER-CATP-E05-REVIEWED-001`, `PER-CATP-E06-REVIEWED-001`
- Covered Decision Revision Ref Set: `DR-CATP-E01-001`, `DR-CATP-E02-001`, `DR-CATP-E03-001`, `DR-CATP-E04-001`, `DR-CATP-E05-001`, `DR-CATP-E06-001`
- Derived Observable Policy Acceptance Coverage: Freeze 6개 PER의 13 acceptance 전체. 아래 zero-based Acceptance Completion Matrix는 exact owned 문장의 projection이며 새 기준을 추가하지 않는다.
- Objective / Contract-Visible Result: 제작자는 공통 library와 같은 의미의 얇은 default-release CLI로 외부 콘텐츠를 검증·설명하고 실제 공개 코어 시험을 실행한다. Source-addressed 진단과 재현 가능한 seed/input/clock/fingerprint/실제 trace receipt를 얻으며 readonly query는 실행을 바꾸지 않고 새 B 시험 준비가 성공해야 A session이 교체된다.
- User-Visible Problem: 현재 external pack loader/public CoreApplication은 존재하지만 일반 authoring facade/CLI·structured code/phase/pack 진단·semantic simulation receipt·시험 reload는 없다. 기존 consume는 Map, public battle probe는 준비까지만 관찰한다. 일부 source는 JSON/terminal release에서 사라지고 host 호출 수만으로 내부 runtime work를 제한할 수 없다.
- User-Visible Completion Condition Set: C01 같은 production API/CLI의 validate/describe/explain/simulate와 current support, 독립 external release 소비; C02 실제 parse/structural/final/compile/generated/trial producer code/phase/known source/related/expected-actual parity 및 unknown precision 정직성; C03 외부 effect 변경의 실제 nonzero battle 관찰과 seed/initial authority/input/clock/content receipt 재현; C04 readonly query와 trace on/off가 state/RNG/revision/다음 결과를 유지하고 terminal/delayed/repeated occurrence를 실제 출처로 설명; C05 실제 B battle 준비 뒤 aggregate 교체, invalid/valid-load failed-prep/취소/필수 출력 실패에서 A 전체 보존과 회복; C06 finite input/binding/clock/work/queue/entity/receipt/trace 한계와 startup·단일 call 내부 중단, optional trace 초과는 gameplay 불변/gap; C07 도입 상태·세대 회수와 기존 pin/V6/profile/private API·개인/공식 source 보존; C08 실제 current source/commands/nonzero/receipts·회귀·비용 관찰·영향받은 선행 currentness 및 별도 fresh 완료 감사. 이들은 구현 뒤 판정할 조건이며 현재 PASS가 아니다.
- In-Scope: AG01의 WP01/02: common production authoring facade와 명시 Cargo CLI, producer diagnostics/current descriptor/immutable source-resolution explain, generic finite public scenario와 private opaque step staging, load-generated/setup/startup/command/deferred/clock opt-in control, actual semantic observation/terminal capture, isolated optional trace, mandatory receipt/complete B swap/drop. WP03은 IVA01–09·13 acceptance·current predecessors·실제 core/release/cost 검증을 소유한다.
- Out-of-Scope: 새 gameplay expressions/events/chain/flow/composition/공식 수치 등 R04–R24 후속 기능, editor UI·watcher·자동 진행 정책, server/Unity 수정·배포·개인 store 접근/삭제/자동 변환, durable midbattle 저장·raw fixture production·전용 콘텐츠 runner. R03의 실행 예산·실제 nonzero 시험·수명/실패 보존을 R23/R20으로 미루지 않는다.
- Reviewed Current Behavior / Implementation Delta: CRA B01–06/P01–07 소비. Public CoreApplication은 private GameCore의 전체 draft→projection→infallible commit을 사용하나 public Clone은 없다. R02 source/writer metadata 일부는 publication에서 폐기되고 LiveRonError는 typed parser/pack/code/phase를 잃는다. R01 source는 실제 cast/command/area/projectile에서 보존되나 serialized log와 release terminal에서 접근되지 않는다. Load-generated setup과 same-time drain/expansion까지 control을 전달하고 일반 scenario·readonly description·staged trace/receipt·실제 준비 후 session swap을 동반 구현한다.
- Goal-Local Source of Truth / Owning Layer: 정책은 exact Freeze PER/DR, 구조는 Plan WP/AG/SC01–06, 구현 권장은 CRA P01–07이다. Production loader/resolver/compiler가 validated token/identity/source를 소유하고 CoreApplication/domain draft만 gameplay authority를 확정한다. Authoring aggregate는 initial/request/step/receipt/control 수명을 소유하며 CLI는 같은 library adapter다. Source/trace metadata는 gameplay RNG·outcome·V6/profile authority가 아니다.
- Goal-Local Invariant Set / Source Refs: I01 SC01/E01/E02/E04: 동일 production validation과 실제 known source/현재 descriptor만 공개, actual writer hash/order 보존. I02 SC02/E02/E03/E05/E06: authoring opt-in control이 load-generated/startup/실제 expansion부터 적용되며 소진은 candidate 실패, ordinary gameplay timeout/Draw 아님. I03 SC03/E03/E05/E06: private opaque per-step fork와 mandatory result 준비 후 확정, B 실제 준비 성공 뒤 complete swap, 실패/취소 A와 quota/receipt 전체 보존. I04 SC04/E03/E04/E06: actual occurrence와 log correlation 분리, terminal 전 capture, trace allowance는 필수 receipt/admission/fuel/bindings에 영향 없음. I05 SC05/E04/E05/E06: process-local bounded/drop, current V6/profile/pin/privacy/개인 파일 보존. I06 SC06/Plan audit: 다음 R 없이 이 surface 사용 가능, 변경 입력의 선행/자체 evidence는 current로 복구. E 번호는 exact trace의 PER/DR다.
- Code/Data/Schema/Store/Transport/Consumer Boundary: CRA P01–07의 shared facade/request/error/source/control/observation 계약을 existing GEX 기록에서 상세화한다. LiveRon/manifest/raw/writer/final/compiler/generated/application producer에서 metadata를 보존하고 immutable generation metadata와 Explain lookup을 연결한다. Existing public command/snapshot/clock 의미를 유지하며 prior-result bounded pointer copies만 허용한다. CoreApplication private fork와 candidate-local control은 startup 전부터 실제 runtime에 전달하고 every exit/reset/commit을 명시한다. Actual effect application/delayed/terminal observation을 bounded owned DTO로 내보내며 optional trace와 mandatory receipt를 분리한다. New public module와 CLI를 명시 등록하되 raw inner borrow/fixture state를 노출하지 않는다. 개인 저장/wire DTO 변경 없이 existing V6/profile owner를 유지한다.
- Consumer Verification Boundary: 아래 workspace-confirmed core/direct public API/CLI consumers 전부 required. Canonical-external Unity/host와 unknown deployed actual은 별도 exclusion이며 호환 성공을 추정하지 않는다. 필수 producer/consumer가 inaccessible 또는 control/source/transaction에서 누락되면 완료할 수 없다.
- Goal-Local Dependencies / Execution Prerequisites: 실제 선행은 `(GS-CAMEC-P2G50-001, GOAL-CAMEC-01)`의 typed effect/support/source와 `(GS-CAPG-P2G50-001, GOAL-CAPG-01)`의 external selection/opaque generation/fingerprint/pin/V6이다. R01 ATR003/audit003과 R02 ATR003/audit002 Complete/High를 root가 실제 target/외부 입력/출력까지 rehash했다. GEX-00에서 현재 기준·권한·dirty attribution/baseline을 인수한다. AG의 두 WP를 함께 전환하고 WP03 검증이 뒤따른다. 구현 전에 exact types/codes/binding grammar/authority ownership/occurrence key/필수-선택 output budget/control propagation/queue cap/실패·취소·drop/변경 symbols와 실제 test oracle를 source에서 상세 설계하고 공유 계약 준비를 확인한다. 내부 기술 준비는 재승인 gate가 아니다.
- Validation Plan / Completion Matrix: WP03이 아래13 acceptance·IVA01–09·10 dimension을 실제 입력/producer/consumer/상태와 실패 관찰에 결속한다. Test selector는 구현 뒤 실제 이름과 nonzero count로 확정한다. Source/data/test/config/lock/toolchain/fixture/명령/exit/raw receipts를 보존하고 failure→AG 교정→영향 범위 재검증을 반복한다. Snapshot 기대값 갱신·문서/compile만·zero/ignored를 실행 성공으로 세지 않는다. 최종 안정된 source와 current predecessor 근거를 별도 fresh GEX-20이 판정한다.
- Structural Completion / Legacy and Fallback Disposition: 두 번째 validator/evaluator, content-name runner, fixture-only production builder/embedded fallback, 무제한 thread/history/cache, raw inner authority, log-seq RNG/semantic identity, source 문자열 재추론/미지원 임시 성공을 남기지 않는다. 기존 consume Map probe는 좁은 도구로 보존하며 새 generic simulator 성공과 구분한다. Source/support/CLI와 actual executor/control/session/trace consumer가 함께 유효해야 한다. 미래 전용 abstraction을 먼저 노출하거나 기존 V6/profile/private API를 약화하지 않는다.
- Canonical Documentation Obligation: [Data Loading Contract](../../data_loading_contract.md)에 실제 common API/CLI 입력·code/phase/precision/source metadata/current support·load-generated control을 반영하고 [Core Runtime Contract](../../core_runtime_contract.md)에 실제 tool staging/clock/semantic observation/terminal/caps/trace isolation/수명·readonly·기존 V6/pin/profile 보존과 새 측정 한계를 반영한다. 실행 가능한 minimal external scenario/API/CLI 예시를 실제 release consumer로 검증한다. 미래 기능을 현재 지원으로 표시하지 않는다.
- Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan: 별도 shared baseline directive N/A. Actual R02 VI027 full core1217 및 current R01/R02 audits는 일치하는 source/input/claim 범위만 재사용할 수 있으며 새 R03 PASS가 아니다. 아래 impact map의 production/runtime/metadata/source/test/harness/config/feature/lock 변화가 해당 evidence를 invalid한다. Final integrated full core lib/tests와 default release public consumers 및 새 control/observer 비용은 WP03 실제 검증 소유다. 새 Phase/날짜만으로 full suite를 반복하지 않고 관련 실패/변경/미완료 우려에 필요한 재실행을 수행한다.
- Assigned Durable Integration Validation Seam Coverage: `IVA-CATP-01`, `IVA-CATP-02`, `IVA-CATP-03`, `IVA-CATP-04`, `IVA-CATP-05`, `IVA-CATP-06`, `IVA-CATP-07`, `IVA-CATP-08`, `IVA-CATP-09` 전부 WP-CATP-03 / PHASE-CATP-01. IVA08은 actual R01/R02 영향·current regression과 fresh audit 복구, IVA09는 새 경로의 실측을 포함한다.
- Applicable Migration/Compatibility/Rollout/Rollback: Freeze TR30-01–06 인수. 공통 source/control/session/CLI 전환은 existing loader/public core semantics를 유지한다. Invalid load/compile/generated/startup/command/cancel/mandatory-output 실패는 후보 미게시와 A 보존이며 optional trace 제한은 gameplay failure가 아니다. Same seed/input/content replay는 새 session 재구성이고 durable live-battle 복원은 아니다. V6/profile/pin·개인 파일 유지, 외부 server/Unity migration/rollout compatibility는 N/A다.
- Goal-Local Known Unknown / Blocker / Stop Condition: KU30-01 exact 타입/limits/control은 AG가 CRA 경계 안에서, KU30-02 실제 tests/cost는 WP03이 실행하여 해소한다. KU30-03 영향받은 선행 currentness는 완료 전 필수다. KU30-04 excluded external actual은 호환 미주장으로 유지한다. Actual source/consumer/도입 state lifetime/실행 safety가 빠지거나 trace flag가 admission/RNG를 바꾸면 owner 구현 교정; source/Phase 근거를 유지할 수 없으면 earliest P2G owner 반환. Material gameplay/공식 balance/저장 손실/UX/호환/권한 확대는 사용자가 결정하며 내부 타입·같은 범위 실패는 자율 교정한다.
- Applicable Risk / Reviewer Requirement and Evidence: 위험은 source 정밀도 조작, load-before-control 누락, infallible helper의 partial success, mutable A/B counter 공유, terminal provenance 소실, optional trace의 receipt/fuel 간섭과 기존 runtime/세대 비용 회귀다. CRA의 독립 읽기 보조 지적 DR01–03/root closure와 Plan SC01–06/9 IVA가 대응한다. 구현 종료 후 안정된 exact source/criteria/evidence를 읽는 별도 fresh reviewer가 GEX-20을 수행한다. 필요한 predecessor currentness 감사도 separate fresh pass로 수행하며 root 자기 점검/설계 보조를 완료 감사로 표시하지 않는다.

#### Consumer Boundary

| 분류 | 경계 / 근거 | 완료 판단 |
| --- | --- | --- |
| workspace-confirmed / required | owned snapshot/typed parse/resolve/final compile/generated setup/public token | common controlled authoring load와 실제 source/code/phase·no-publication parity |
| workspace-confirmed / required | CoreApplication public commands/snapshot/clock와 private application draft/kernel | generic observed inputs, actual nonzero execution, source/occurrence/terminal/step commit/reload |
| workspace-confirmed / required | new thin registered default-release CLI와 standalone public consumer | 동일 common API 의미와 real external input 결과; fixture feature 없이 실행 |
| workspace-confirmed / required | R01 source/delayed/readonly 및 R02 pin/V6/profile, existing API privacy/performance consumers | actual changed-input regression/currentness and required fresh audits, 새 비용 실측 |
| canonical-external / excluded | canonical runtime 문서의 server/Unity wire·host storage consumers | core regression은 required, 외부 mapper/배포 호환·personal store 변경은 제외 |
| known-inaccessible | current required core source/consumer 중 확인된 inaccessible 대상 없음 | 필수 접근 공백 발견 시 성공 가정 없이 owner 반환 |
| unknown / excluded | 실제 deployed server/Unity와 개인 저장 population | compatibility/migration/rollout 성공 미주장 |

#### Acceptance Completion Matrix

PER member는 Freeze projection의 zero-based acceptance다. 원문과 owning IVA를 연결하며 현재 실행 증거가 아니다.

| Acceptance member locator | Completion condition / required observation | Integration source / durable owner |
| --- | --- | --- |
| `PER-CATP-E01-REVIEWED-001[0]` | 독립 외부 RON 입력이 API와 CLI에서 같은 production 로딩·컴파일·실행에 연결된다. 해당 IVA의 실제 결과·오류·보존 oracle를 관찰. | IVA-CATP-01, IVA-CATP-03 / WP-CATP-03 |
| `PER-CATP-E01-REVIEWED-001[1]` | 지원 설명은 실제 지원과 일치하고 미지원 조합은 명시적으로 거부된다. fixture-only 생성이나 콘텐츠 이름별 전용 runner로 성공을 대신하지 않는다. 해당 IVA의 실제 결과·오류·보존 oracle를 관찰. | IVA-CATP-01, IVA-CATP-02 / WP-CATP-03 |
| `PER-CATP-E02-REVIEWED-001[0]` | 같은 잘못된 입력에 API와 CLI가 동일 의미의 code/phase/source 진단을 제공한다. 해당 IVA의 실제 결과·오류·보존 oracle를 관찰. | IVA-CATP-01, IVA-CATP-02 / WP-CATP-03 |
| `PER-CATP-E02-REVIEWED-001[1]` | 구문·구조·최종 조합·컴파일·generated setup·시험 준비 실패는 알려진 실제 출처와 관련 입력을 보존하며 잘못된 후보를 성공으로 게시하지 않는다. 해당 IVA의 실제 결과·오류·보존 oracle를 관찰. | IVA-CATP-02, IVA-CATP-06, IVA-CATP-07 / WP-CATP-03 |
| `PER-CATP-E03-REVIEWED-001[0]` | 독립 외부 RON의 변경이 공통 공개 코어를 거친 실제 전투 실행과 관찰 결과에 반영된다. 문서·컴파일 성공만을 실행으로 세지 않는다. 해당 IVA의 실제 결과·오류·보존 oracle를 관찰. | IVA-CATP-03 / WP-CATP-03 |
| `PER-CATP-E03-REVIEWED-001[1]` | 같은 초기 authority·콘텐츠·seed·입력·clock은 적용 가능한 semantic 결과와 출처를 재현하고 trace 수집 여부가 gameplay 결과와 RNG를 바꾸지 않는다. 해당 IVA의 실제 결과·오류·보존 oracle를 관찰. | IVA-CATP-04, IVA-CATP-07 / WP-CATP-03 |
| `PER-CATP-E04-REVIEWED-001[0]` | Explain/Preview의 반복 전후 state·authority·revision과 이어지는 결정론적 실행이 유지된다. 해당 IVA의 실제 결과·오류·보존 oracle를 관찰. | IVA-CATP-04, IVA-CATP-05 / WP-CATP-03 |
| `PER-CATP-E04-REVIEWED-001[1]` | 실제 선택 정의·효과 origin과 관찰을 설명하며 미구현 후속 기능의 의미를 실행된 결과처럼 꾸미지 않는다. 해당 IVA의 실제 결과·오류·보존 oracle를 관찰. | IVA-CATP-01, IVA-CATP-05 / WP-CATP-03 |
| `PER-CATP-E05-REVIEWED-001[0]` | 정상 A→B reload는 B의 실제 시험 준비 뒤 교체되고 새 state·token·입력·receipt가 B와 일치한다. 해당 IVA의 실제 결과·오류·보존 oracle를 관찰. | IVA-CATP-06 / WP-CATP-03 |
| `PER-CATP-E05-REVIEWED-001[1]` | 잘못된 B와 검증은 성공했으나 시험 준비가 실패한 B는 A state·token·seed·입력·receipt를 반복 시도에서도 보존하며 이후 정상 B로 복구할 수 있다. 해당 IVA의 실제 결과·오류·보존 oracle를 관찰. | IVA-CATP-06, IVA-CATP-07 / WP-CATP-03 |
| `PER-CATP-E06-REVIEWED-001[0]` | 빈값·경계·초과·손상 요청의 결과가 실제 한계와 일치하고 실패가 유효한 시험 session을 교체하지 않는다. 해당 IVA의 실제 결과·오류·보존 oracle를 관찰. | IVA-CATP-02, IVA-CATP-06, IVA-CATP-07 / WP-CATP-03 |
| `PER-CATP-E06-REVIEWED-001[1]` | 시험 취소·교체 때 더 이상 참조하지 않는 도입 상태를 회수하며 기존 publication·pin·save·profile·공개 API 계약은 current로 유지된다. 해당 IVA의 실제 결과·오류·보존 oracle를 관찰. | IVA-CATP-06, IVA-CATP-08 / WP-CATP-03 |
| `PER-CATP-E06-REVIEWED-001[2]` | 실제 source·toolchain·workload·명령·nonzero 실행·receipt가 실행·재사용·남은 한계를 구별하고 prototype 수치는 공식 정책과 분리한다. 해당 IVA의 실제 결과·오류·보존 oracle를 관찰. | IVA-CATP-03, IVA-CATP-08, IVA-CATP-09 / WP-CATP-03 |

#### Command and Changed Input-to-Seam Map

아래 기존 target 기반 명령은 실행 후보이며 새 selector/binary 이름은 구현 후 확정한다. P2G에서 게임 검사는 실행하지 않았다.

| 입력 변화 | 후보 command / seam | 완료·재사용 경계 |
| --- | --- | --- |
| diagnostics/source/metadata/descriptor/explain/parser/loader | `cargo test -p game_core --lib game::data`; actual authoring API integration/CLI parity fixture | IVA01/02/05/07/08/09; actual phase/source/expected-actual·hash preservation/load-time work control, generated custom policy/boss regressions |
| actual runtime/control/observation/delayed/terminal | `cargo test -p game_core --test skill_runtime_contract`; source/proc/area/projectile/world focused tests; new generic external scenario | IVA03/04/07/08/09; same-call exhaustion and actual semantic/source/trace-on-off continuation; internal and public nonzero oracles |
| application fork/session/binding/reload/clock/output/drop | `cargo test -p game_core --lib game::world`; new library session integration test and release external consumer | IVA03–08; complete A authority/seed/quota/receipt and V6/profile preservation, actual prep failure/retry/cancel/final observation, no private builder |
| shared code/harness/config/feature/dependency/final target | `cargo check -p game_core --all-targets`; `cargo test -p game_core --lib --tests`; `cargo check -p game_core --release --lib`; actual registered authoring CLI release build/run and public API negatives | WP03 final full core and actual default-release API/CLI. Existing architecture guard workload/output/caps preserved; changed control/observer/clone path measured again |
| new memory/retention/limits or observation path | plain release and separate allocation-observer workload around load/prep/nonzero step/query/reload/cancel/drop | IVA09 and IVA07; exact sample/peak/end/count/input source with optional timing separated from gameplay; old measurements cannot certify new cost |
| documentation/example or unchanged evidence | exact relevant source/test/lock/toolchain/claim identity and real parser/public example check | only positive non-impact evidence reused; no automatic broad rerun for documentation-only changes |

#### Validation Dimension Matrix

| Dimension | Disposition (`required` / `N/A`) | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |
| --- | --- | --- | --- |
| success | required | IVA01/03/05/06: external A/B API+CLI→actual public battle→readonly explanation/reload | C01/C03–05: real changed effect execution, source identity/receipt and B prepared before swap |
| invalid_input | required | IVA02/07: syntax/structure/contract/capability/final/compile/generated/trial/unknown binding | C02/C06: API/CLI parity, exact known precision, no token/candidate publication; invalid B retains A |
| failure_rollback | required | IVA02/06/07: late startup/fuel/cancel/mandatory-result failure; separate optional trace overflow | C04–06: A complete authority/token/seed/quota/receipt and prior accepted steps retained; optional trace failure only gap, no false postcommit Err |
| retry_idempotency | required | IVA04/05/06: repeated exact run/readonly and rejected B retries then valid B | Same initial/source/input/clock yields same semantic continuation; failed attempt residuals absent; no new network dedupe/exactly-once semantics |
| ordering_concurrency | required | IVA03/04/06/07: actual command/event/clock order, same-time repeats/delayed/terminal and A/B separation | Existing order/authority/RNG maintained; cancellation input cannot mutate A quota; no parallel session writes approved; explicit source correlation vs semantic occurrence |
| boundary_empty_max | required | IVA02/07/09: empty/missing/minimum/exact cap/cap+1/overflow, nested bindings/clock, generated startup/same-call fuel and trace capacity | Finite checked limits with actual in-call stop and no partial success; trace-on at optional cap has same mandatory admission/commit/RNG as trace-off |
| disconnect_restart_persistence | required | IVA04/06/08: process-local cancel/drop/replay, readonly reconnect and existing V6/profile/pin guards | Disposable session lost at process end; explicit same-basis replay, no claimed live checkpoint restore; no personal files, same generation/lifetime and current storage semantics |
| live_data | required | IVA01/02/03/06: isolated real external RON through same default-release loader with authored effect change and bad B | Actual data affects nonzero battle and diagnostics; no embedded fill/content-ID runner/official source overwrite |
| producer_consumer | required | IVA01–09: source/compile/generated/start→public app/runtime→staged observation/result/session→API/CLI | Real cross-boundary output/phase/source/readonly/replay/retention/rollback, not mock-only or type existence; registered release consumer required |
| legacy_fallback_absence | required | IVA01/02/07/08: actual production dispatch/privacy/source and unsupported negatives | No second evaluator/fixture-only builder/unlimited host fallback/guessed source/empty executor/log-seq RNG/new storage alias; existing supported core reused |

## Policy–Phase–Goal Traceability

Exact Freeze membership/Decision과 Plan primary selector에서 파생하는 검증기 입력이다. 별도 정책 authority가 아니다.

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| CATP-E01 | PER-CATP-E01-REVIEWED-001 | DR-CATP-E01-001 | PHASE-CATP-01 | GOAL-CATP-01 |
| CATP-E02 | PER-CATP-E02-REVIEWED-001 | DR-CATP-E02-001 | PHASE-CATP-01 | GOAL-CATP-01 |
| CATP-E03 | PER-CATP-E03-REVIEWED-001 | DR-CATP-E03-001 | PHASE-CATP-01 | GOAL-CATP-01 |
| CATP-E04 | PER-CATP-E04-REVIEWED-001 | DR-CATP-E04-001 | PHASE-CATP-01 | GOAL-CATP-01 |
| CATP-E05 | PER-CATP-E05-REVIEWED-001 | DR-CATP-E05-001 | PHASE-CATP-01 | GOAL-CATP-01 |
| CATP-E06 | PER-CATP-E06-REVIEWED-001 | DR-CATP-E06-001 | PHASE-CATP-01 | GOAL-CATP-01 |

## Goal Authoring Summary

단일 Phase의 2 final nodes, 6정책·13acceptance·9 IVA·10 required dimensions를 하나의 Single Goal로 인계한다. 표준 구조 검사와 root 의미 검토를 통과한 complete 단계 결과만 current selection을 소유한다. Handoff/GEX 활성화/게임 PASS는 이 문서가 부여하지 않는다.
