# U06 기본 행동·시작 세트·전술과 선택 — Single Goal Set

## Goal Set

- Goal Set Ref: GS-CBST-001
- Predecessor Goal Set Ref: N/A
- Lifecycle Disposition: current
- Exact Prerequisite Composite Basis: WR-CBST-003; FPF-CBST-001; PP-CBST-001; PVR-CBST-001; changes/obligations explicit empty
- Goal Structure Fact: single_phase
- Phase Plan Ref: PP-CBST-001
- Final Policy Freeze Ref: FPF-CBST-001
- Complete Goal Member Set: GOAL-CBST-01
- Applicable P2G-50 Obligation Key Set: explicit empty
- Goal Quality Contract Version: 2
- Authoring Status: Authored
- Activation Status: Not Activated
- Implementation Authorized: No

Goal 작성은 구현 권한이 아니다. 원 연속 요청이 P2G 인계 이후 GEX 구현·검증·독립 감사와 교정을 별도로 승인했다.

### Phase-to-Goal Member Mapping

| Phase ID | Goal ID | Assigned Final Execution Nodes | Cardinality |
| --- | --- | --- | --- |
| PHASE-CBST-01 | GOAL-CBST-01 | WP-CBST-BASIC, WP-CBST-STARTER, WP-CBST-TACTICS, WP-CBST-BLOCK, WP-CBST-TIME, WP-CBST-VERIFY | 1 |

### Shared Validation Baseline Reuse Contract

- User-Approved Directive Ref: PIS-CBST-001 IN-001 연속 요청의 기존 계획/유효 증거 재사용; IN-005 workflow efficiency update. 원래 oracle/필수 독립 감사 유지.
- Baseline Evidence Key: actual source/data/test/config/input/toolchain/lock/dependency, command/selector, raw receipt와 owning Goal.
- Reuse Rule: WORKFLOW_INTAKE_001.json의 U05 product368 exact identity와 문서7개 proven non-impacting delta를 인수한다. U06 새 실행의 증거로 사용하지 않는다.
- Production Input Invalidation / Focused Revalidation Map: profile/ally/basic→V04/05/14/19; starter/grant/deploy→V01–06; tactical/work→V07–12/15/18; movement→V13/14/15; session/host→V16–18; all new fields→V18/19 and inherited clone/shape/queue/native/fault/observer seams.
- Test/Fixture/Probe Change Invalidation / Rebaseline Rule: fixture 바뀐 seam을 current source에서 nonzero 실행하고 원래 입력/oracle/변경 이유와 raw를 보존한다. 새 starter 제품 검증과 legacy 수치 fixture adapter를 구분한다.
- Shared Harness/Schema/Persistence/Transport/Dependency Invalidation Rule: 모든 affected producer/consumer/seam을 invalidation. test bytes만 같아도 runtime 변경 전 evidence를 새 성공으로 쓰지 않는다.
- Phase-Start Rule: 단계 시작만으로 full suite를 반복하지 않는다. 입력 영향과 실패만 재실행을 결정한다.
- Initial Full Baseline Owner / Timing / Command Set: GOAL-CBST-01, mutation 전 U05 CompleteHigh baseline Core1758/1758/1767,host91/99+heap1,fmt/check/event-log/clippy/release/public PC576 및 실제 gameplay31/typedrefusal5, continuation/fault/native/observer 근거를 currentness 확인으로 재사용.
- Final Full Baseline Owner / Timing / Command Set: GOAL-CBST-01 WP-CBST-VERIFY, integrated current Core default/test-fixtures/runtime-observation,host default/observation/heap,fmt/check/event-log/clippy/release, normal6starter/public/stockhost/PC576과 affected141 inherited/31R06.
- Normal Full-Validation Count: initial은 불변 U05 baseline1회 재사용, final integrated set1회 성공 고정. 수정/실패/입력변경에 필요한 반복은 허용한다.
- Failure/Zero-Execution/Post-Run-Change Retry Rule: 실패/skip/zero/실행후입력변경은 pass가 아니다. 도구오류와 제품오류를 raw로 구분하고 교정 후 해당 set 재결속. 기존 R05 승인 peak예외와20분stress 제외는 그대로이며 새 예외 없음.

### Integration Acceptance-to-Goal Mapping

| Integration Acceptance ID / Source | Seam Disposition | Owning Goal ID / Final Execution Node | Locator / Evidence Contract |
| --- | --- | --- | --- |
| IVA-CBST-V01 / CODE_REVIEW_001.ko.md Element Coverage 01; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V01; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V02 / CODE_REVIEW_001.ko.md Element Coverage 02; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V02; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V03 / CODE_REVIEW_001.ko.md Element Coverage 03; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V03; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V04 / CODE_REVIEW_001.ko.md Element Coverage 04; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V04; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V05 / CODE_REVIEW_001.ko.md Element Coverage 05; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V05; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V06 / CODE_REVIEW_001.ko.md Element Coverage 06; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V06; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V07 / CODE_REVIEW_001.ko.md Element Coverage 07; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V07; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V08 / CODE_REVIEW_001.ko.md Element Coverage 08; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V08; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V09 / CODE_REVIEW_001.ko.md Element Coverage 09; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V09; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V10 / CODE_REVIEW_001.ko.md Element Coverage 10; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V10; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V11 / CODE_REVIEW_001.ko.md Element Coverage 11; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V11; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V12 / CODE_REVIEW_001.ko.md Element Coverage 12; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V12; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V13 / CODE_REVIEW_001.ko.md Element Coverage 13; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V13; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V14 / CODE_REVIEW_001.ko.md Element Coverage 14; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V14; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V15 / CODE_REVIEW_001.ko.md Element Coverage 15; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V15; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V16 / CODE_REVIEW_001.ko.md Element Coverage 16; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V16; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V17 / CODE_REVIEW_001.ko.md Element Coverage 17; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V17; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V18 / CODE_REVIEW_001.ko.md Element Coverage 18; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V18; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V19 / CODE_REVIEW_001.ko.md Element Coverage 19; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V19; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |
| IVA-CBST-V20 / CODE_REVIEW_001.ko.md Element Coverage 20; exact Freeze acceptance | assigned_execution_node | GOAL-CBST-01 / WP-CBST-VERIFY | execution/single_goal/COVERAGE.json V20; actual nonzero execution, exact source/data/config/toolchain/input/command/raw receipt and expected-versus-observed; failures and original numerical oracles retained |

## Single Goal Member `GOAL-CBST-01`

- Goal ID: GOAL-CBST-01
- Goal Kind: single
- Phase ID: PHASE-CBST-01
- Assigned Final Execution Node Set: WP-CBST-BASIC, WP-CBST-STARTER, WP-CBST-TACTICS, WP-CBST-BLOCK, WP-CBST-TIME, WP-CBST-VERIFY
- Covered Policy Element Revision Ref Set: PER-CBST-01-REVIEWED-001, PER-CBST-02-REVIEWED-001, PER-CBST-03-REVIEWED-001, PER-CBST-04-REVIEWED-001, PER-CBST-05-REVIEWED-001, PER-CBST-06-REVIEWED-001, PER-CBST-07-REVIEWED-001, PER-CBST-08-REVIEWED-001, PER-CBST-09-REVIEWED-001, PER-CBST-10-REVIEWED-001, PER-CBST-11-REVIEWED-001, PER-CBST-12-REVIEWED-001, PER-CBST-13-REVIEWED-001, PER-CBST-14-REVIEWED-001, PER-CBST-15-REVIEWED-001, PER-CBST-16-REVIEWED-001, PER-CBST-17-REVIEWED-001, PER-CBST-18-REVIEWED-001, PER-CBST-19-REVIEWED-001, PER-CBST-20-REVIEWED-001
- Covered Decision Revision Ref Set: DR-CBST-01-001, DR-CBST-02-001, DR-CBST-03-001, DR-CBST-04-001, DR-CBST-05-001, DR-CBST-06-001, DR-CBST-07-001, DR-CBST-08-001, DR-CBST-09-001, DR-CBST-10-001, DR-CBST-11-001, DR-CBST-12-001, DR-CBST-13-001, DR-CBST-14-001, DR-CBST-15-001, DR-CBST-16-001, DR-CBST-17-001, DR-CBST-18-001, DR-CBST-19-001, DR-CBST-20-001
- Derived Observable Policy Acceptance Coverage: FPF-CBST-001 exact20 member acceptance 전체; CODE_REVIEW_001 Element Coverage V01–20과 위 integration mapping이 실행 경계를 인계한다.
- Objective / Contract-Visible Result: 정상6시작세트의 실제 사본/무기/단일·범위 기본치유와 active를 지급하고, 동시8배치·태세/이동/교대/작업·실제저지·일반/배치/스킬선택과설정/Pause를 한 Core/host 정상 명령·조회 계약으로 완성한다.
- User-Visible Problem: 현재 starter는 모두 동일검/baseline이며 HQ grant가 비어 있다. 순수기본치유와 active아군target, 이동후1.5초자동Attack, 작업퇴장Attack, 실제접촉sweep, 공통focus/settings 및 실행과일치하는preview가 부족하다.
- User-Visible Completion Condition Set: V01–20 전체: TP/동시8/출전우선/철수새실체/엄격profile; 단일·범위치유와6active 정상grant; 태세/Rest/이동/교대/작업/timer; 위치보존저지와basic-only예외; 읽기전용query; 한복원owner/0.5선택/settings/Pause; clone/host재접속/Continue와비용·독립감사.
- In-Scope / Out-of-Scope: BASIC→STARTER, TACTICS→BLOCK, TIME와VERIFY를 한Phase에서 순차 구현. Unity카메라/고스트/카드렌더링, 미합의UI/최종보스/새무기계열, 미래전체채용/원정, 새침식피해, 새live battle disk포맷, unrelated cleanup/remote deploy/20분stress 제외.
- Reviewed Current Behavior / Implementation Delta: CODE_REVIEW_001 RQ01–08/TD01–08/IF01–04. strict basic heal+AllySingle→공통HP; normalcopygrant→live8/roster10; same-unitforcedtimer/workexit→segmentcontact/basicexception; commonrestore/settings→sharedeligibilityDTO와actualhost. 기존damage/U05use/forcedcorrosionclock/U04결과는보존.
- Goal-Local Source of Truth / Owning Layer: raw/compiled weapon/action/fragment definition은 작성의미, BattleCore는 행동/HP/전술/timer/work/block, World candidate는grant/TP/입력원자성, ActiveBattleSession은선택/시간복원, host는실제입력/게시를소유한다. UI가규칙을복제하지않는다.
- Goal-Local Invariant Set / Source Refs: Freeze20요소 exact. heal은적대damageproc없음/피해뒤/부활없음; condition우선/live8; 이동교대같은실체/한TP5/양쪽원자; forced1500→Attack/work즉시; 실제접촉위치보존/basic적대만예외; 선택복원한개/settings·disconnectPause우선; failedcandidate무변경/no-native-cost-oracle완화.
- Code/Data/Schema/Store/Transport/Consumer Boundary: BASIC strict RON→compiled/profile→target/start/resolveHP→queue/fact/clone/query; STARTER real grant/equip→start/recruit/deploy; TACTICS validation→finalpositions→work/forcedtimer/effects/block/query; BLOCK segment/backend/body/log→relation; TIME command→commonrestore/settings→capability/hostDTO/tick/reconnect. TD01–08의순서/거절대안 안에서 exactRust타입/이벤트/수치를GEX가정한다.
- Consumer Verification Boundary: required local CoreApplication 정상command/result/query, loader/authoring/catalog, headless kernel, 실제host actor/stockwebsocket, controlledclone/roundtrip와U04 durableContinue/SavePoint/원본보호.149개소스검토. 제공되지않은Unity화면/remote deployed실행은제외.
- Goal-Local Dependencies / Execution Prerequisites: U05 CompleteHigh와product368/운영문서delta현재성, exactFreeze/Plan/receipt/Goal/Handoff. GEX00에서dirty/선행원본인수. BASIC후STARTER, TACTICS후BLOCK; 다른WP는기존stable unit/action/session seam. 새field·queuecarrier의clone/extent/fault를먼저준비·검증하고사용가능전API를가정하지않는다.
- Validation Plan / Completion Matrix: V01–20 exactacceptance와아래10차원; success/invalid/stale/fault/CC/Pause/same-time/death/withdraw/clone/reconnect/strictnegative를actualnonzero로검증. 작은schema/clone/queue/extent→feature→normal6starterpublicencounter→필수fullCore/host/native/observer/PC. 각141선행+31R06impact를명시.
- Structural Completion / Legacy and Fallback Disposition: 기본heal을damage/흡혈/보호막으로대체하거나active를baselineIDs에넣는fallback없음. Defense-onlymove/channel-onlystancegate/workexitDefense 제거; logicaltile snap/unswept contact 금지. skill-only독립restore를공통owner로전환하고settings우회차단. 기존피해특수총/적자동행동/U05수명/U04저장owner는명시유지.
- Canonical Documentation Obligation: skill_target_contract.md/core_runtime_contract.md/data_loading_contract.md/host unity_bridge_protocol의 실제typedheal/allytarget/grant/tactics/query/time입력변경 갱신. 최신game_rulebook/game_rules의 owner위치/작성초기값을정확히연결. 기존감사원본보존; ROADMAP완료게시와후속선택은독립감사뒤.
- Inherited Baseline Contract / Changed Input-to-Seam Impact Map / Focused Revalidation and Rebaseline Plan: 공통baseline계약과WORKFLOW_INTAKE_001을사용해141개+31R06각각reuse/affected/intentionalchange/새evidence를기록. 새로운starter로깨지는legacyfixture는원래수치입력을명시복원하며oracle완화없음. 선행action/source/cancel/fault/restore/nativebounds/5관측모드의변경영향을재실행.
- Assigned Durable Integration Validation Seam Coverage: IVA-CBST-V01–V20 전부 GOAL-CBST-01/WP-CBST-VERIFY. durable actual command/query/authoring/host/probe는구현자가자체완료하고감사자에게구현을미루지않는다.
- Applicable Migration/Compatibility/Rollout/Rollback: schema/compiled/currentlivecontent/queryconsumer를각WP내동시전환. unsupported형식은typed거절/원본보존하며자동삭제변환없음. liveclone/rollback은healcarrier/timer/work/focus/settings를독립보존. 같은actor재접속과U04미확정pre-entryContinue를구분. actualbefore/afterarchive와fuzz0복원증거.
- Goal-Local Known Unknown / Blocker / Stop Condition: exacttypedpayload/수치/event/sweep/DTO선택은TD01–08내GEX상세이며현재policy/sourcegap0. 새material의미/필수source/단계경계오류면최초P2Gowner에서불변successor수리. 사용자가중단/승인범위완료/권한환경으로해결불가실제장애일때만멈춤.
- Applicable Risk / Reviewer Requirement and Evidence: 구현비참여GPT-6AstraHigh가20acceptance/10차원/141선행+31R06과실제source/currentness/복원/비용/host를독립감사. healproc누출/queue큰variant/clone누락, 교대부분성공/작업timer경쟁, 빠른접촉누락/기존저지축출, 중첩복원/settingsdisconnect재개가주요위험. finding교정·재감사CompleteHigh전U07종속구현금지.

### Validation Dimension Matrix

| Dimension | Disposition | Validation Seam / Evidence Contract | Completion Requirement / N/A Reason |
| --- | --- | --- | --- |
| success | required | V01–20 public normal6starter encounter | 실제치유/active/grant/TP/tactics/block/time/query 일치 |
| invalid_input | required | V02/04/05/06/09/11/15–17 strict/stale/one-side invalid | typed거절과TP/위치/HP/selection/revision 불변 |
| failure_rollback | required | V06/10–12/18/19 controlledfault and clone | 사본binding/양쪽위치/timer/queue/facts/fullcandidate rollback |
| retry_idempotency | required | V03/10/12/16/17 duplicate command/stale epoch | onewithdraw/oneworkcompletion/no staleAttack/no restore overwrite |
| ordering_concurrency | required | V05/10–14/17 simultaneous damage/heal/work/timer/contact | damage first/no revive; stablecapacity/contact; command order/Pause |
| boundary_empty_max | required | V01/02/05/06/13 fullHP/0target/live8/roster10/TP99/fastsegment | caps/lowHPties/emptyheal/no tunneling/currentposition |
| disconnect_restart_persistence | required | V16–18 actualhostdisconnect/settings/focus/Continue | paused restore priorities and U04 rights/originals; cloneindependent |
| live_data | required | V04–06/19 actualRON6starter+6HQ/strictnegative | loader→grantcopy→runtimebasic/active outcomes;no directkernelonlysuccess |
| producer_consumer | required | V04–06/15–19 authoring/Core/query/stockhost/cloneextent | samecurrentauthoritative semantics and actual size/work accounting |
| legacy_fallback_absence | required | V04–07/09/12–17 negative/source+actualoldregressions | no damageheal/baselineactive/Defense-onlymove/snap/separatefocusrestore;preservedoldvalidcontracts |

## Policy–Phase–Goal Traceability

| Stable Element ID | Policy Element Revision Ref | Decision Revision Ref | Primary Phase ID | Single/Phase Goal ID |
| --- | --- | --- | --- | --- |
| PE-CBST-01 | PER-CBST-01-REVIEWED-001 | DR-CBST-01-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-02 | PER-CBST-02-REVIEWED-001 | DR-CBST-02-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-03 | PER-CBST-03-REVIEWED-001 | DR-CBST-03-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-04 | PER-CBST-04-REVIEWED-001 | DR-CBST-04-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-05 | PER-CBST-05-REVIEWED-001 | DR-CBST-05-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-06 | PER-CBST-06-REVIEWED-001 | DR-CBST-06-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-07 | PER-CBST-07-REVIEWED-001 | DR-CBST-07-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-08 | PER-CBST-08-REVIEWED-001 | DR-CBST-08-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-09 | PER-CBST-09-REVIEWED-001 | DR-CBST-09-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-10 | PER-CBST-10-REVIEWED-001 | DR-CBST-10-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-11 | PER-CBST-11-REVIEWED-001 | DR-CBST-11-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-12 | PER-CBST-12-REVIEWED-001 | DR-CBST-12-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-13 | PER-CBST-13-REVIEWED-001 | DR-CBST-13-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-14 | PER-CBST-14-REVIEWED-001 | DR-CBST-14-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-15 | PER-CBST-15-REVIEWED-001 | DR-CBST-15-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-16 | PER-CBST-16-REVIEWED-001 | DR-CBST-16-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-17 | PER-CBST-17-REVIEWED-001 | DR-CBST-17-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-18 | PER-CBST-18-REVIEWED-001 | DR-CBST-18-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-19 | PER-CBST-19-REVIEWED-001 | DR-CBST-19-001 | PHASE-CBST-01 | GOAL-CBST-01 |
| PE-CBST-20 | PER-CBST-20-REVIEWED-001 | DR-CBST-20-001 | PHASE-CBST-01 | GOAL-CBST-01 |
