# CA01 최소 작성·실행 지원 계약 — Goal Set

2026-09-09 · P2G-50 · Goal Quality Contract 2. 이 파일은 Goal 작성 결과다. 구현·게임 테스트·완료 감사 결과는 후속 GEX 실행 원본이 소유한다.

Ref 표기의 `... Ref`는 그 이름의 typed alias이며 아래 Artifact ID와 producer가 발행한 `Content Digest: N/A`를 함께 뜻한다. `Goal Set Ref`는 identity envelope metadata이고 자신이 식별하는 canonical payload에 포함되지 않는다. Goal ID는 이 Set 내부 member key다. current 선택·lifecycle은 외부 [단계 결과](P2G_50_RESULT_001.json)에서 파생하며 검증 전 candidate는 후속 입력이 아니다.

## Goal Set

- Goal Set Ref: `GS-CAMEC-P2G50-001`
- Predecessor Goal Set Ref: N/A — 최초 Goal Set; 기존 Topic/Review/Freeze/Plan은 재사용한다.
- Exact Prerequisite Composite Basis: Working Revision Ref `WR-CAMEC-P2G20-001`; Final Policy Freeze Ref `FPF-CAMEC-P2G30-001`; Phase Plan Ref `PP-CAMEC-P2G40-002`; Goal Set Ref N/A; Authoritative Change Obligation Declaration Ref Set `[]`. 모든 typed Ref의 Content Digest는 producer 값 N/A다.
- Goal Structure Fact: `single_phase`
- Phase Plan Ref: `PP-CAMEC-P2G40-002`
- Final Policy Freeze Ref: `FPF-CAMEC-P2G30-001`
- Complete Goal Member Set: 아래 Single Goal Member `GOAL-CAMEC-01`의 전체 canonical payload 하나. 이 ID 표기는 파생 index이며 별도 Set artifact가 아니다.
- Applicable P2G-50 Obligation Key Set: `[]`
- Goal Quality Contract Version: `2`
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No
- User-Approved Validation Baseline Reuse Directive Ref: N/A — 이 Topic에 applicable한 승인 directive가 없다. 다른 Topic의 initial/final 횟수 계약을 인수하지 않는다.

입력은 [Freeze](P2G_30_FREEZE_001.json)와 [Plan](P2G_40_PHASE_PLAN_002.json), passed [Receipt](P2G_40_PHASE_PLAN_VALIDATION_RECEIPT_002.json)다. 정책 의미는 Freeze의 exact PER/DR, 구현 경계는 `CRA-CAMEC-P2G20-001`에 결속된 [Review §3–5](CODE_REVIEW_001.ko.md)와 Plan이 소유한다. 파일·source의 실제 hash와 작성 당시 entry currentness는 단계 결과/검증 기록에 둔다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| PHASE-CAMEC-01 | GOAL-CAMEC-01 | AG-CAMEC-01, WP-CAMEC-03 | 1 Phase → 1 Single Goal |

### Integration Acceptance-to-Goal Mapping

Plan의 `integration` 전체에서 파생한다. source/disposition/node는 보존하고 Goal owner는 위 Phase mapping에서 결정한다. locator와 evidence contract의 원본은 각 Plan member다. 아래 완료 검사로 구체화하되 정책 의미와 검증 node를 바꾸지 않는다.

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |
| IVA-CAMEC-01 / CAMEC-E01[0,1], E02[1]; Review V01/V04 | assigned_execution_node | GOAL-CAMEC-01 / WP-CAMEC-03 | Plan integration `IVA-CAMEC-01`의 locator/evidence/prerequisite 전체. `skill_runtime_contract`의 `run_battle`와 기존 `hit_gated_self_heal_and_buff_stack_repeat_attack_work_together`를 바탕으로 다른 step의 상수 Damage→Self Heal fixture 작성. metadata를 제외한 기존 실행 결과·순서와 재조합/unsupported 분기를 대조. 공식 HP 식/accepted-hit 정책을 새로 결정하지 않음. 테스트 이름·명령은 구현 후 실제 filter로 기록. |
| IVA-CAMEC-02 / CAMEC-E02[0], E03[2], E06[0]; Review V02/V04 | assigned_execution_node | GOAL-CAMEC-01 / WP-CAMEC-03 | Plan integration `IVA-CAMEC-02`의 locator/evidence/prerequisite 전체. `skill_data` contract fixture와 `live_ron::bundle_validation_returns_typed_skill_contract_errors` 확장. 중복 주소·음수 Damage·dangling 참조·unsupported op/property·carrier 오용에 source/object/action/effect/field 원인을 assert. 공개 descriptor의 실제 지원 조합마다 compiler/executor 결과 대조. 태그가 op를 허용하지 않음. 동적 스크립트·임의 property eval·OS/network/file 실행 입구 부재를 코드 경계와 거부 fixture로 확인. |
| IVA-CAMEC-03 / CAMEC-E03[0,1], E04[1]; Review V03 | assigned_execution_node | GOAL-CAMEC-01 / WP-CAMEC-03 | Plan integration `IVA-CAMEC-03`의 locator/evidence/prerequisite 전체. 같은 effect AND와 두 effect 존재를 서로 다른 fixture로 판별. 특정 원형/형태 ID 선택과 일반 tag predicate 분리. Damage scalar 검사·변경이 다른 Self Heal payload/대상을 바꾸지 않고 공유 carrier 요청은 영향 주소가 있는 진단인지 확인. 지원되지 않은 고급 carrier 변형이나 유물 활성 closure를 통과 fixture로 만들지 않음. |
| IVA-CAMEC-04 / CAMEC-E04[0,1]; Review V01/V05 | assigned_execution_node | GOAL-CAMEC-01 / WP-CAMEC-03 | Plan integration `IVA-CAMEC-04`의 locator/evidence/prerequisite 전체. 작성 원형/추가 효과의 source와 effect 태그 주소를 fixture에서 구분. runtime caster/cast/proc origin 연결을 대조하고 fragment/equipment SkillId·range preview·projectile/area shared command 경로의 metadata 연결 회귀를 기록. 최소 metadata 표현/연결 검증. 유물 영구 사본·실제 부여/연쇄 runtime 구현 완료 claim 없음. |
| IVA-CAMEC-05 / CAMEC-E05[0,1], E02[0]; Review V02/V04/V06 | assigned_execution_node | GOAL-CAMEC-01 / WP-CAMEC-03 | Plan integration `IVA-CAMEC-05`의 locator/evidence/prerequisite 전체. 유효 embedded 최신 bundle 로드, invalid bundle의 token 미반환, old/new 문법 병렬 성공 부재와 direct fixture 동시 전환을 검사. checkpoint capture/restore 기존 fixture로 정의 index가 새 영구 key가 되지 않았음을 확인. 개인 저장 접근/삭제/자동 변환 없음. 기존 V4/V5 reader의 제거 또는 미래 호환 인증을 요구하지 않음. |
| IVA-CAMEC-06 / Freeze TR30-03/04; Review V01/V05/V06 | assigned_execution_node | GOAL-CAMEC-01 / WP-CAMEC-03 | Plan integration `IVA-CAMEC-06`의 locator/evidence/prerequisite 전체. cast 완료·철수 뒤 늦은 step, HP-loss gate Pending/terminal, signed Heal, step ModifyDamage와 tactical resource signal, effect 순서·타격 수를 기존 fixture 관찰과 대조. invalid publication이 gameplay state를 부분 변경하지 않는 경계 확인. 기존 actual 보존 검사. 새 accepted-hit·비동기 handle·저장/배포 전환 의미는 별도 owner. |

### Single Goal Member — GOAL-CAMEC-01

- Goal ID: `GOAL-CAMEC-01`
- Goal Kind: `single`
- Phase ID: `PHASE-CAMEC-01`
- Assigned Final Execution Node Set: `AG-CAMEC-01`, `WP-CAMEC-03`
- Covered Policy Element Revision Ref Set: `PER-CAMEC-E01-REVIEWED-001`, `PER-CAMEC-E02-REVIEWED-001`, `PER-CAMEC-E03-REVIEWED-001`, `PER-CAMEC-E04-REVIEWED-001`, `PER-CAMEC-E05-REVIEWED-001`, `PER-CAMEC-E06-REVIEWED-001`
- Covered Decision Revision Ref Set: `DR-CAMEC-E01-001`, `DR-CAMEC-E02-001`, `DR-CAMEC-E03-001`, `DR-CAMEC-E04-001`, `DR-CAMEC-E05-001`, `DR-CAMEC-E06-001`
- Derived Observable Policy Acceptance Coverage: Freeze의 위 6개 PER가 소유한 12개 acceptance 전부. 아래 Acceptance Completion Matrix의 zero-based member locator를 따른다. 이 Goal에서 acceptance를 추가·삭제·재정의하지 않는다.
- Objective / Contract-Visible Result: 기존 literal Damage/Heal 및 Instant 실행을 재사용하면서 효과별 안정된 작성 주소·태그·typed property·출처·지원 정보와 진단을 같은 validated publication에 연결한다. 콘텐츠 ID별 전용 executor 없이 같은 지원 조합의 정의를 추가하고 core/headless에서 작성→검증→실행 결과를 확인할 수 있게 한다.
- User-Visible Problem: 현재 효과는 step 안의 벡터 값이며 효과별 작성 ID/태그/descriptor/compiled SourceMap이 없어 작성자가 어떤 효과와 공유 action을 선택했는지, 어떤 조합을 실제 실행할 수 있는지, 어느 원문 필드가 잘못됐는지 일관되게 확인하기 어렵다. 기존 오류의 일부도 lowering 때 object/path를 잃는다.
- User-Visible Completion Condition Set: C01 같은 지원 조합을 다른 정의 ID로 재조합해 같은 공통 경로로 실행; C02 실제 지원 정보와 주소 있는 거부 진단을 core/headless 소비자가 획득; C03 같은 효과 AND·다른 효과 존재·전용 원형/형태 선택 구분 및 독립 scalar 변경; C04 원형/추가 효과의 작성 출처와 runtime caster/cast/proc 발생 출처의 연결·태그 판별; C05 최신 parser/catalog/직접 소비자가 하나의 token으로 일치하고 invalid 입력은 공개되지 않음; C06 기존 순서·타격 수·HP-loss gate·signed Heal·ModifyDamage/signal·취소 결과 보존; C07 저장에 정의 index를 새 영구 key로 넣지 않고 개인 파일 비삭제/비변환; C08 지원 밖 op/property와 외부 코드/명령은 명시적으로 거부. 각 조건은 아래 acceptance/IVA/검증 dimension의 direct evidence로 독립 판정한다.
- In-Scope: Plan WP 01/02의 최소 작성/검증/실행 metadata와 최신 embedded schema/catalog·직접 fixture 공동 전환, WP 03의 IVA 01–06 durable 통합 검증. 공개하는 모든 지원 조합을 실제 validator/executor에 연결하고 기존 다른 op/spatial 동작은 보존한다. exact 타입/locator/schema 문법과 심볼 배치는 Review L01–05 의미 안의 구현 선택이다.
- Out-of-Scope: CA01 전체 표현식/selector/flow VM, UI 제작기·외부 다중 팩·hot reload·content generation, 파편/유물 영구 사본·활성/부여/연쇄 runtime, 보스 ActionHandle/graph, 공식 콘텐츠 수치·태그 목록·accepted-hit/HP 식 결정, 새 저장 포맷·개인 데이터 이동/삭제, 서버/Unity wire 변경·배포. 미래 전용 abstraction을 이 최소 delta의 선행으로 만들지 않는다.
- Reviewed Current Behavior / Implementation Delta: Review B01–07/§3/L01–05를 인수한다. `GameplayRonSourceSet::embedded`→raw resolve→contract/reference/generated-preview 검증→`ValidatedGameData`→`CoreApplication`은 existing actual이다. skill/step ID와 shared target/delivery/flow, ordered commands, cast/proc source 및 checkpoint가 있다. 새 effect 주소/typed descriptor/SourceMap/최소 query는 아직 없으며 AG가 이 publication과 runtime/직접 소비 경계에 연결해야 한다. 기존 Heal은 signed i32이고 HP-loss gate는 accepted-hit receipt가 아니다. source 31개 unchanged 확인은 작성 검토/단계 결과를 따른다.
- Goal-Local Source of Truth / Owning Layer: 정책은 exact Freeze PER/DR; 책임·동시 전환·검증 소유는 Plan `work_packages`, `atomic_groups`, `Shared Contract Handoff` SC01–03와 Review L01–04다. 작성 정의·metadata·지원/오류는 `skill_data`/`live_ron`의 compiler와 validated publication이 소유한다. runtime은 기존 `ability`/`battle`의 cast/step/command owner, fragment/equipment는 SkillId 소비자, range preview는 core 투영 소비자다. SourceMap은 작성 원문 주소를 소유하고 caster/cast/proc 발생 문맥·영구 소유 사본을 대체하지 않는다.
- Goal-Local Invariant Set / Source Refs: I01 SC01 — effect/action 정의 주소, compiled index, runtime cast/proc identity를 혼합하지 않음; I02 SC02 및 PER E03/E06 — 태그는 predicate metadata이며 op 권한/수치/임의 실행이 아님, 같은 effect AND와 서로 다른 effect 존재를 구분; I03 SC02 — amount 같은 effect scalar와 target/delivery/반경의 action owner 구분, 공유 carrier 변경은 영향 주소로 거부/진단; I04 SC03 및 Freeze TR30-01/04 — 검사 전체 성공 뒤 단일 token 공개, invalid candidate는 gameplay state를 부분 변경하지 않음; I05 Freeze TR30-03 — 기존 effect 순서·ModifyDamage·signed Heal·gate·cast 종료/철수/늦은 step 의미 보존; I06 TR30-02 및 PER E05 — 정의 metadata는 새 저장 사본 key가 아니며 기존 V4/V5 reader를 이번 변경으로 제거하지 않음; I07 Plan audit PHASE-CAMEC-01 — 후속 Phase 없이 최신 catalog와 모든 직접 소비자가 일치한 유효 상태. E 번호는 아래 trace table의 exact reviewed PER/DR를 뜻한다.
- Code/Data/Schema/Store/Transport/Consumer Boundary: AG가 raw `RawSkillDef/RawSkillStepDef`→`SkillDef/SkillStepDef`의 주소·typed metadata/lowering과 `validate_skill_contracts`·`LiveRonError` 위치 보존을 상세화한다. 기존 `GameDataBase::load_live_sources`/`ValidatedGameData` 공개 순서에 합치고 `resolve_skill_step`→`build_skill_step_commands`→damage/heal 및 기존 proc source로 전달한다. 실제 변경 전 fragment `apply_to_profile`, equipment activation/TriggerAbility, range preview `get_by_id`, projectile/area의 공통 helper와 직접 타입 생성 fixture까지 호출자를 확인한다. manifest/skills RON과 parser/runtime를 AG에서 함께 전환한다. 정확한 타입·필드·source locator/진단 인코딩, 지원 조합, 수정 symbol은 현재 코드에서 상세 설계한다. `EffectSite`/`ActionSite`는 Review의 proposal 이름이다. 저장 capture/restore와 wire는 비변경 경계를 확인하며 변경이 필요하면 해당 owner에 반환한다.
- Consumer Verification Boundary: 아래 Consumer Boundary 표의 workspace-confirmed core/headless 경로가 required다. canonical-external·known-inaccessible·unknown 분류를 보존한다. 존재만 아는 외부 소비자를 검증했다고 하지 않으며, 실제로 필수 외부 계약 변경이 드러나면 완료를 보류하고 P2G-20/정책 owner로 반환한다.
- Goal-Local Dependencies / Execution Prerequisites: 선행 Goal dependency는 N/A — 같은 Plan의 단일 Phase이고 다른 미구현 CA 전체를 선행으로 요구하지 않는다. 실제 existing publication·SkillId·executor와 CRAR 감사/currentness 인수 근거를 GEX-00에서 확인한다. AG의 WP 01/02는 독립 완료하지 않으며 그 단일 publication 결과를 WP 03이 소비한다. 구현 전 현재 producer→state/result→consumer→checkpoint/fixture를 조사하고 정확한 타입·입출력·주소 유일성 범위·상태 수명·처리 순서·오류/취소·cutover·시험 입력/기대값·변경 symbol을 GEX의 기존 PLAN/notes에 작성한다. 준비 점검은 지원 descriptor와 validator/executor 일치, source 유지, raw/compiled/catalog 정합성, cast 결과와 실패/늦은 step, 저장 비변경, 검증 가능성을 다룬다. 이 준비 점검을 별도 사용자 승인 Gate로 만들지 않는다. 새 정책·책임·Phase 변경은 아래 중단 조건을 따른다.
- Validation Plan / Completion Matrix: 아래 12개 acceptance와 IVA 01–06, 10개 dimension 및 Command/Impact Map을 완료 근거로 연결한다. WP 03이 실제 production loader/공통 executor를 쓰는 fixture를 구현·실행하고 공개 지원 조합 전부의 정상·거부·경계를 관찰한다. 실패 시 같은 Goal 내 AG 교정→영향 시험 재실행을 완료까지 반복한다. 실행 명령·selector·fixture/input·source/data/config/test·toolchain/lock·exit/test 수·관찰 결과를 실제 GEX 기록에 고정한다. zero/skipped/미실행은 pass가 아니며 snapshot 기대값만 바꿔 실패를 숨기지 않는다. 구현/검증 후 fresh GEX-20 판정까지는 후속 실행 요청·현재 GEX 계약을 따른다.
- Structural Completion / Legacy and Fallback Disposition: SC01–03과 C05/C08의 단일 authority를 코드 경로/strict loader/통합 증거로 확인한다. 새 문법 도입 시 폐기된 authoring parser/alias/dual executor를 남기지 않는다. 구형 효과를 조용히 성공시키는 fallback, descriptor-only 빈 실행, test/fixture 전용 production branch, 현재 live path가 소비하지 않는 미래 VM/registry는 완료 불가다. 기존 typed executor와 다른 op·projectile/area는 재사용하며 제거 대상과 혼동하지 않는다. 기존 V4/V5 저장 reader도 이번 legacy 제거 대상이 아니다.
- Canonical Documentation Obligation: 구현 뒤 [Data Loading Contract](../../data_loading_contract.md)의 live authoring/strict loader/schema publication/fixture 책임에 실제 effect 주소·supported shape·error/source/cutover 예제를 반영한다. [Core Runtime Contract](../../core_runtime_contract.md)의 cast/command/source 소비 책임과 [Skill Target Contract](../../skill_target_contract.md)의 step shared target/delivery·실행 lifecycle 의미는 actual 변경 부분만 맞춘다. source 표현만 바뀌어 기존 의미가 그대로면 보존 근거를 해당 owner 문서/실행 기록에 남긴다. 문서가 미래 API를 현재 지원으로 표시하지 않게 한다. 이번 P2G에서 canonical 문서를 고치지 않는다.
- Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan: inherited shared directive N/A. 아래 impact map과 GEX §6.4를 적용한다. 생산 source/data/config가 바뀌면 test 파일이 같아도 해당 evidence를 invalid 처리하고 focused 검증을 실제 non-zero로 실행한다. fixture/harness/lock/schema 변경은 관련 seam 전체에 전파한다. unchanged evidence는 exact input/method/environment/claim이 유지되는 경우에만 재사용하며 문서 날짜·Phase 시작 자체는 전체 재실행 사유가 아니다. 새 계약의 시험은 과거 CRAR pass로 대체하지 않는다.
- Assigned Durable Integration Validation Seam Coverage: `IVA-CAMEC-01`, `IVA-CAMEC-02`, `IVA-CAMEC-03`, `IVA-CAMEC-04`, `IVA-CAMEC-05`, `IVA-CAMEC-06` 전부 WP-CAMEC-03. 위 공통 integration map과 아래 completion matrix가 같은 Single Goal에 구현·완료 근거를 연결한다. fixture를 만드는 일과 게임 시험 성공은 별도로 기록한다.
- Applicable Migration/Compatibility/Rollout/Rollback: Freeze TR30-01–05를 인계한다. schema/parser/latest embedded RON/direct consumer/기존 fixture는 AG에서 함께 cutover하고 old/new 문법의 병렬 성공을 추가하지 않는다. publication 실패는 token 미반환과 state 비변경으로 검증한다. 새 저장 포맷/영구 metadata key·개인 파일 삭제/자동 변환은 없으며 capture/restore의 기존 의미를 확인한다. live reload·release rollback·서버/Unity rollout은 이 최소 Goal에서 N/A이고 성공 claim도 없다.
- Goal-Local Known Unknown / Blocker / Stop Condition: KU30-01의 타입/문법/locator/신규 시험 selector는 위 AG/WP 03의 구현 전 상세 설계 및 완료 전 증거로 해소한다. KU30-02/03의 미정 콘텐츠 gameplay·사본/generation·새 상태는 비범위 owner에 남긴다. KU30-04의 외부 actual/runner와 KU30-05의 과거 대화 독립 획득도 기존 제한을 유지한다. required source/consumer를 확인할 수 없거나 reviewed premise/ownership/Phase·상태 수명·타격/취소·지원 의미가 달라지면 근거를 기록하고 가장 이른 P2G owner로 반환한다. 새 material policy·권한·개인 데이터 처리는 사용자의 결정 지점이다. 단순 내부 타입/구현 실패는 승인 범위의 기술 교정·관련 재검증 대상이다.
- Applicable Risk / Reviewer Requirement and Evidence: Plan AG-CAMEC-01/SC01–03/TR30-03/04가 보여 주는 주요 위험은 schema/catalog/runtime 불일치, source 단절, 공유 carrier/순서/취소 의미의 의도치 않은 변경과 false support다. 구현 전 준비 점검, IVA 전체·실제 직접 소비자 검증, 구현 종료 뒤 GEX §6.7의 fresh read-only completion review가 필요하다. 현재 Freeze/Plan/사용자 요청에는 별도 reviewer identity를 강제하는 조건이 없고 P2G는 reviewer를 지정하지 않는다. 후속 GEX가 실제 위험과 applicable requirement를 인수하며 명시적 독립 reviewer 조건이 생기면 지켜야 한다. 자기 점검을 독립 감사로 보고하지 않는다.

#### Consumer Boundary

| 분류 | 경계와 근거 | 완료 판단 |
| --- | --- | --- |
| workspace-confirmed / required | Review B01–05: live loader→validated token→CoreApplication/BattleCore; real raw/compiled shapes·source diagnostics | 실제 loader와 executor를 잇는 정상·invalid fixture 및 token/state 관찰 필요 |
| workspace-confirmed / required | Review §3: fragment SkillId→profile/stat pipeline/battle; equipment→TriggerAbility→proc invoke | 기존 선택/activation 결과와 source 전달의 실제 core 회귀 필요 |
| workspace-confirmed / required | Review §3: range preview, projectile/area shared command helper 및 직접 타입 생성 fixture | 최신 schema·타입과 함께 컴파일/관찰; mock끼리만 맞는 것으로 대체 불가 |
| workspace-confirmed / preservation | Review §3: checkpoint/BattleEntrySnapshot capture/restore, V4/V5 reader | 격리 상태 fixture의 보존·왕복과 새 영구 definition key 부재 확인 |
| canonical-external / excluded | core_runtime_contract의 Unity-Facing Projection Boundaries, skill_target_contract의 Projection And Authoring Boundaries가 외부 wire consumer를 명시 | core의 기존 preview 결과 보존까지만 required. 외부 Unity/서버 구현·wire 왕복 성공은 이 Goal에서 claim하지 않음 |
| known-inaccessible | 현재 필수 workspace 소비자 중 접근 불가로 확인된 대상 없음 | 외부 actual을 접근 시도 없이 접근 불가라고 단정하지 않음. 필수 소비자 접근 문제 발견 시 중단 |
| unknown / excluded | Released/Deployed actual, 외부 host 적용·runner routing, Review RQ07/KU30-04 | 배포·실행 적용 상태 미검증. 외부 포트 완성을 로컬 Goal 완료로 간주하지 않음 |

#### Acceptance Completion Matrix

모든 `PER[...]`는 위 Covered PER의 `Observable Policy Acceptance` zero-based member다. policy 문구 원본은 Freeze projection과 해당 PER이며 아래 관찰은 그 최소 검토 범위의 완료 증거다. 행에 적힌 성공은 미래 완료 조건이고 현재 실행 결과가 아니다.

| Acceptance member locator | Completion condition / required observation | Integration source / durable owner |
| --- | --- | --- |
| `PER-CAMEC-E01-REVIEWED-001[0]` | C01: 두 정의 ID에 동일 지원 조합을 작성하고 실제 공통 executor로 실행; 콘텐츠 ID 분기 부재를 코드와 결과로 확인. | IVA-CAMEC-01 / WP-CAMEC-03 |
| `PER-CAMEC-E01-REVIEWED-001[1]` | C01/C08: 공개 descriptor 밖 동작은 unsupported 원인/주소로 거부하고 공통 기능 delta로 구분; 빈 성공 부재. | IVA-CAMEC-01, IVA-CAMEC-02 / WP-CAMEC-03 |
| `PER-CAMEC-E02-REVIEWED-001[0]` | C02: core 소비자가 typed 지원 정보와 source/object/action/effect/field 진단을 얻고 잘못된 입력에는 token 미반환. | IVA-CAMEC-02, IVA-CAMEC-05 / WP-CAMEC-03 |
| `PER-CAMEC-E02-REVIEWED-001[1]` | C01/C02: UI 없는 core/headless 실제 loader/BattleCore fixture에서 작성→검증→상수 실행·출처 관찰까지 재현. | IVA-CAMEC-01 / WP-CAMEC-03 |
| `PER-CAMEC-E03-REVIEWED-001[0]` | C03: 같은 effect 태그/property AND와 두 effect 존재를 서로 다른 fixture로 판별; effect scalar를 바꿔도 다른 Self Heal 대상/payload 보존. | IVA-CAMEC-03 / WP-CAMEC-03 |
| `PER-CAMEC-E03-REVIEWED-001[1]` | C03: 일반 태그가 같아도 지정 원형/형태 ID 조건을 충족하지 않는 negative fixture와 별도 ID 선택의 positive 대조. | IVA-CAMEC-03 / WP-CAMEC-03 |
| `PER-CAMEC-E03-REVIEWED-001[2]` | C03/C08: 태그 추가만으로 op/property 지원 또는 실행 수치가 변하지 않음; unsupported와 shared-carrier 오용을 주소로 진단. | IVA-CAMEC-02 / WP-CAMEC-03 |
| `PER-CAMEC-E04-REVIEWED-001[0]` | C04: 원형/추가 효과 작성 출처를 fixture에서 구별하고 caster/cast/proc origin까지 연결; 영구 유물 사본/부여 runtime은 claim하지 않음. | IVA-CAMEC-04 / WP-CAMEC-03 |
| `PER-CAMEC-E04-REVIEWED-001[1]` | C03/C04: 추가 효과도 같은 effect 태그 predicate에서 선택되며 전용 대상 ID와 구분. 최소 metadata/query 검증이며 유물 활성 closure 전체는 제외. | IVA-CAMEC-03, IVA-CAMEC-04 / WP-CAMEC-03 |
| `PER-CAMEC-E05-REVIEWED-001[0]` | C05: 최신 parser/RON/모든 직접 소비자 동시 전환, 구형 authoring 별칭·이중 실행·자동 저장 변환을 추가하지 않았음을 strict loader/코드/fixture로 확인. | IVA-CAMEC-05 / WP-CAMEC-03 |
| `PER-CAMEC-E05-REVIEWED-001[1]` | C07: 개인 파일을 fixture로 사용하지 않고 기존 저장 writer/복원 경계를 검사하여 이 delta가 삭제/자동 변환 경로를 추가하지 않음을 증명. | IVA-CAMEC-05 / WP-CAMEC-03 |
| `PER-CAMEC-E06-REVIEWED-001[0]` | C08: typed op/property 해석과 실제 executor 지원 목록에서만 실행; 임의 스크립트/property eval 및 OS/파일/네트워크 명령 입구 부재를 코드와 inert 거부 fixture로 확인. | IVA-CAMEC-02 / WP-CAMEC-03 |

#### Command and Changed Input-to-Seam Map

아래는 기존 package/test target에서 확인한 **실행 후보**다. GEX 인수/상세 설계에서 fixture 격리·side effect·실제 filter·toolchain·feature를 확인한 뒤 실행 기록으로 고정한다. 이번 P2G에서 실행한 게임 command는 0개다. 새 test 이름을 여기서 존재하는 이름처럼 지정하지 않는다.

| 입력/책임 변화 | 후보 command 또는 실제 seam | 완료·재사용 경계 |
| --- | --- | --- |
| skill raw/compiled/typed validation/SourceMap 또는 loader/RON/manifest | `cargo test -p game_core --lib game::data::skill_data`; `cargo test -p game_core --lib game::data::live_ron`; 새 주소/unsupported fixture 포함 | IVA 02/03/05. 실제 non-zero selector로 duplicate/negative/dangling/unsupported/source 위치·유효 embedded/invalid token을 증명. 공개 조합별 누락은 추가 fixture와 실행으로 닫음 |
| cast/step/effect source/commands, damage/heal/gate 또는 shared spatial helper | `cargo test -p game_core --test skill_runtime_contract`; Review V01의 기존 helper와 신규 wrapper recipe | IVA 01/04/06. 기존 결과와 metadata 출처 모두 대조하고 signed Heal·gate Pending/terminal·철수/늦은 step·타격 수/순서 경계를 유지 |
| latest catalog, fragment/equipment SkillId 또는 range preview 소비 | `cargo test -p game_core --test live_skill_catalog_audit`; `cargo test -p game_core --test live_item_skill_activation`; 실제 fragment/range preview unit seam | IVA 04/05. real catalog/activation과 새 compiled shape 왕복을 확인. 기존 unit filter는 구현 시 실제 함수/모듈 이름으로 기록 |
| checkpoint 의존/비변경 또는 definition index lifetime | `cargo test -p game_core --lib game::world`; 기존 capture/restore fixture 중 해당 seam과 static writer field 검사 | IVA 05/06. 개인 저장 비접근, 새 영구 key/auto-migration/no-delete 경계 확인. 상태 변화가 필요하면 비변경 claim을 유지하지 않고 owner에 반환 |
| common schema/type, fixture harness 또는 dependency/feature/lock 변화; 마지막 통합 상태 | `cargo check -p game_core --all-targets`; `cargo test -p game_core --lib --tests`의 현재 target·격리·비용 확인 후 위험에 비례한 통합 검증 | 컴파일 성공만으로 새 acceptance 완료를 선언하지 않음. shared 변경 영향이 분리되지 않으면 관련 전체 core target을 검증. 적용 directive가 없으므로 initial/final 횟수 규칙은 새로 강제하지 않음 |
| canonical 문서만 수정 또는 unchanged seam 재사용 | current source와 owner 문서·fixture·명령 결과의 exact identity/claim 대조 | 관련 source/data/test/config/toolchain/lock·결과가 unchanged일 때만 과거 evidence 재사용; 현재 새 시험 실행으로 세지 않음 |

#### Validation Dimension Matrix

| Dimension | Disposition (`required` / `N/A`) | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |
| --- | --- | --- | --- |
| success | required | IVA 01/03/04: 실제 작성 정의→validated metadata/executor→BattleCore 결과; 다른 정의 ID 재조합과 Damage→별도 Self Heal | C01–04. 기존 실행 결과·순서와 effect/cast source를 함께 assert; headless 입구에서 UI 없이 재현 |
| invalid_input | required | IVA 02: 중복 effect/action 주소, dangling ref, 음수 Damage, unsupported op/property와 carrier 오용 fixture; 실제 loader/compiler 진단 | C02/C08. source/object/action/effect/field 원인을 얻고 token이 반환되지 않음. raw string 오류로 위치를 잃거나 태그만으로 미지원 op가 통과하면 실패 |
| failure_rollback | required | IVA 05/06 및 TR30-04: validation 후반 실패까지 invalid publication, 격리 gameplay state/기존 valid token 관찰 | C05/C06. candidate token 미반환·기존 state 비변경·부분 live 교체 없음. 새 hot reload/배포 rollback 프로토콜을 발명하지 않음 |
| retry_idempotency | required | IVA 02/05/06의 V04/V06 보완: 동일 raw 입력을 새 compile 시도에서 반복하고 invalid 뒤 valid 재시도; 기존 cast/terminal fixture 재실행 | C02/C05/C06. 실패 잔여 metadata가 다음 성공에 섞이지 않고 같은 작성 주소의 의미가 유지됨; 기존 cast 결과 소비/늦은 step 동작 보존. 포인터 equality나 새 네트워크 exactly-once는 요구하지 않음 |
| ordering_concurrency | required | IVA 01/04/06: 효과 순서, step ModifyDamage 선행 합성, HP-loss gate Pending/terminal, cast 종료/철수 뒤 늦은 step, 기존 projectile/area 관찰 | C04/C06. 현행 결과·타격 수/취소 수명 유지; metadata wrapper가 공유 action 순서를 바꾸거나 별도 비동기 handle 의미를 만들면 실패 |
| boundary_empty_max | required | IVA 02/03/06의 typed/schema 경계 fixture: missing ID/field, empty selector의 설계상 disposition, Damage 0/음수/현재 타입 경계, signed Heal·주소 재정렬·중복 | C02/C03/C06/C08. 기존 허용/거부·HP clamp/산술 경계를 보존하고 잘못된 값은 주소 있는 오류. 새 gameplay limit은 임의 결정하지 않고 필요 시 정책 owner로 반환 |
| disconnect_restart_persistence | required | IVA 05/06: 기존 checkpoint/BattleEntrySnapshot 격리 capture/restore·writer field 검사와 definition metadata lifetime 확인 | C07. 저장 왕복으로 기존 SkillId/gameplay 의미 보존; 새 compiled index 영구 key·개인 파일 삭제/자동 변환 없음. 네트워크 재접속/배포·새 content generation은 현재 상태 owner에 없으므로 그 성공은 claim하지 않음 |
| live_data | required | IVA 02/05: production embedded manifest/latest RON을 real strict loader로 검증하고 invalid source set을 격리 주입 | C05/C08. 최신 catalog 전체 직접 참조가 유효하며 구형/new authoring alias 이중 성공과 invalid-data token 발행 없음 |
| producer_consumer | required | IVA 01–05: 공개 지원 정보→raw compiler→validated token→실제 executor; fragment/equipment/preview/projectile/area 직접 소비자의 실제 shape | C01–05. 공개되는 각 지원 조합에 direct evidence, 정의/action/effect/source 주소의 왕복·일치; mock-only·descriptor 목록만으로 완료 불가 |
| legacy_fallback_absence | required | IVA 02/05/06: strict parser/dispatch/publication authority 코드 감사와 legacy/unsupported 거부 fixture, 최신 catalog 실행 | C05/C08. 폐기 authoring path·alias·빈 성공·dual executor·콘텐츠 ID 전용 분기·fixture 전용 production branch·미래 전용 abstraction 부재. 기존 다른 op/spatial/V4/V5 reader를 임의 제거하지 않음 |

## Policy–Phase–Goal Traceability

Freeze의 exact member/decision과 Plan primary selector 및 위 Phase mapping에서 파생한 표준 검증기 입력 표현이다. 독립 Matrix나 정책 authority가 아니다.

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| CAMEC-E01 | PER-CAMEC-E01-REVIEWED-001 | DR-CAMEC-E01-001 | PHASE-CAMEC-01 | GOAL-CAMEC-01 |
| CAMEC-E02 | PER-CAMEC-E02-REVIEWED-001 | DR-CAMEC-E02-001 | PHASE-CAMEC-01 | GOAL-CAMEC-01 |
| CAMEC-E03 | PER-CAMEC-E03-REVIEWED-001 | DR-CAMEC-E03-001 | PHASE-CAMEC-01 | GOAL-CAMEC-01 |
| CAMEC-E04 | PER-CAMEC-E04-REVIEWED-001 | DR-CAMEC-E04-001 | PHASE-CAMEC-01 | GOAL-CAMEC-01 |
| CAMEC-E05 | PER-CAMEC-E05-REVIEWED-001 | DR-CAMEC-E05-001 | PHASE-CAMEC-01 | GOAL-CAMEC-01 |
| CAMEC-E06 | PER-CAMEC-E06-REVIEWED-001 | DR-CAMEC-E06-001 | PHASE-CAMEC-01 | GOAL-CAMEC-01 |

## Goal Authoring Summary

단일 Phase의 2개 final node, 6개 정책 요소·12개 acceptance와 6개 integration row를 하나의 Single Goal member로 인계한다. 작성 내용의 의미 검토는 [작성 검토](P2G_50_AUTHORING_REVIEW_001.ko.md), 실제 표준 구조 검증과 current 선택은 [단계 결과](P2G_50_RESULT_001.json)가 소유한다. 이 요약은 Handoff·activation·구현 허가나 게임 시험 PASS를 뜻하지 않는다.
