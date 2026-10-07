# R02 외부 팩·검증 세대·새 런/저장 결속 — Goal Set

2026-09-10 KST · P2G-50 · Goal Quality Contract 2. Goal 작성 결과이며 구현·게임 검증·완료 감사는 GEX가 소유한다. `... Ref`는 해당 typed Artifact ID와 producer Content Digest N/A의 alias다. Goal Set Ref는 identity envelope이고 canonical payload에 자기 참조하지 않는다. Current 선택은 단계 결과가 소유한다.

## Goal Set

- Goal Set Ref: `GS-CAPG-P2G50-001`
- Predecessor Goal Set Ref: N/A — 최초 R02 Goal Set.
- Exact Prerequisite Composite Basis: Working Revision Ref `WR-CAPG-P2G20-001`; Final Policy Freeze Ref `FPF-CAPG-P2G30-001`; Phase Plan Ref `PP-CAPG-P2G40-001`; Goal Set Ref N/A; Authoritative Change Obligation Declaration Ref Set `[]`.
- Goal Structure Fact: `single_phase`
- Phase Plan Ref: `PP-CAPG-P2G40-001`
- Final Policy Freeze Ref: `FPF-CAPG-P2G30-001`
- Complete Goal Member Set: 아래 Single Goal Member `GOAL-CAPG-01`의 전체 payload 하나.
- Applicable P2G-50 Obligation Key Set: `[]`
- Goal Quality Contract Version: `2`
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No
- User-Approved Validation Baseline Reuse Directive Ref: N/A — 별도 shared initial/final 횟수 directive 없음. 사용자 AUTORUN의 비례 검증·unchanged evidence 재사용과 GEX invalidation 규칙은 아래 impact map에 반영.

입력: [Freeze](P2G_30_FREEZE_001.json), [Plan](P2G_40_PHASE_PLAN_001.json), passed [Receipt](P2G_40_PHASE_PLAN_VALIDATION_RECEIPT_001.json), `CRA-CAPG-P2G20-001`의 [코드 검토](CODE_REVIEW_001.ko.md). 이 P2G의 권한과 현재 사용자가 별도로 승인한 후속 eligible GEX를 구분한다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| PHASE-CAPG-01 | GOAL-CAPG-01 | AG-CAPG-01, WP-CAPG-03 | 1 Phase → 1 Single Goal |

### Integration Acceptance-to-Goal Mapping

Plan의 8개 integration row를 그대로 인계한다. 상세 locator/evidence/prerequisite는 각 owning Plan member가 소유한다.

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |
| IVA-CAPG-01 / V01; Freeze Binding Completeness Evidence / acceptance-to-seam projection | assigned_execution_node | GOAL-CAPG-01 / WP-CAPG-03 | 동일 바이너리·실제 A/B directory, 공식 pack packaging과 외부 원문 수정 → 실제 서로 다른 정의/실행 및 source path, embedded fallback 부재. 격리 공식 source·별도 output·기존 custom live_data fixture를 준비해 packaging 전후 source/custom bytes 보존을 대조. E01/E05/E06 reproducible isolated fixture/public API, command+nonzero count+terminal exit+source identity and direct assertions; no personal file access. V08 audit output is separate read-only auditor work, root retains implementation ownership. |
| IVA-CAPG-02 / V02; Freeze Binding Completeness Evidence / acceptance-to-seam projection | assigned_execution_node | GOAL-CAPG-01 / WP-CAPG-03 | 독립 namespace의 동일 local 이름, global override, recipe extension, explicit winner/chain, shuffled pack/file 목록 → exact typed key·안정된 실행 순서/지문. 참조를 함께 정리한 유효 Remove 뒤 최종 catalog·조회 index·등록/획득 소비자에서 제거 결과를 관찰. 모호 writer·중복/누락 dependency·cycle·없는 target는 오류. E02/E06 reproducible isolated fixture/public API, command+nonzero count+terminal exit+source identity and direct assertions; no personal file access. V08 audit output is separate read-only auditor work, root retains implementation ownership. |
| IVA-CAPG-03 / V03; Freeze Binding Completeness Evidence / acceptance-to-seam projection | assigned_execution_node | GOAL-CAPG-01 / WP-CAPG-03 | cross-pack skill/corroded preset, loser의 폐기된 dangling ref, winner가 남긴 dangling buff/reward/registration → 최종 목록만 검증, late compile/generated setup 실패는 미게시. E01/E02/E03 reproducible isolated fixture/public API, command+nonzero count+terminal exit+source identity and direct assertions; no personal file access. V08 audit output is separate read-only auditor work, root retains implementation ownership. |
| IVA-CAPG-04 / V04; Freeze Binding Completeness Evidence / acceptance-to-seam projection | assigned_execution_node | GOAL-CAPG-01 / WP-CAPG-03 | A 런/briefing 유지 중 B 게시, 새 B 런, invalid B와 B briefing 준비 실패·허용되지 않은 StartNewGame → A state/profile/seed/generation/출력 무변경, 성공 B는 briefing/starter/전투 pin. A/B export fingerprint가 각 pinned 세대와 일치함을 직접 대조. E03/E05 reproducible isolated fixture/public API, command+nonzero count+terminal exit+source identity and direct assertions; no personal file access. V08 audit output is separate read-only auditor work, root retains implementation ownership. |
| IVA-CAPG-05 / V05; Freeze Binding Completeness Evidence / acceptance-to-seam projection | assigned_execution_node | GOAL-CAPG-01 / WP-CAPG-03 | matching 최신 콘텐츠의 새 application restart/continue/explicit checkpoint load → 복원 전후 run identity·상태·난수 진행 또는 이어지는 결정론적 결과를 직접 대조. 같은 ID 다른 내용, contract mismatch·old V4/V5·unknown fields·checksum·load-limit는 기존 상태/파일 무변경 오류. E04/E05 reproducible isolated fixture/public API, command+nonzero count+terminal exit+source identity and direct assertions; no personal file access. V08 audit output is separate read-only auditor work, root retains implementation ownership. |
| IVA-CAPG-06 / V06; Freeze Binding Completeness Evidence / acceptance-to-seam projection | assigned_execution_node | GOAL-CAPG-01 / WP-CAPG-03 | 실제 외부 manifest/catalog의 구형 schema·미지원 authoring/execution capability·실행되지 않는 unsupported opcode, 누락/빈 selection·duplicate ID/path·case collision·절대/탈출/symlink·동시변경·file/byte/depth/dependency 한계 경계 → source 진단·token 미반환·embedded fallback 부재. 같은 입력 재시도/주석 변경·명시적 순서 변동의 지문, strong reference 회수 관찰. E01/E06 reproducible isolated fixture/public API, command+nonzero count+terminal exit+source identity and direct assertions; no personal file access. V08 audit output is separate read-only auditor work, root retains implementation ownership. |
| IVA-CAPG-07 / V07; Freeze Binding Completeness Evidence / acceptance-to-seam projection | assigned_execution_node | GOAL-CAPG-01 / WP-CAPG-03 | 기존 catalog/item/skill/fragment/preview/map/VN/new-run/profile/checkpoint 회귀와 release lib/실제 pack 소비 binary. 전체 core baseline을 최종 owning node가 수행. E01–E06 reproducible isolated fixture/public API, command+nonzero count+terminal exit+source identity and direct assertions; no personal file access. V08 audit output is separate read-only auditor work, root retains implementation ownership. |
| IVA-CAPG-08 / V08; Freeze Binding Completeness Evidence / acceptance-to-seam projection | assigned_execution_node | GOAL-CAPG-01 / WP-CAPG-03 | R01 IVA02/04/05/06에 대한 source/parser/publication/store 변경 영향 map, 필요한 nonzero 회귀와 current target의 fresh R01 GEX-20. 과거 V4 보존 delta와 현재 R02 전환을 구분. R01 accepted outputs 보존 reproducible isolated fixture/public API, command+nonzero count+terminal exit+source identity and direct assertions; no personal file access. V08 audit output is separate read-only auditor work, root retains implementation ownership. |

### Single Goal Member — GOAL-CAPG-01

- Goal ID: `GOAL-CAPG-01`
- Goal Kind: `single`
- Phase ID: `PHASE-CAPG-01`
- Assigned Final Execution Node Set: `AG-CAPG-01`, `WP-CAPG-03`
- Covered Policy Element Revision Ref Set: `PER-CAPG-E01-REVIEWED-001`, `PER-CAPG-E02-REVIEWED-001`, `PER-CAPG-E03-REVIEWED-001`, `PER-CAPG-E04-REVIEWED-001`, `PER-CAPG-E05-REVIEWED-001`, `PER-CAPG-E06-REVIEWED-001`
- Covered Decision Revision Ref Set: `DR-CAPG-E01-001`, `DR-CAPG-E02-001`, `DR-CAPG-E03-001`, `DR-CAPG-E04-001`, `DR-CAPG-E05-001`, `DR-CAPG-E06-001`
- Derived Observable Policy Acceptance Coverage: Freeze의 6개 PER가 소유한 14개 acceptance 전체. 아래 zero-based Acceptance Completion Matrix에서 원문 projection과 validation seam을 연결하며 의미를 변경하지 않는다.
- Objective / Contract-Visible Result: 동일 바이너리에서 선택한 외부 공식·사용자 팩을 검증·합성해 불변 세대로 게시한다. 기존 런은 A를 유지하고 허용된 새 런은 B를 briefing/starter부터 저장까지 고정한다. 최신 저장은 실제 콘텐츠·계약이 일치할 때만 복원하며 실패 후보/복원은 기존 런과 profile을 바꾸지 않는다.
- User-Visible Problem: 현재 게임 원문은 고정 embedded inventory에 결속돼 외부 팩 선택·명시 override·검증된 세대 전환이 없다. 저장의 checksum은 같은 ID의 다른 콘텐츠를 구별하지 못하고 state-only commit은 데이터 교체를 보존하지 못한다.
- User-Visible Completion Condition Set: C01 동일 release 소비 바이너리가 실제 외부 A/B 정의와 실행 차이를 관찰하고 공식 pack도 같은 loader 사용; C02 Add/Replace/Remove·namespace·명시 winner·final 참조/등록/획득 결과가 순서 우연에 무관; C03 invalid/late compile 후보는 source 오류와 token 미반환; C04 A pin 보존, 허용 B 새 런은 briefing/starter/전투/export에 같은 generation, preparation 실패와 금지된 시작은 A 무변경; C05 동일 최신 콘텐츠의 새 application에서 run identity/state/RNG 복원, mismatch/구형은 무변경 거부; C06 공식 source/custom input/개인 파일 비덮어쓰기와 profile/도감 보존; C07 지원·경로·입력 예산 검사, 동일 의미 지문·외부 변경 불변성·미참조 세대 회수; C08 실제 core/release/기존 소비자 회귀와 영향받은 R01 계약의 current 감사 복구. 아래 모든 관찰은 미래 완료 조건이며 현재 PASS가 아니다.
- In-Scope: Plan WP01/02의 AT02 전체: owned external snapshot/manifest/typed namespace·dependency·explicit operations, 최종 raw resolve/lowering/compile/generated setup, semantic fingerprint/source receipt/opaque generation, official packaging과 production embedded 경로 제거, application eligibility/briefing pin/commit/latest checkpoint 전환. WP03은 8 IVA와 R01 영향·최종 core/release 검증을 소유한다.
- Out-of-Scope: R03의 전체 validate/describe/explain/simulate CLI와 test-session reload, 후속 expression/flow/사본/유물/보스 등 R04–R24 신규 기능, RON editor UI, 서버/Unity 변경·외부 배포, 개인 저장 삭제/자동 변환/자동 source↔live_data 덮어쓰기, 새 gameplay/데이터 손실 예외 승인. R02 최소 public loader·실제 release 소비와 모든 저장 경계는 후속 CLI에 미루지 않는다.
- Reviewed Current Behavior / Implementation Delta: Review B01–06/L01–06 인수. 24 borrowed source와 include_str input→parse/validate/generated preview→opaque database가 현재 실제 경로다. Skill/corroded raw가 조기 참조 lowering하고 실제 배열 순서가 seed 선택에 영향을 준다. GameCore는 bare data Arc, prepared commit은 state/seed만 보존하며 briefing은 RunState보다 먼저 data를 쓴다. V4/V5 checksum 저장에는 content identity가 없다. 이를 winning raw→검증된 token→generation-aware app/latest save로 함께 전환한다.
- Goal-Local Source of Truth / Owning Layer: 정책은 exact Freeze PER/DR, 경계는 Plan WP/AG/SC01–05 및 Review L01–06이다. Host selection은 후보 입력을 소유하고 core snapshot/resolver/compiler만 검증 generation을 발행한다. Token이 database+contract/fingerprint를 한 authority로 소유하며 GameCore의 prepared commit과 강한 참조가 런 수명을 소유한다. Save capture/cache/reader는 같은 token identity를 소비한다. Source receipt/EffectSource 진단은 semantic identity나 개인 profile authority가 아니다.
- Goal-Local Invariant Set / Source Refs: I01 SC01/E01/E02/E06: dependency 순서/디렉터리 열거는 winner 아님; explicit typed full-definition operations와 최종 winning 참조 검사, UUID/local/reference-keyed/singleton 구별. I02 SC02/E06: 실제 authored 실행 배열과 fingerprint가 일치하고 byte receipt/derived caches/source metadata를 semantic hash와 구별. I03 SC02–03/E03/E05: 검증 성공한 opaque token만 공개; 기존 gate 후 briefing 전에 pin하고 state/seed/token/output 동시 commit; B 실패는 A 무변경 오류. I04 SC04/E04: 모든 capture/cache/export/import/continue가 최신 계약·content identity·checksum을 mutation 전에 검증. I05 TR30-02/03/05/E05: profile/도감과 개인 파일 보존, R01 기존 executor/효과 source·순서·취소 의미 유지. I06 SC05/Plan audit: 후속 Phase 없이 R02 전체 사용 가능하며 source 변경으로 영향받은 R01 accepted 계약을 current 검증/감사로 복구. E 번호는 아래 exact trace PER/DR를 뜻한다.
- Code/Data/Schema/Store/Transport/Consumer Boundary: AG는 live_ron/data의 owned source와 모든 typed raw catalog, final lowering/generated preview 및 published token을 상세화한다. 공통 resolver는 winning definition의 원래 pack/file/field locator를 유지한다. Global canonical key와 namespace::local exact string, recipe 참조키/UUID/definition-local 키를 구별하고 replacement 위치와 신규 항목 순서를 결정한다. CoreApplication/GameCore에서 StartNewGame의 비영구 typed candidate를 existing gate 뒤 draft에 설치하고 commit에 token/state/seed/output을 포함한다. state/active_run/node_flow와 공개 application checkpoint 포트를 최신 fingerprint format으로 함께 전환한다. 기존 effect execution은 같은 compiled catalog를 소비하며 source metadata 전달을 유지한다. 공식 source packager는 명시 별도 output만 만들고 기존 custom input을 자동 동기화하지 않는다. Typed API/enum/file 배치는 GEX에서 실제 caller와 함께 확정한다.
- Consumer Verification Boundary: 아래 workspace-confirmed core consumers 전부 required. 로컬 서버의 direct embedded caller도 확인했지만 변경/전체 workspace 호환 claim은 사용자 제외 범위다. Canonical-external Unity projection과 deployed actual은 별도 분류하며 확인하지 않은 외부 성공을 쓰지 않는다. 필수 core consumer가 inaccessible이거나 AT02에서 누락되면 완료하지 않는다.
- Goal-Local Dependencies / Execution Prerequisites: 기능상 선행은 `(GS-CAMEC-P2G50-001, GOAL-CAMEC-01)`의 R01 최소 authoring/source 계약이다. 현재 ATR-CAMEC-R01-001와 Complete/High 감사 291-member currentness 및 transitive fixtures를 P2G에서 확인했다. 이는 R01 전체 미래 기능을 요구하지 않는다. GEX-00에서 해당 source/basis를 다시 확인하고 baseline copy를 확보한다. AG WP01/02는 독립 완료하지 않으며 WP03이 전체 결과를 소비한다. 구현 전에 실제 producer→consumer→save 경로, exact raw adapters·입출력/ID 유일성·source/세대 수명·처리 순서·eligible commit·실패/재시도·budget·시험 입력과 oracle·수정 symbol을 기존 GEX PLAN/notes에 상세 설계한다. 내부 준비는 새 사용자 승인 gate가 아니다.
- Validation Plan / Completion Matrix: WP03이 아래 14 acceptance, IVA01–08 및 10 dimension을 실제 입력→생산자→소비자→상태/오류 관찰에 연결한다. 새 테스트 이름은 구현 뒤 실제 selector로 기록한다. command/fixture/source/data/test/config/toolchain/lock/exit/nonzero count/관찰을 고정하고 실패하면 AG 교정→영향 검증을 반복한다. Zero/ignored/미실행·컴파일만·snapshot 기대값 갱신만은 완료가 아니다. 끝에는 현재 target에 대한 separate fresh GEX-20 판정이 필요하다.
- Structural Completion / Legacy and Fallback Disposition: production embedded fallback/fixed borrowed source authority, 폐기된 V4/V5 reader/자동 변환/추정 fingerprint, live generation setter/독립 data+fingerprint authority, file-order last-wins, 무시된 Remove/unsupported 빈 성공을 남기지 않는다. Test fixture embedded 원문은 명시 fixture boundary만 허용하며 release 입력을 대신하지 않는다. 기존 supported executor와 공통 검증은 재사용하고 새 compiler/descriptor-only 성공이나 mock-only 외부 팩을 완료로 세지 않는다. 후속 VM/전역 무제한 registry를 미래 편의로 추가하지 않는다.
- Canonical Documentation Obligation: 구현 뒤 [Data Loading Contract](../../data_loading_contract.md)에 실제 외부 selection/manifest/namespace/operation/budget/source diagnostics/fingerprint/official packaging과 최신 입력 예시를 반영한다. [Core Runtime Contract](../../core_runtime_contract.md)에 token authority·eligible generation-aware commit·briefing pin/latest checkpoint/profile 보존·실제 API와 서버 후속 adaptation을 반영한다. 기존 [Skill Target Contract](../../skill_target_contract.md)의 source/cast semantics에 실제 변경이 있으면 해당 부분만 맞춘다. API/example은 실제 parser/public loader로 검증하며 미래 제안을 현재 지원으로 표시하지 않는다.
- Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan: 별도 shared baseline directive N/A. R01 final 1,166개와 fresh audit 4개 probe는 exact unchanged 입력·환경·claim만 재사용 가능하며 새 R02 지원을 증명하지 않는다. 아래 impact map의 source/data/config/fixture/harness/schema/store/lock 변경은 영향받은 evidence를 invalid 처리하고 실제 focused nonzero 검사로 rebaseline한다. 마지막 integrated target의 전체 core lib/tests와 release public pack 소비는 WP03 소유다. 날짜·새 Phase 자체로 전체 재실행을 늘리지 않으며 실패/관련 변경이 있을 때만 반복한다.
- Assigned Durable Integration Validation Seam Coverage: `IVA-CAPG-01`, `IVA-CAPG-02`, `IVA-CAPG-03`, `IVA-CAPG-04`, `IVA-CAPG-05`, `IVA-CAPG-06`, `IVA-CAPG-07`, `IVA-CAPG-08` 전부 WP-CAPG-03 / PHASE-CAPG-01. V08은 R01 IVA02/04/05/06 영향 map과 fresh 감사 복구를 포함하며 과거 V4 보존 delta와 현재 R02 latest-only 전환을 구별한다.
- Applicable Migration/Compatibility/Rollout/Rollback: Freeze TR30-01–06 인수. 외부 source/모든 catalog/공식 packaging→generation/new-run/save를 AT02로 전환한다. Latest-only reader/writer와 기존 checksum/seed/load-count/lifecycle 의미를 함께 검증하며 개인 파일에 접근·삭제·자동 변환하지 않는다. 실패 후보/briefing/restore는 기존 A/profile 무변경 오류다. File 재확인/immutable directory 교체 가이드는 원자적 filesystem snapshot 보장이 아니다. 서버/Unity 실제 migration/rollout은 제외하고 API adaptation 필요만 문서화한다.
- Goal-Local Known Unknown / Blocker / Stop Condition: KU30-01 exact types/manifest/adapters/budgets는 AG가 reviewed 의미 안에서, KU30-02 test selectors/성능관찰은 WP03이 실제 측정으로 해소한다. KU30-03 R01 currentness 복구는 R02 완료 전 필수다. KU30-04 외부 host/deployed population은 제외와 미검증 상태를 유지한다. Namespace/실행 순서/기존 gating·profile/저장 손실 의미를 유지할 수 없거나 필수 source/consumer가 미확인이면 earliest P2G owner로 반환한다. Material policy·새 권한/개인 데이터 예외는 사용자 결정이며 내부 타입·구현 실패는 자율 교정 대상이다.
- Applicable Risk / Reviewer Requirement and Evidence: 주요 위험은 raw loser 참조 조기 평가, 배열 순서와 hash 불일치, draft와 live generation authority 분리, 저장 reader/cache 누락, budget/path 우회와 R01 source metadata 회귀다. Plan SC01–05/TR30–01–06·8 IVA가 직접 대응한다. 사용자 AUTORUN의 fresh 감사·별도 읽기 전용 reviewer 운용 요청을 GEX에 인계한다. Root는 구현 owner이며 자기 점검을 독립 감사로 표시하지 않는다. R01 impact 감사와 R02 완료 감사는 실제 별도 fresh auditor가 current target을 읽고 판정해야 한다.

#### Consumer Boundary

| 분류 | 경계 / 근거 | 완료 판단 |
| --- | --- | --- |
| workspace-confirmed / required | data raw/resolve/compiler/generated preview/public token 및 실제 24 catalog 소비 | A/B·final 참조·acquisition/index/remove·source 오류와 no-publication 직접 관찰 |
| workspace-confirmed / required | CoreApplication/GameCore, gate→draft→VN/briefing/starter/map/battle | 기존 gating, pinned token와 state/seed/output/profile 원자성 |
| workspace-confirmed / required | checkpoint capture/cache/export/continue/explicit importer와 새 application | 최신 roundtrip identity/RNG, mismatch/old/checksum/load-count rejection 무변경 |
| workspace-confirmed / required | R01 effect source/skill/fragment/equipment/preview/proc·canonical fixtures | 영향 테스트와 fresh current predecessor audit, core/release 공개 소비 |
| workspace-confirmed / excluded | game_server/src/main.rs:123과 서버 시험의 GameDataBase::load_live_embedded 호출 | 호스트 API adaptation을 문서화. 서버 소스 변경/전체 workspace 호환 주장 제외 |
| canonical-external / excluded | canonical runtime/target 문서의 Unity projection consumers | core projection 회귀만 required, 외부 wire/배포 성공 미주장 |
| known-inaccessible | 현재 required core consumer 중 확인된 inaccessible 대상 없음 | 외부를 접근 시도 없이 inaccessible이라고 부르지 않음; 필수 접근 문제는 owner 반환 |
| unknown / excluded | 실제 deployed server/Unity·사용자 저장 population | 성공/호환/자동 migration claim 없음 |

#### Acceptance Completion Matrix

PER member는 Freeze projection의 zero-based acceptance다. 원문 의미와 직접 관찰은 owning IVA와 위 C01–08에서 연결되며 P2G 실행 결과가 아니다.

| Acceptance member locator | Completion condition / required observation | Integration source / durable owner |
| --- | --- | --- |
| `PER-CAPG-E01-REVIEWED-001[0]` | 동일 바이너리에서 외부 A/B 입력을 달리하면 선택된 유효 정의가 실제 런에 반영된다. 해당 IVA의 실제 fixture·결과·무변경 oracle를 전부 관찰. | IVA-CAPG-01, IVA-CAPG-04, IVA-CAPG-07 / WP-CAPG-03 |
| `PER-CAPG-E01-REVIEWED-001[1]` | 구형/미지원 규격이나 불완전한 외부 입력은 명시적으로 거부되며 embedded 내용으로 성공을 대신하지 않는다. 해당 IVA의 실제 fixture·결과·무변경 oracle를 전부 관찰. | IVA-CAPG-01, IVA-CAPG-03, IVA-CAPG-06 / WP-CAPG-03 |
| `PER-CAPG-E02-REVIEWED-001[0]` | 추가와 명시적 교체·삭제가 최종 정의와 기존 등록·참조 소비자에 적용된다. 해당 IVA의 실제 fixture·결과·무변경 oracle를 전부 관찰. | IVA-CAPG-02, IVA-CAPG-03, IVA-CAPG-07 / WP-CAPG-03 |
| `PER-CAPG-E02-REVIEWED-001[1]` | 미해결 다중 교체·중복/누락 의존·cycle은 오류다. 명시적 winner/교체 관계가 있으면 파일 열거 순서에 무관한 같은 결과다. 해당 IVA의 실제 fixture·결과·무변경 oracle를 전부 관찰. | IVA-CAPG-02, IVA-CAPG-06 / WP-CAPG-03 |
| `PER-CAPG-E02-REVIEWED-001[2]` | 최종 catalog의 dangling 참조는 거부하고, 명시적으로 제거된 loser에서만 남는 참조는 최종 참조 실패로 오인하지 않는다. 해당 IVA의 실제 fixture·결과·무변경 oracle를 전부 관찰. | IVA-CAPG-03 / WP-CAPG-03 |
| `PER-CAPG-E03-REVIEWED-001[0]` | A 런 유지 중 B를 게시해도 A의 정의·상태는 그대로이고 새 B 런의 브리핑·starter·전투·저장은 B에 결속된다. 해당 IVA의 실제 fixture·결과·무변경 oracle를 전부 관찰. | IVA-CAPG-04, IVA-CAPG-05 / WP-CAPG-03 |
| `PER-CAPG-E03-REVIEWED-001[1]` | B 검증 또는 새 런 준비가 실패하면 기존 런·세션·profile은 바뀌지 않고 B 요청은 오류다. 이전 A를 B 성공처럼 반환하지 않는다. 해당 IVA의 실제 fixture·결과·무변경 oracle를 전부 관찰. | IVA-CAPG-03, IVA-CAPG-04 / WP-CAPG-03 |
| `PER-CAPG-E04-REVIEWED-001[0]` | 같은 최신 계약과 실제 콘텐츠로 만든 저장은 새 application에서 복원되고 런 identity·상태·난수 진행 의미를 보존한다. 해당 IVA의 실제 fixture·결과·무변경 oracle를 전부 관찰. | IVA-CAPG-05, IVA-CAPG-07 / WP-CAPG-03 |
| `PER-CAPG-E04-REVIEWED-001[1]` | ID가 같아도 실제 콘텐츠 또는 계약이 다르면 복원을 명시적으로 거부하며 기존 런·profile·입력 파일은 그대로다. 해당 IVA의 실제 fixture·결과·무변경 oracle를 전부 관찰. | IVA-CAPG-05 / WP-CAPG-03 |
| `PER-CAPG-E04-REVIEWED-001[2]` | fingerprint 없는 이전 형식과 미지원 버전은 거부하고 자동 변환·추정 fingerprint·삭제로 해결하지 않는다. 해당 IVA의 실제 fixture·결과·무변경 oracle를 전부 관찰. | IVA-CAPG-05 / WP-CAPG-03 |
| `PER-CAPG-E05-REVIEWED-001[0]` | 공식 pack 패키징은 명시한 별도 출력에 생성하고 기존 사용자 live_data나 개인 저장을 자동 수정·삭제하지 않는다. 해당 IVA의 실제 fixture·결과·무변경 oracle를 전부 관찰. | IVA-CAPG-01, IVA-CAPG-07 / WP-CAPG-03 |
| `PER-CAPG-E05-REVIEWED-001[1]` | 검증 성공·실패 및 새 런 전환 뒤 사용자 profile과 도감의 적용 가능한 기존 진행이 보존된다. 해당 IVA의 실제 fixture·결과·무변경 oracle를 전부 관찰. | IVA-CAPG-04, IVA-CAPG-05, IVA-CAPG-07 / WP-CAPG-03 |
| `PER-CAPG-E06-REVIEWED-001[0]` | 지원되지 않은 코드/opcode/capability와 허용 root 밖 입력, 경로 충돌·파일 누락·읽기 중 변경·입력 한계 초과는 후보 게시 전에 source를 식별하는 오류로 거부된다. 해당 IVA의 실제 fixture·결과·무변경 oracle를 전부 관찰. | IVA-CAPG-06, IVA-CAPG-07 / WP-CAPG-03 |
| `PER-CAPG-E06-REVIEWED-001[1]` | 동일 입력·선택·계약은 같은 resolved fingerprint를 가지며 이미 게시된 불변 세대는 외부 bytes 변경의 영향을 받지 않는다. 미참조 세대는 영구 전역 수집 없이 회수 가능하다. 해당 IVA의 실제 fixture·결과·무변경 oracle를 전부 관찰. | IVA-CAPG-02, IVA-CAPG-04, IVA-CAPG-06 / WP-CAPG-03 |

#### Command and Changed Input-to-Seam Map

기존 package/target 기반 실행 후보이며 신규 selector는 구현 뒤 확정한다. P2G에서 게임 검사는 실행하지 않았다.

| 입력 변화 | 후보 command / seam | 완료·재사용 경계 |
| --- | --- | --- |
| raw catalog/loader/namespace/manifest/source/packager | `cargo test -p game_core --lib game::data`; 새 external directory integration target | IVA01–03/06, 실제 filesystem A/B·충돌·final refs·source/budget와 official packaging 보존. 새 test target은 등록·nonzero 확인 |
| app generation/gating/VN/starter/save/profile | `cargo test -p game_core --lib game::world`; 공개 application checkpoint/generation tests | IVA04/05, token/database/profile/state/seed/output authority snapshot을 포함한 정상/실패/재시도/복원 oracle |
| R01 parser/source/publication/store 및 직접 소비 | `cargo test -p game_core --test skill_runtime_contract`; `--test live_skill_catalog_audit`; `--test live_item_skill_activation`; 관련 authoring/live_ron/proc/withdraw unit seams | IVA07/08와 R01 IVA02/04/05/06. 새 source 때문에 invalid된 evidence를 rebaseline하고 fresh R01 GEX-20 currentness 복구 |
| shared schema/harness/feature/dependency 또는 최종 통합 상태 | `cargo check -p game_core --all-targets`; `cargo test -p game_core --lib --tests`; `cargo check -p game_core --release --lib`; 실제 release public pack 소비 binary A/B 실행 | WP03 최종 전체 core baseline. Cargo autobins=false이므로 실제 새 consumer 등록/feature 확인. fixture-only debug 경로가 release 증거를 대신하지 않음 |
| 문서/예시 또는 unchanged evidence | 관련 source/test/lock/toolchain/claim SHA와 parser/public API 예시 대조 | 변경 영향 없는 evidence만 재사용; 문서만 바뀌면 관련 링크/API 일치 확인, 무관 게임 전량 재실행 없음 |

#### Validation Dimension Matrix

| Dimension | Disposition (`required` / `N/A`) | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |
| --- | --- | --- | --- |
| success | required | IVA01–05/07: external A/B→public token→실제 app/battle/save→새 app 복원 | C01–06. 정의/실행 차이, remove/index/acquisition, briefing pin과 identity/RNG 왕복을 직접 관찰 |
| invalid_input | required | IVA02/03/05/06: old/unknown schema·capability/opcode, dependency/namespace/conflict/dangling, 경로/입력 한계, save mismatch | C02/C03/C05/C07. source 있는 오류, token 미반환과 기존 authority/input bytes 무변경 |
| failure_rollback | required | IVA03–05: 후반 compile/generated setup, B briefing/context preparation, checksum/content/restore validation 실패 | C03–06. token/state/seed/profile/output/cached checkpoint snapshot 무변경, 이전 A를 B 성공으로 반환하지 않음 |
| retry_idempotency | required | IVA02/04–06: invalid 뒤 valid 재시도, 동일 pack/계약 반복 compile, 반복 StartNewGame/restore의 기존 gate/load-count | 실패 잔여물이 다음 generation에 섞이지 않고 동일 semantic fingerprint. 재호출은 기존 eligibility/lifecycle를 보존하며 새 네트워크 exactly-once를 발명하지 않음 |
| ordering_concurrency | required | IVA02/04/06: shuffled inventory/selection, 명시 writer chain/winner, 실제 candidate 배열 순서, 파일 변경 중 읽기, A 실행 중 B 게시 | 실행과 hash 순서 일치; 우연 last-wins 없음. immutable A pin 보존, 감지한 file race 거부; atomic filesystem snapshot 성공을 과장하지 않음 |
| boundary_empty_max | required | IVA02/05/06: empty/missing/duplicate selection/ID/path, byte/file/depth/dependency budget 경계, UUID/local/reference key 구별, load-count 한계 | C02/C05/C07. host-owned budget 경계 허용/초과 거부와 overflow 방어, official 입력 실제 측정. 새 gameplay cap은 임의 결정하지 않음 |
| disconnect_restart_persistence | required | IVA04/05: A/B exports·matching 최신 content의 새 CoreApplication, continue/explicit load/cache, missing fingerprint/old format | C04–06. 동일 pin/contract/fingerprint와 run identity/state/RNG 복원. mismatch는 기존 profile/seed/token/file 무변경. 네트워크 재접속 자체는 scope 밖 |
| live_data | required | IVA01/03/06/07: 실제 official source 별도 packaging 후 외부 production loader, 수정 A/B 및 invalid external bytes | C01/C03/C06/C07. 동일 release 바이너리와 real catalog를 사용하고 embedded fallback/자동 custom 덮어쓰기 없음 |
| producer_consumer | required | IVA01–08: all typed raw roots→final winner/compile/generated preview→opaque token→VN/starter/battle→save readers | C01–08. mock-only·타입/문서 존재 대신 실제 모든 required core 소비자 결과. 원문 source 위치/R01 cast origin이 merge 위치와 혼동되지 않음 |
| legacy_fallback_absence | required | IVA01/05–08: production entry/dispatch audit, old RON/manifest/save·unsupported negative fixture, release external consumer | C01/C03/C05/C07. 폐기 embedded production/old readers/aliases/dual authority/빈 성공/fixture 전용 production branch 부재. 기존 supported executor 재사용과 구별 |

## Policy–Phase–Goal Traceability

Exact Freeze membership/decision과 Plan primary selector에서 파생하며 별도 정책 authority가 아니다.

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| CAPG-E01 | PER-CAPG-E01-REVIEWED-001 | DR-CAPG-E01-001 | PHASE-CAPG-01 | GOAL-CAPG-01 |
| CAPG-E02 | PER-CAPG-E02-REVIEWED-001 | DR-CAPG-E02-001 | PHASE-CAPG-01 | GOAL-CAPG-01 |
| CAPG-E03 | PER-CAPG-E03-REVIEWED-001 | DR-CAPG-E03-001 | PHASE-CAPG-01 | GOAL-CAPG-01 |
| CAPG-E04 | PER-CAPG-E04-REVIEWED-001 | DR-CAPG-E04-001 | PHASE-CAPG-01 | GOAL-CAPG-01 |
| CAPG-E05 | PER-CAPG-E05-REVIEWED-001 | DR-CAPG-E05-001 | PHASE-CAPG-01 | GOAL-CAPG-01 |
| CAPG-E06 | PER-CAPG-E06-REVIEWED-001 | DR-CAPG-E06-001 | PHASE-CAPG-01 | GOAL-CAPG-01 |

## Goal Authoring Summary

단일 Phase의 2개 final node, 6개 정책·14개 acceptance, 8 IVA와 10 required dimension을 하나의 Single Goal로 인계한다. 표준 구조 검증과 의미 검토를 마친 complete 단계 결과만 current 선택을 소유한다. Handoff·GEX 활성화·게임 PASS는 이 작성 문서가 부여하지 않는다.
