from pathlib import Path
import json
p=Path('core_enhanced/docs/goals/content_authoring_value_expressions_reassessment');assert not (p/'P2G_50_GOAL_SET_001.ko.md').exists()
f=json.loads((p/'P2G_30_FREEZE_001.json').read_text())['artifacts'][0]['payload'];plan=json.loads((p/'P2G_40_PHASE_PLAN_001.json').read_text())['artifacts'][0]['payload']
s='''# R06 재검토·최종 취합 — Goal Set

Ref 표시는 Artifact Type에 해당하는 producer-issued Artifact ID, Content Digest N/A다. Goal ID는 이 Goal Set 내부 member key다. 이 Goal Set은 계획 결과이며 GEX 인수·activation·구현·감사를 승인하지 않는다.

## Goal Set

- Goal Set Ref: `GS-CAVER-P2G50-001`
- Predecessor Goal Set Ref: N/A — initial reassessment Goal Set; old R06 has no formal Goal Set.
- Exact Prerequisite Composite Basis: Working Revision Ref `WR-CAVER-P2G20-003`; Final Policy Freeze Ref `FPF-CAVER-P2G30-001`; Phase Plan Ref `PP-CAVER-P2G40-001`; Goal Set Ref N/A; Authoritative Change Obligation Declaration Ref Set `[]`.
- Goal Structure Fact: `single_phase`
- Phase Plan Ref: `PP-CAVER-P2G40-001`
- Final Policy Freeze Ref: `FPF-CAVER-P2G30-001`
- Phase Plan Validation Receipt Ref: `PVR-CAVER-P2G40-001`
- Complete Goal Member Set: `GOAL-CAVER-01` — 아래 Single member 전체 canonical payload.
- Applicable P2G-50 Obligation Key Set: explicit empty `[]`.
- Goal Quality Contract Version: `2`
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No
- User-Approved Validation Baseline Reuse Directive Ref: N/A — 사용자 지정 initial/final full 횟수 없음. AUTORUN §3/8의 currentness·변경 영향·검증된 재사용만 적용한다.

[Freeze](P2G_30_FREEZE_001.json), [Phase Plan](P2G_40_PHASE_PLAN_001.json), [generated receipt](P2G_40_PHASE_PLAN_VALIDATION_RECEIPT_001.json), [current 코드 검토·구현 제안](CODE_REVIEW_003.ko.md)을 exact basis로 소비한다. 이전 CODE_REVIEW_001/002와 P2G-20 후보들은 교정 이력이며 current는003이다. `COMPARISON_ORIGINAL_*`은 과거 승인 근거 해석용 원문이고 current member/Decision basis가 아니다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| PHASE-CAVER-01 | GOAL-CAVER-01 | AG-CAVER-01, WP-CAVER-04 | 1 Phase → 1 Single Goal |

### Integration Acceptance-to-Goal Mapping

Plan의 21개 integration row 전체를 그대로 이 Goal의 WP04에 배정한다. 검증기 통과는 아래 게임 시험 실행을 의미하지 않는다.

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |
'''
for row in plan['integration']:
 s+=f"| {row['id']} / {row['source']} | {row['disposition']} | GOAL-CAVER-01 / WP-CAVER-04 | {row['locator']} {row['evidence']} |\n"
s+='''
### Single Goal Member — `GOAL-CAVER-01`

- Goal ID: `GOAL-CAVER-01`
- Goal Kind: `single`
- Phase ID: `PHASE-CAVER-01`
- Assigned Final Execution Node Set: `AG-CAVER-01`, `WP-CAVER-04`
'''
s+='- Covered Policy Element Revision Ref Set: '+', '.join('`'+r['Artifact ID']+'`' for r in f['Ordered code_reviewed Policy Element Revision Ref Set'])+'\n'
s+='- Covered Decision Revision Ref Set: '+', '.join('`'+r['Artifact ID']+'`' for r in f['Decision Revision Ref Set'])+'\n'
s+='''- Derived Observable Policy Acceptance Coverage: Freeze의 위 9 exact member가 소유한 27 acceptance 전부. 아래 Completion Matrix는 member의 원문에서 파생한 1-based projection이며 정책을 새로 정하지 않는다.
- Objective / Contract-Visible Result: R06의 typed finite ValueExpr가 공개 RON→validation→compiler→실제 Damage/Heal 및 관찰 포트까지 실행된다. snapshot/live·typed resource/result·유한 집계가 명확하며 Attack/SpellPower·무기 성장/사본 enhancement를 한 번만 계산하고 AT06 projection/save/fingerprint를 함께 전환한다.
- User-Visible Problem: 현재 효과량은 literal이고 SpellPower 입력이 없으며, 장비 copy projection이 강화 UUID를 잃고 실제 수치·표시·저장·평가 입력을 서로 다른 계산으로 만들 위험이 있다. 콘텐츠 작성자가 단위/시점/자원/결과 식을 안전하게 실행·설명할 수 없다.
- User-Visible Completion Condition Set: C01 공개 authored 식과 typed 오류; C02 release snapshot/live/cancel 동작; C03 actual finite/grouped aggregate; C04 Attack100/weapon40 단일 합성 및 SpellPower 분리; C05 source-copy/resource/result identity와 원자적 failure; C06 동일-context amount/trace 및 old V7 round-trip/Certified input truth; C07 실제 T06/applicable T18/T27 및 유한성/비용/전체 회귀; C08 구현 종료 후 요청 범위에 맞는 fresh independent audit. 계획 완료는 이 C01–C08 게임 완료가 아니다.
- In-Scope / Out-of-Scope: In: Frozen CAVER-E01–09, AT06, CRA D01–D10/V01–V21, 위 두 final execution nodes. Out: R07+ 새 기능, fragment rank 제거, R19 final weights/공식 수치, 미정 Replace 우선순위, server/Unity/외부 배포/UI, 개인 save 삭제·자동변환, 콘텐츠 이름별 executor, P2G/GEX/validator/AGENTS/global 설정 변경. 감사는 후속 사용자 요청 범위에서만 실행한다.
- Reviewed Current Behavior / Implementation Delta: `CRA-CAVER-P2G20-003` C01과 source evidence001의 286 current source/data/test files, source003 직접 입력270개 일치, old review34파일 일치. Current literal/void callback/active query/copy-loss/V7 hash와 proposed D01 compiler, D02 sequential effects, D03 capture, D04 tile grouping, D05 composition, D06 prefix read, D07 serialization/fingerprint, D08 error propagation를 구별한다. source hash는 실행 PASS가 아니다.
- Goal-Local Source of Truth / Owning Layer: 정책은 Freeze→PER/DR, 표현/실행 제안은 CRA D01–D08, 최종 실행 배정은 Phase Plan. CompiledAmountPlan은 immutable ContentGeneration; stat composition은 stat_pipeline; owned enhancement는 inventory; exact action/fact는 R04 runtime_facts, lease/read는 R05 ResourceStore; caller-local evaluator가 새 별도 mutable gameplay authority를 만들지 않는다.
- Goal-Local Invariant Set / Source Refs: I01 typed checked deterministic operations와 explicit rounding(D01/E01); I02 captured snapshot·live moment 및 exact capability 보존(D03/E02/E05); I03 explicit finite source/grouping/empty semantics(D04/E03); I04 weapon+growth+copy enhancement 한 번과 cadence 의미 보존(D05/E04/E06); I05 같은 candidate의 state/RNG/UUID/queue/log/resource/result rollback(D02/D06/D08/E08); I06 shared plan·frame/trace, transient 미저장, legacy checksum/content identity·owned data 유지(D07/E07/E09); I07 지원 capability는 actual public executor와 negative evidence를 요구(D09/E08/E09). source ownership의 의미 변경은 해당 P2G owner로 반환한다.
- Code/Data/Schema/Store/Transport/Consumer Boundary: CRA D01–D10의 파일·symbol·types·입출력·처리 순서를 구현 전 읽는다. 신규 game/value_expr 모듈과 ability/authoring+skill_data+GameDataBase compiler, sim/area/projectile/result/resource/fallible callbacks, stats/equipment_data/battle/types/player_spawns/inventory/employee, authoring query/preview/compiled/control/clone, active_run/state 및 combat_power. 기존 src/authoring/tests와 world tests, 신규 tests/value_expr_contract.rs와 external fixtures는 WP04. public transport shape는 core-side additive observation 우선; external server/client 변경은 별도 owner.
- Consumer Verification Boundary: workspace-confirmed — Core loader/runtime/world/profile/save/authoring API/CLI bodies and direct server source captured read-only. External Unity/server deployment compatible 상태는 주장하지 않으며 out-of-scope다. Static describe에는 runtime context 부재를 표시하고 실제 trial preview와 runtime amount는 exact frame에서 검증한다.
- Goal-Local Dependencies / Execution Prerequisites: D10 순서대로 mutation 전 planning-close와 current source/dirty 비교, implementation baseline 보존, 직접 R01/R04/R05 output/closure/currentness 확인, old-build V7/literal fixture 고정, parser bounded visitor·Result callback·prefix witness·zero omission 공유 계약 준비 점검을 수행한다. 그 뒤 WP01→WP02→WP03 통합→WP04 증거 순서. 미래 API는 준비 완료로 가정하지 않는다. single writer, 다른 작업자 overlap이면 쓰기 전에 관찰 범위/격리 해결.
- Validation Plan / Completion Matrix: 아래 27-row exact acceptance→IVA mapping와 CRA D09 V01–V21을 모두 수행. commands/selector discovery/실행 건수/expected actual/source identity는 아래 Command Matrix. 새 fixture는 실제 external RON/production API를 사용하며 private builder만으로 완료하지 않는다. 기술 한도와 실제 성능 측정·fresh audit는 별도 증거다.
- Structural Completion / Legacy and Fallback Disposition: literal authored spelling은 canonical Constant로 남고 별도 literal executor/usefulness 계산은 제거한다. R07 rank/기존 Percent/cadence는 보존. Resource/result zero/last-value, definition-wide copy join, unsupported opcode successful descriptor, checksum/identity bypass, swallowed arbitrary expression error는 제거/금지한다. 새 capture/prefix는 CloneShape/native retention/close/restore 경계에 모두 연결돼야 한다.
- Canonical Documentation Obligation: CRA D10의 core_runtime_contract.md, data_loading_contract.md, skill_target_contract.md, game_rulebook.md 및 가장 가까운 authoring architecture owner에 expression/moment/result/fault/save/profile/descriptor 의미와 supported 제한·실제 예시를 갱신한다. 기존 policy/immutable artifact/실패·감사 verdict는 수정하지 않는다.
- Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan: planning-close manifest는 mutation 전 대조 자료이며 implementation baseline과 다르다. 초기 현재 full baseline을 기록하고 필요 없는 날짜/Phase 시작 재실행은 피한다. source/data/test/config/toolchain/lock/fixture/command/selector 중 relevant input이 달라지면 영향을 받는 IVA 증거를 invalid로 두고 focused revalidation, shared schema/clone/persistence/runtime harness 변경은 broad regression+new baseline을 수행한다. 마지막 source 변경 뒤 full final baseline과 public probes가 필수. 실패/zero-tests/skip/실행 후 변경을 이전 pass로 덮지 않는다.
- Assigned Durable Integration Validation Seam Coverage: `WP-CAVER-04`가 `IVA-CAVER-01`부터 `IVA-CAVER-21`의 모든 durable tests/fixtures/probes와 결과를 소유한다. AG source 수정은 원 owner로 반환하며 감사자가 구현/시험 소스를 고치지 않는다. P2G-60 종료 시 이 증거는 아직 미실행이다.
- Applicable Migration/Compatibility/Rollout/Rollback: old literals·old actual V7 raw checksum을 보존하도록 default+zero omission 및 actual old fixture로 확인한다. nonzero SpellPower는 저장/hash에 포함; changed content와 mismatched save는 typed reject. 삭제/일괄 변환/identity bypass 금지. 실패는 whole candidate rollback, prior committed timestamp prefix 유지. public capability는 full atomic cutover 이후만 supported; 외부 rollout은 N/A(요청 제외).
- Goal-Local Known Unknown / Blocker / Stop Condition: CRA D10의 parser hook, actual prefix interface/callback Result, raw legacy save/current canonical bytes, 현실 workload 비용은 해당 WP owner의 실행 전 준비 및 완료 전 실제 검증이 필요하다. 상한 완화나 새 정책으로 증거 부족을 메우지 않는다. 정책/저장 손실/미정 gameplay 선택 또는 required source inaccessible이면 owner 반환/사용자 질문. 현재 별도 구현 요청 없이는 GEX-00조차 시작하지 않는다.
- Applicable Risk / Reviewer Requirement and Evidence: 교차 schema/runtime/stat/save의 atomic cutover·오류 전파·기본값 checksum·clone 비용 위험을 D10 공유 계약 준비 점검으로 검토한다. 이후 GEX 감사가 요청되면 구현·설계·교정·시험 작성에 참여하지 않은 fresh reviewer와 exact stable target/currentness가 필요하다. 현재 계획 작성자 자기 검토나 P2G validator PASS를 독립 게임 감사로 사용하지 않는다.

#### Completion Matrix

acceptance 문구는 Freeze exact member에서 그대로 파생했다. 각 행의 모든 검증 기대는 CRA D09의 해당 V행과 공유 D계약을 함께 적용한다.

| Exact member / acceptance | Frozen completion condition | Owning validation seam |
| --- | --- | --- |
'''
coverage={1:[[1],[2,3],[2,19]],2:[[5,6],[6,17],[5,6,12]],3:[[7],[8],[7,8,19]],4:[[9],[10],[9,10]],5:[[12],[13],[5,6,12]],6:[[11],[11],[1,11]],7:[[17],[15],[10,16]],8:[[18,19],[19,20],[18,21]],9:[[4,15,18],[21],[10,16,21]]}
for i,entry in enumerate(f['Observable Policy Acceptance Set'],1):
 for j,acc in enumerate(entry['Observable Policy Acceptance'],1):
  seams=', '.join(f'IVA-CAVER-{v:02}' for v in coverage[i][j-1]);s+=f"| `{entry['Policy Element Revision Ref']['Artifact ID']}[{j}]` | {acc} | {seams} / WP-CAVER-04 |\n"
s+='''
#### Command and Changed Input-to-Seam Map

명령은 후속 구현에서 실행할 계획이다. cwd `/mnt/f/work/simulator`; `game_core` crate. 생성 예정 test 이름은 아직 존재하는 selector로 가장하지 않는다. 먼저 `cargo test -p game_core --test value_expr_contract -- --list`로 구현한 test 목록을 확인하고 위 21개 V행/27 acceptance의 실제 비어 있지 않은 selector mapping을 기록한다. test-fixtures flag가 필요한 내부 fixture와 default production 경로를 분리한다.

| 변경 입력 | 명령 / actual seam | 수용 및 증거 |
| --- | --- | --- |
| AST/unit/parser/compiler/catalog | `cargo test -p game_core --lib game::value_expr`; public `cargo test -p game_core --test value_expr_contract` | V01–V04/V07/V16/V18/V19; unknown/depth/unit/rounding/overflow no-publication |
| runtime/source/capture/group/resource/result/Result propagation | `cargo test -p game_core --test skill_runtime_contract`; 신규 value_expr_contract runtime cases; `cargo test -p game_core --lib game::battle` | V05–V08/V12–V14/V19/V20, immediate/area/projectile native+controlled |
| stats/copy/profile/cadence/save | `cargo test -p game_core --lib game::world`; `cargo test -p game_core --test live_item_skill_activation`; 신규 actual old-V7 import test | V09–V11/V15–V17, old raw fixture before mutation, full checksum and copied owner assertions |
| headless public authoring/CLI | `cargo test -p game_core --lib authoring`; `cargo build -p game_core --release --bins`; `target/release/author_content validate <trusted-root> <selection.json>`; describe uses same root/selection, explain/simulate add exact request.json | external fixture acquisition→compiled token→executor/trace, negative exits/diagnostics; no fixture-only proof |
| same-ID override/official legacy content | package_official_content의 실제 CLI usage를 source에서 확인하고 unchanged game_resources/data로 pack 생성, default consume_content_pack/author_content로 load+trial | V04/V16/V18/V21, old/new generation identity, declared support truthful |
| clone/carrier/limits/cancellation/performance | inherited source003 cost harness/source/log를 확인해 동일 input·threshold로 재실행, 별도 V20 many-cast/area/source-copy probe 작성 | requested allocation/RSS 구분, original cap9,030,188 및 실패126 보존, actual finite workload/fault/retained/clone/cleanup |
| integrated final/shared schema/harness/toolchain/lock | `cargo fmt --check`; `cargo test -p game_core --all-targets`; `cargo test -p game_core --all-targets --features test-fixtures`; `cargo check -p game_core --all-targets --features event_log_checks`; `cargo build -p game_core --release --bins` | 실제 exit/nonzero test수/ignored/failed와 current source+fixture+toolchain hash 기록; baseline failure는 원인·scope와 함께 보존, passing subset로 전부 완료 주장 금지 |

각 실제 invocation은 raw stdout/stderr, command/flags/cwd, exit, selector/count, expected/actual, source/fixture/config/lock/toolchain identity를 연결한다. placeholder root/request 경로는 구현자가 실제 생성·검증한 파일로 바꾸고 no-op probe를 PASS로 기록하지 않는다. 새 상수 예제는 공식 balance가 아니다.

#### Validation Dimension Matrix

| Dimension | Disposition | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |
| --- | --- | --- | --- |
| success | required | IVA01–18의 typed public 식/stat/실제 delivery/frame/save | C01–C06 actual output과 frozen oracle 일치 |
| invalid_input | required | IVA01–03/06/09/12/13/15/18/19 | unit/context/copy/future/old checksum/unsupported typed refusal, 기존 상태 무변경 |
| failure_rollback | required | IVA14/15/19/20 native+controlled full-candidate snapshots | HP/resource/RNG/UUID/queue/log/frontier와 prior committed prefix 보존 |
| retry_idempotency | required | IVA12–15/19/20 실패→정상 retry·stale handle·end/cancel/restore | double apply/old authority 재활성 없음, 실패 evidence 유지 |
| ordering_concurrency | required | IVA05/07/08/13/14/20 effect-prefix·same-time·permutation·same-copy isolation | authored effect 순서, explicit aggregate order, logical single-writer semantics; 병렬 writer 승인 아님 |
| boundary_empty_max | required | IVA01–03/07/08/11/19/20 | empty0/error 구별, limit−1/limit/+1, checked overflow·max/min·양수 cadence |
| disconnect_restart_persistence | required | IVA06/12/15 새 application/process V7와 재배치 | persistent copy/growth/enhancement/SP 복원, transient lease/action/result 미복원, checksum no bypass |
| live_data | required | IVA04/16/18/21 실제 외부 RON/공식 pack/default release CLI | parser/validation/compiler/executor 모두 actual, unsupported 공개 성공 금지 |
| producer_consumer | required | IVA01–21, SC-CAVER-01/02 | actual source-copy/state/result/save/preview/explain/fingerprint 일치, 타입 존재만으로 완료하지 않음 |
| legacy_fallback_absence | required | IVA04/09/13/15/16/18/21 | old literal 의미/Percent/rank 보존, duplicate calculator/definition join/default result/checksum bypass 제거 |

## Policy–Phase–Goal Traceability

Freeze의 exact ordered members와 Plan primary/mapping을 기계적으로 파생한 조회다.

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single Goal ID |
| --- | --- | --- | --- | --- |
'''
for i,r in enumerate(f['Ordered code_reviewed Policy Element Revision Ref Set'],1):s+=f"| CAVER-E{i:02} | {r['Artifact ID']} | {f['Decision Revision Ref Set'][i-1]['Artifact ID']} | PHASE-CAVER-01 | GOAL-CAVER-01 |\n"
s+='''
## Goal authoring comparison and handoff

기존 R06에는 authored Goal Set이 없었다. 새 결과는 이전 approved 27 acceptance를 보존하고, actual source로 보완한 상세 알고리즘·state lifetime·error/cancel·raw save 호환·21 integration owners·10 dimensions를 하나의 Single Goal에 연결했다. P2G-50 검증 후에만 current로 선택하며 P2G-60은 exact 계획 인계만 수행한다. 구현 전 D10과 별도 implementation baseline은 필수 미완료다. 원래 사용자 요청의 P2G-60 종료점에서 반드시 멈춘다.
'''
(p/'P2G_50_GOAL_SET_001.ko.md').write_text(s)
print('Goal Set candidate: one Single member, 27 derived acceptances, 21 integration owners, 10 required dimensions')
