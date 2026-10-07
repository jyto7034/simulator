# Experiments

- Goal: `current_runtime_architecture_deep_review`
- Status: `complete`
- Baseline: `BASELINE-20260822-064427-KST`

## 기록 규칙

실제로 실행하거나 읽은 bounded investigation만 시간순으로 기록한다. 계획을 결과처럼 기록하지 않는다.

각 항목은 가능한 한 다음을 포함한다.

```markdown
## E-###. <제목>

- 시각:
- 기준선:
- criterion:
- 질문/가설:
- 정확한 command 또는 read scope:
- 실행 위치:
- 입력/workload:
- 변경한 내용:
- 실제 결과:
- 해석의 한계:
- 연결된 map/finding/gap:
```

## 실행 기록

## E-001. Candidate source identity와 heading routing catalog

- 시각: `2026-08-22T06:44:27+09:00` ~ `2026-08-22T06:45:40+09:00`
- 기준선: baseline 생성 전 source-catalog 단계
- criterion: A00
- 질문/가설: 후보 문서를 전부 정독하지 않고도 책임·freshness·후속 read scope를 정할 수 있는가?
- 정확한 command 또는 read scope: 13개 candidate의 `stat`, `sha256sum`, Markdown heading scan
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: GOAL_PROMPT에 열거된 13개 경로
- 변경한 내용: `DOCUMENT_SOURCE_MAP.md` 생성
- 실제 결과: 모든 경로가 존재했다. Byte size/hash와 heading-based responsibility를 catalog했고 SRC-13은 initial
  skip, SRC-05~12는 matching scenario까지 defer했다. Candidate 본문은 읽지 않았다.
- 해석의 한계: heading은 routing metadata이며 policy/implementation evidence가 아니다.
- 연결된 map/finding/gap: `DOCUMENT_SOURCE_MAP.md`; finding/gap 없음

## E-002. Dirty runtime/test baseline 식별

- 시각: `2026-08-22T06:44:27+09:00` ~ `2026-08-22T06:46:47+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A00
- 질문/가설: 대규모 dirty tree를 commit ID만 사용하지 않고 current content로 재현할 수 있는가?
- 정확한 command 또는 read scope: `git rev-parse HEAD`, `git status --short`, toolchain/OS query, sorted
  runtime/test `sha256sum` manifest와 candidate-source identity
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: `Cargo.toml`, `src/lib.rs`, `src/game/**`, `tests/**`, `../Cargo.lock`, 13 source candidates
- 변경한 내용: `BASELINE.md`, `BASELINE_MANIFEST.sha256`, `BASELINE_GIT_STATUS.txt` 생성
- 실제 결과: HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`; runtime/test 184 files,
  5,553,490 bytes, aggregate `ae1b66e9218ee493bffaa4da3f9b9d7f0bbd8849516d6e2a5f49749be78acb31`.
  Git status는 325 lines였고 Goal 시작 전부터 production/docs/adjacent projects가 광범위하게 dirty했다.
- 해석의 한계: deleted paths는 Git-status capture에만 있고 current manifest에는 없다. Live data content는 아직
  접근·snapshot하지 않았다.
- 연결된 map/finding/gap: `BASELINE.md`, `DOCUMENT_SOURCE_MAP.md`; finding/gap 없음

## E-003. 최초 broad test baseline

- 시각: `2026-08-22T06:47:49+09:00` ~ `2026-08-22T06:49:18+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A00, A06
- 질문/가설: 현재 dirty baseline이 all-target/all-feature test suite에서 재현 가능한가?
- 정확한 command 또는 read scope: `cargo test --all-targets --all-features`
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: current workspace, `event_log_checks` 포함 all features
- 변경한 내용: production/test/data 변경 없음; Cargo build artifacts only
- 실제 결과: exit code 0. `src/lib.rs` 1080 passed. Four fixture-generator binaries each 0 tests.
  Integration binaries: `live_item_skill_activation` 4, `live_skill_catalog_audit` 4, `ron_loading` 25,
  `skill_runtime_contract` 14, `skill_test_suite` 4 passed. Total runner-reported executed tests 1131; failures,
  ignored, measured and filtered-out tests were 0 in every reported binary.
- 해석의 한계: all-pass는 architecture 적합성이나 Rulebook alignment를 증명하지 않는다. Test suite는 live RON을
  읽었지만 this Goal은 아직 adjacent data를 architecture evidence로 직접 조사하지 않았다.
- 연결된 map/finding/gap: EVIDENCE_MATRIX A00/A06; finding/gap 없음

## E-004. Formatter baseline

- 시각: `2026-08-22T06:49:40+09:00` ~ `2026-08-22T06:49:42+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A00, A06
- 질문/가설: current baseline이 formatter check를 통과하는가?
- 정확한 command 또는 read scope: `cargo fmt --all -- --check`
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: workspace Rust targets
- 변경한 내용: 없음 (`--check`)
- 실제 결과: exit code 0, stdout/stderr 없음.
- 해석의 한계: formatting은 architecture quality나 change fitness 증거가 아니다.
- 연결된 map/finding/gap: EVIDENCE_MATRIX A00/A06; finding/gap 없음

## E-005. Strict clippy baseline

- 시각: `2026-08-22T06:49:54+09:00` ~ `2026-08-22T06:50:25+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A00, A06
- 질문/가설: current baseline이 all-target/all-feature strict lint를 통과하는가?
- 정확한 command 또는 read scope: `cargo clippy --all-targets --all-features -- -D warnings`
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: current workspace, all targets/features
- 변경한 내용: production/test/data 변경 없음; Cargo build artifacts only
- 실제 결과: exit code 101. Runner는 lib compile에 82 previous errors, lib-test compile에 133 previous errors를
  보고했다. 대표 categories는 unusual byte grouping, unnecessary `map_or`, too-many-arguments, clone-on-copy,
  field-reassign-with-default, type-complexity, items-after-test-module, unnecessary-get-then-check와 test-style lint다.
- 해석의 한계: counts는 target 간 중복을 포함하며 architecture finding 수가 아니다. 이 Goal은 lint remediation을
  수행하지 않는다. 일부 lint는 API/cognitive-load 조사 신호로만 route한다.
- 연결된 map/finding/gap: EVIDENCE_MATRIX A00/A06; finding/gap 없음

## E-006. Game mode와 run-start vertical trace

- 시각: `2026-08-22T06:51:00+09:00` ~ `2026-08-22T06:53:30+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A00, A01, A02, A03
- 질문/가설: Rulebook의 game identity/mode/core loop가 current production entry와 typed authority에 연결되는가?
- 정확한 command 또는 read scope: Rulebook lines 1-141; `game/mod.rs`; `world.rs:43-805`; declarations in
  `world/state.rs`, `battle/mod.rs`, `data/mod.rs`; `world/active_run.rs:1-157`; `map/progression.rs:1-120`; matching tests
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: StartNewGame → pre-run briefing → starter selection → RunProgression → ViewingMap
- 변경한 내용: `REQUIREMENT_PRESERVATION_MAP.md`, `DOMAIN_MODEL_MAP.md`, source route record
- 실제 결과: typed `GameMode` flows from required command input through `pending_game_mode` into `RunProgression`.
  Endless tutorial gating occurs before state mutation. Standard/Endless progression is enum-owned and protected by focused
  tests. `GameCore` is a private-state application root but routes all command families and coordinates many state roots.
- 해석의 한계: only run start/mode was traced. Map, battle, reward/failure and persistence scenarios remain unclassified.
- 연결된 map/finding/gap: REQ-001, ROUTE-001, `DOMAIN_MODEL_MAP.md`; ARCH-REVIEW-001 hypothesis

## E-007. Facility graph와 node lifecycle vertical trace

- 시각: `2026-08-22T06:54:00+09:00` ~ `2026-08-22T06:57:30+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A00, A01, A02, A04, A05
- 질문/가설: exploration/node policy가 cohesive graph authority와 atomic enter/complete boundary로 구현되는가?
- 정확한 command 또는 read scope: Rulebook lines 142-337; declarations in `map/**` and node-related world modules;
  `world/node_flow.rs:180-394,602-1085,1235-1325`; `map/progression.rs:150-335`; checkpoint/RunState field definitions
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: generate Floor → select reachable node → confirm entry → complete node/floor/run → checkpoint restore
- 변경한 내용: REQ-002, `AGGREGATE_STATE_MAP.md`, domain/source maps
- 실제 결과: facility template and typed directional graph own navigation. Select opens a preview without movement;
  confirm revalidates inside a whole-state transaction. Map completion plans a typed outcome on clones and commits it.
  Checkpoint validates and restores the map/progression generation together.
- 해석의 한계: event/shop/reward handler internals and actual change amplification were not traced in this investigation.
- 연결된 map/finding/gap: REQ-002, ROUTE-002, `AGGREGATE_STATE_MAP.md`, ARCH-REVIEW-001

## E-008. Player-visible battle와 application session trace

- 시각: `2026-08-22T06:58:00+09:00` ~ `2026-08-22T07:01:30+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A00, A01, A02, A04, A07
- 질문/가설: DefenseRoute tactical policy가 one-session authority와 atomic command boundary에 연결되는가?
- 정확한 command 또는 read scope: Rulebook lines 338-596; `world/combat.rs` declarations and setup/advance/tactical
  transaction paths; `world/state.rs:340-440`; `battle/core/mod.rs:60-260`
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: combat setup → server tick; deploy/withdraw/stance/relocate/swap/manual skill
- 변경한 내용: REQ-003, aggregate/domain/dependency maps, architecture hypotheses
- 실제 결과: `ActiveBattleSession` wraps `BattleCore`, execution, deployment, tactical resource, playback/focus and
  revision. Advance and tactical paths clone complete staged state, produce projection, then publish. Typed kernel commands and
  focused tests protect tactical semantics.
- 해석의 한계: target/delivery details, performance magnitude and boss extension fan-out remain pending.
- 연결된 map/finding/gap: REQ-003, ARCH-REVIEW-002/003

## E-009. Battle kernel contract와 dependency trace

- 시각: `2026-08-22T07:01:30+09:00` ~ `2026-08-22T07:04:30+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A02, A03, A06, A07, A10
- 질문/가설: Battle kernel이 deterministic domain boundary로 독립되어 있고 selected runtime contract가 current한가?
- 정확한 command 또는 read scope: Rulebook lines 597-718; SRC-02 lines 20-118,404-455;
  `battle/core/sim.rs:55-220,1580-1795`; battle/setup/core import and public-declaration inventory
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: validated setup → typed command/event step → terminal/finalize → result projection
- 변경한 내용: `DEPENDENCY_DIRECTION_MAP.md`, source route/classification, REQ-003 update
- 실제 결과: named terminal outcome, stable execution state, typed live commands and event-log/RNG separation match selected
  contract and tests. Kernel is not dependency-independent: it stores full `GameDataBase`, imports behavior error/DTO types and
  exposes several mutable fields.
- 해석의 한계: static dependency evidence does not alone prove change amplification; scenarios 1~3 will test it.
- 연결된 map/finding/gap: ROUTE-003/004, dependency map, ARCH-REVIEW-002/003

## E-010. Skill activation/target/delivery/interrupt change-scenario trace

- 시각: `2026-08-22T07:00:07+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A02, A03, A04, A05, A06, A07
- 질문/가설: 새 activation/target/delivery/interrupt 조합이 한 owner와 compiler-enforced path로 확장되는가?
- 정확한 command 또는 read scope: SRC-04 lines 14-416; SRC-10 lines 50-107; `ability.rs:80-830` selected types;
  `data/skill_data.rs:330-590`; `skill_fragment.rs:700-790`; `battle/core/sim.rs:500-900,1630-1795,2780-3015`;
  `battle/core/{target_usefulness,skill_runtime/**}` public/internal function and selected implementation trace;
  `world/combat.rs:2480-2805`; `world/state.rs:920-1035`; matching battle-core/world tests.
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: fragment-authored activation plus player-selected delayed hostile delivery/interrupt as a representative
  change; no production mutation or runtime probe.
- 변경한 내용: `CHANGE_SCENARIO_MATRIX.md`, source/requirement/aggregate/dependency maps and findings.
- 실제 결과: cast selection, target semantics, reach, step retarget, delivery and interrupt identity are separate typed
  authorities and selected contract matches code. Loader validation and exhaustive enums protect ordinary target/delivery
  extension. Activation policy is less data-owned: active fragments hard-code Manual. Player activation is also not routed
  through the staged tactical-session helper; an invalid future event cursor can be rejected after cast/revision mutation.
- 해석의 한계: no failure injection was run; the post-mutation error is a direct control-flow proof, while operational
  frequency/host behavior is unmeasured. S03 alone cannot determine whole-core disposition.
- 연결된 map/finding/gap: S03, ROUTE-005/006, ARCH-REVIEW-003/004

## E-011. Boss phase/summon/omen/special-result change-scenario trace

- 시각: `2026-08-22`
- 기준선: `BASELINE-20260822-064427-KST`; selected adjacent live-data identities DATA-001/002
- criterion: A01, A02, A03, A04, A05, A06, A08, A10
- 질문/가설: simple phase/summon and cross-run boss mechanics can share one coherent runtime owner without policy leakage?
- 정확한 command 또는 read scope: SRC-01 lines 1188-1266; SRC-02 lines 875-958; SRC-07 lines 1-48; SRC-11 lines
  1157-1293 and 1425-1507; `data/boss_runtime_data.rs` authored/compiled types and selected validation; `battle/scenario.rs`
  trigger/terminal/setup validation; `combat_setup/boss_runtime.rs:1-230`; `battle/core/boss_runtime.rs` types/API/test inventory;
  DATA-001 lines 1-480 and DATA-002 lines 1465-1565.
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: Nothing There phase chain, current summon/signal, egg later-phase rematch and bounded Little Red contract.
- 변경한 내용: S01 and source/runtime-data routing records.
- 실제 결과: phase/current-task/summon/signal terminal extensions are data-driven behind strict compilation. New task vocabulary
  has explicit authored/compiled/runtime/test fan-out and compiler guards. Full Little Red/egg/omen pressure spans battle-local
  behavior, encounter orchestration and run progression; current actor setup intentionally accepts only opponent spawns. One
  Boss runtime should not own those run/interaction policies.
- 해석의 한계: future design sections contain open policy and are boundary stress tests, not required implementation. Only one
  small production Boss Runtime reference is live, so production diversity is not demonstrated.
- 연결된 map/finding/gap: S01, REQ-007, ROUTE-007~010

## E-012. Movement mode/collision/route change-scenario trace

- 시각: `2026-08-22`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A01, A02, A03, A04, A05, A06, A09, A10
- 질문/가설: a new movement/collision/route policy can extend one bounded subsystem with deterministic tests?
- 정확한 command 또는 read scope: selected SRC-02 movement section already routed by ROUTE-004;
  `battle/types.rs:270-310`; `data/abnormality_data.rs:65-105`; `battle/scenario.rs:195-225`;
  `battle/core/movement/{types,engine,planner,blocking,rapier_backend}.rs` selected production paths and test inventory;
  movement call sites in `sim.rs`, boss and tactical modules.
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: third mobility kind, explicit static/unit collision policy and non-cell route representation.
- 변경한 내용: S02 and dependency/domain observations; no production mutation or probe.
- 실제 결과: planner → typed input → engine/backend → typed output is a cohesive seam; boss reuses it and logical blocking is
  intentionally separate from physical overlap. Route-progress queries are duplicated consumers of `EnemyMovementPlan`, and
  BattleCore owns the concrete Rapier backend, but focused deterministic tests are strong. Small policy/query refactors are
  narrower than subsystem replacement.
- 해석의 한계: no high-entity performance probe yet; only current Ground/Airborne and cell-path modes have production evidence.
- 연결된 map/finding/gap: S02, ARCH-REVIEW-003, performance scenario pending

## E-013. Run Relic/Monolith progression change-scenario trace

- 시각: `2026-08-22`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A01, A02, A04, A05, A08, A10, A11
- 질문/가설: future Relic을 current Monolith 방식으로 통합할 때 run/growth 소유 경계가 유지되는가?
- 정확한 command 또는 read scope: SRC-01 lines 719-1005; SRC-05 lines 14-232; SRC-06 lines 39-140;
  `monolith.rs:1-360`; Monolith references in map encounter/content/node, shop, VN, combat, snapshot, checkpoint and
  capability modules; `encounter_progression.rs` selected state/plan methods.
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: add run-owned Relic inventory/loadout, reward/shop acquisition and battle modifiers while retaining Monolith.
- 변경한 내용: S04 and source routes; no production mutation.
- 실제 결과: Monolith is a cohesive ledger with deterministic identities and staged resolution, but its integration spans many
  application modules. No Relic runtime exists. A separate `RunRelicState` plus loadout compiler can reuse reward/checkpoint
  seams without putting Relic policy in Monolith or mutating shared skill/equipment definitions.
- 해석의 한계: Relic values, exact hook vocabulary and stacking policy are draft/open; extension shape, not gameplay values,
  was evaluated.
- 연결된 map/finding/gap: S04, ROUTE-011~013, ARCH-REVIEW-001

## E-014. Skill fragment form/growth schema trace

- 시각: `2026-08-22`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A01, A02, A04, A05, A08, A10
- 질문/가설: current fragment inventory/loadout/compiler can express selectable future forms without battle rewrite?
- 정확한 command 또는 read scope: SRC-01 lines 719-1005; SRC-06 lines 39-140;
  `data/skill_fragment_data.rs` effect/validation/database paths; `skill_fragment.rs` inventory/progress/loadout/profile
  compilation; maintenance handlers, DTO and checkpoint references.
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: replace/extend ordinal upgrade + awakened Boolean with selectable stable forms and activation policy.
- 변경한 내용: S05 and REQ-004 classification; no production mutation.
- 실제 결과: ownership, loadout and battle-profile compilation are already distinct and worth preserving. The current
  ordinal/Boolean progression cannot represent several unlocked/selectable forms cleanly; a bounded versioned schema migration
  to stable form IDs is sufficient. Manual activation is currently a compiler literal and should move into authored data.
- 해석의 한계: future note does not itself approve removal of current upgrade gameplay, so no migration value is prescribed.
- 연결된 map/finding/gap: S05, ROUTE-011/013, REQ-004

## E-015. EGO corrosion cross-boundary lifecycle trace

- 시각: `2026-08-22`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A01, A02, A03, A04, A05, A07, A08, A10
- 질문/가설: equipment-set eligibility, battle substitution and post-attempt Breakdown can have one coherent owner?
- 정확한 command 또는 read scope: SRC-08 lines 15-136; SRC-01 lines 1075-1173; `employee.rs` condition/activity paths;
  `BattleEmployeeAttemptState`, equipment/fragment loadout checks, current summon/withdraw/result staging and production search
  for corrosion concepts.
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: explicit corrosion command withdrawing one employee, summoning an allied abnormality and committing trauma/
  deadline consequence.
- 변경한 내용: S06 and REQ-006 notes; no production mutation.
- 실제 결과: current code has reusable typed primitives but no EGO corrosion lifecycle. Eligibility is an application query,
  summoned-actor lifecycle is battle-local, and collapse deadline is run/checkpoint state. A typed cross-layer use case and
  reasoned clock gate fit; putting all policy in Boss Runtime would violate ownership.
- 해석의 한계: exact timing/presentation, equipment origin semantics and node limit are not live-authoritative.
- 연결된 map/finding/gap: S06, ROUTE-014/017, REQ-006, ARCH-REVIEW-004

## E-016. Generic dice/tool change-scenario trace

- 시각: `2026-08-22`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A01, A02, A04, A05, A07, A08, A10
- 질문/가설: Event d20 can become a generic deterministic dice primitive without centralizing consumer effects?
- 정확한 command 또는 read scope: SRC-09 lines 16-114; `data/event_data.rs` checks; `resources/selection.rs`
  `EventCommittedCheckState`; event node choice resolution; consumable schema/use; economy grant staging and matching tests.
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: consumer-defined faces/critical policy, stable request ID/receipt, run-limited tool use and atomic effect.
- 변경한 내용: S07 and source route; no production mutation.
- 실제 결과: Event already provides deterministic seed, a committed receipt and duplicate reuse. A pure roll planner can
  generalize this while the Event/tool consumer owns its ledger/effect. Current consumable/economy staging is reusable, but a
  common application transaction must cover receipt plus effect publication.
- 해석의 한계: no generic dice/tool production workload or Giant Sap ledger exists; performance was not measured.
- 연결된 map/finding/gap: S07, ROUTE-015, ARCH-REVIEW-001/004

## E-017. Roguelike node/session/application-boundary trace

- 시각: `2026-08-22`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A01, A02, A03, A04, A05, A07, A08, A10, A11
- 질문/가설: one new node/session/effect can be added through a bounded owner with atomic result and persistence coverage?
- 정확한 command 또는 read scope: SRC-01 lines 1006-1173; `map/types.rs`, `map/session.rs`, facility template/generator/
  progression declaration and category-match inventory; `resources/{selection,state}.rs`; `world/state.rs`, `node_flow.rs`,
  `map_content.rs`, `reward.rs`, `shop.rs`, `maintenance.rs`, `headquarters.rs`, root behavior routing and snapshot/checkpoint
  declarations. Static counts: `MapNodeCategory` in 39 files, `ActiveNodeContent` in 14, `GameState::` in 23.
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: add a genuinely new roguelike content category with resumable session, reward/effect and checkpoint semantics.
- 변경한 내용: S08, aggregate/domain/source/requirement maps and finding updates; no production mutation.
- 실제 결과: graph generation, deterministic node identity, typed sessions and full non-battle rollback are strong assets.
  New category semantics cross several parallel closed state representations. `NodeSession`, `ActiveNodeContent`, `GameState`
  and RunState session maps can disagree structurally, and `RunState` Deref hides `EncounterProgressionState` ownership. Root
  transaction invocation remains handler-local across command families.
- 해석의 한계: static fan-out includes legitimate exhaustive matches; no deliberately injected node or failure test was added.
- 연결된 map/finding/gap: S08, ROUTE-016/017, REQ-002/005/006, ARCH-REVIEW-001/004

## E-018. Test architecture와 behavior-oracle inventory

- 시각: `2026-08-22`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A06, A11
- 질문/가설: current tests를 subsystem rewrite와 application-boundary migration의 old/new oracle로 재사용할 수 있는가?
- 정확한 command 또는 read scope: `Cargo.toml`; `src/game/world/tests/**`, `tests/**`, inline `#[cfg(test)]` inventory;
  checkpoint, battle atomicity, node-session, movement, skill and live-RON representative tests. Static inventory found 1,131
  `#[test]` attributes, 109 Rust files containing test configuration/tests, and 26 files/1,165,045 bytes in
  `src/game/world/tests` plus integration `tests`. Search for `proptest|quickcheck|criterion` returned no match.
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: public/near-root commands, isolated domain engines, live embedded data and invalid-checkpoint mutation matrix.
- 변경한 내용: Goal evidence documents only.
- 실제 결과: leaf engines can be tested without server/network/filesystem; world tests compare observable result and broad
  authority fingerprints. Strict V4 tests reject missing/extra/legacy fields, exercise 19 graph mutations before publication,
  and compare restore-recapture semantics. Live-RON integration tests use the production loader. The suite has no property-test,
  benchmark or universal mutating-command/failure-injection harness; broad fixtures often construct a large GameCore/GameData
  context, and direct battle handlers are not all covered by the same rollback conformance test.
- 해석의 한계: attribute/file counts are inventory, not quality scores. The broad suite was executed only once as E-003;
  no production tests were changed or rerun.
- 연결된 map/finding/gap: `TESTABILITY_MAP.md`, ARCH-REVIEW-003/004/005

## E-019. Core/host operation boundary와 data/protocol/save evolution trace

- 시각: `2026-08-22`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A03, A07, A08, A10, A11
- 질문/가설: Core가 durable I/O를 소유하지 않으면서 validated data와 versioned continuation port를 제공하고,
  protocol/save를 domain과 별도로 진화시킬 수 있는가?
- 정확한 command 또는 read scope: SRC-03 lines 15-58,242-270,349-378; SRC-02 lines 455-483;
  `data/live_ron.rs:100-162,1424-1550,2570-2609`; `data/mod.rs:167-215`; `world/active_run.rs:1-138`;
  `behavior.rs` public DTO/version/strict-serde inventory; tracing/metrics call-site inventory; matching live-RON and checkpoint
  tests.
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: embedded bundle startup, candidate publication, strict Core V4 checkpoint export/continue and host handoff.
- 변경한 내용: source routes and Goal evidence documents only.
- 실제 결과: one macro owns 24 gameplay source paths; strict manifest plus parse/local/cross/generated setup validation occurs
  before one `Arc<GameDataBase>` publication. Fixture builders are separate. Core exposes bytes+monotonic metadata and rollback,
  while selected contract assigns checksum/current/backup/temp/fsync/lock/journal/tombstone to host. Current protocol DTOs are
  centralized in `behavior.rs` (6,459 lines/231,584 bytes, 166 public struct/enum declarations by static scan), have local
  setup/catalog/event versions but no single root protocol version, and use strict unknown-field rejection selectively. Save is
  strict V4 with no migration reader.
- 해석의 한계: game_server and Unity were not audited, so actual durable crash behavior and consumer compatibility are not
  certified. Static DTO counts are change-surface evidence only when combined with S03/S08 paths. SRC-03 prose count of 23 is
  stale versus code's generated count of 24.
- 연결된 map/finding/gap: ROUTE-018/019, `OPERATIONS_BOUNDARY_MAP.md`, `DATA_CONTRACT_EVOLUTION_MAP.md`,
  ARCH-REVIEW-001/003/005

## E-020. Current performance/resource scaling probe

- 시각: `2026-08-22T07:30:00+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A09, A05, A07
- 질문/가설: complete active-battle staging and constant-tail reconnect query cost grow with retained log/state, while current
  ordinary workloads remain light.
- 정확한 command 또는 read scope: preregistered `/tmp/core_arch_perf` release harness, then
  `cargo run --release --offline --manifest-path /tmp/core_arch_perf/Cargo.toml`; production code was consumed as a path
  dependency. Harness identities: `Cargo.toml` `dd9d4a...e35`, `Cargo.lock` `816785...914`, `src/main.rs`
  `203cf2...5c0`.
- 실행 위치: WSL2 Linux x86_64, AMD Ryzen 9 7950X 16c/32t, rustc 1.91.1, optimized release, one unpinned process.
- 입력/workload: 10 live loads; 2,000 map queries; 500 snapshots; 2,000 nonbattle rollback captures; 500 checkpoint exports;
  an initial live combat with 2,000 zero ticks/200 one-ms ticks/500 reconnect tails; synthetic 1K/10K/100K append-only event
  histories; staged one-ms steps at 10/100/1,000 retained units.
- 변경한 내용: `/tmp/core_arch_perf` only; no production/test/live-data/canonical document change.
- 실제 결과: p95 was 144.110ms live load, 4.122us map query, 196.552us/46,984 bytes run snapshot, 3.492us nonbattle
  rollback capture, 22.621us/17,137 bytes checkpoint export, 10.200us zero tick, 10.848us one-ms tick, and 1.298us current
  reconnect tail. At 1K/10K/100K event entries, `BattleCore` clone p95 was 13.576us/125.243us/1.879ms; last-10 query p95
  0.802us/3.940us/96.916us; full delta p95 25.436us/222.856us/2.741ms. Event-log JSON was
  123,809/1,257,810/12,777,811 bytes. Staged one-ms work for 10/100/1,000 units was 2.930us/20.341us/207.524us p95.
- 해석의 한계: timings are local characterization, not a production SLA; no allocation/RSS, projectile/effect-heavy boss,
  concurrent-session or cross-machine distribution was measured. Synthetic `BattleCore` clone is a lower bound for complete
  `ActiveBattleSession`. The probe demonstrates the O(retained history/state) mechanism, not current user-visible slowness.
- 연결된 map/finding/gap: S09, `PERFORMANCE_BUDGET.md`, ARCH-REVIEW-002, GAP-001

## E-021. Strict checkpoint continuation equivalence probe

- 시각: `2026-08-22T07:30:00+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A10, A06, A08
- 질문/가설: a public SavePoint export/continue restores the same observable state and yields the same next command results as
  uninterrupted execution.
- 정확한 command 또는 read scope: same `/tmp/core_arch_perf` harness as E-020; public `AdminEnterSavePoint`,
  `CompleteNode`, `export_active_run_checkpoint`, `continue_from_active_run_checkpoint`, `SelectMapNode`,
  `ConfirmEnterNode`; JSON/serialized `BehaviorResult` equality.
- 실행 위치: same as E-020.
- 입력/workload: live embedded Standard run, seed 20816, fixed player UUID; export last completed SavePoint to a fresh GameCore
  initialized with a deliberately different constructor seed, then apply the same available-node selection and confirmation.
- 변경한 내용: `/tmp` harness only.
- 실제 결과: checkpoint payload was 17,137 bytes; pre-command run snapshots were byte-equal; node-selection previews and
  node-confirm results were serialized-value equal (`true/true/true`). Code restores checkpoint seed, namespaced UUID manager,
  map/progression and strict validated payload. Existing tests separately cover graph rejection-before-mutation,
  restore-recapture semantic equality and deterministic gate replay.
- 해석의 한계: one bounded noncombat continuation sequence is not a cross-platform physics replay or every future boss/relic/
  corrosion path. It supplies an executable migration oracle seam, not universal equivalence proof.
- 연결된 map/finding/gap: S10, `DETERMINISM_CONTINUATION_MAP.md`, GAP-002

## E-022. A00~A11 independent architecture verdict lock

- 시각: `2026-08-22T07:34:00+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A00~A11
- 질문/가설: 과거 defect 감사에 접근하지 않고 source/domain/aggregate/dependency/scenario/test/operation/data/performance/
  continuation 증거만으로 독립 판정을 내리고 보존 자산과 migration seam을 정할 수 있는가?
- 정확한 command 또는 read scope: E-001~E-021과 이 Goal의 maps; S01~S10; active findings and gaps. Past audit paths were
  not opened/searched before this record.
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: fixed dirty baseline and routed source sections recorded in `DOCUMENT_SOURCE_MAP.md`.
- 변경한 내용: `EVIDENCE_MATRIX.md` 1차 판정, `PRESERVATION_ASSET_REGISTER.md`, finding status/severity and Plan phase.
- 실제 결과: A00 PASS; A01 PARTIAL; A02 FAIL; A03 FAIL; A04 PARTIAL; A05 FAIL; A06 PARTIAL; A07 FAIL; A08 PARTIAL;
  A09 FAIL; A10 PARTIAL; A11 PARTIAL. Cohesive domain engines, data/validation, deterministic identity, strict checkpoint and
  broad tests are preservation assets. The root pressures are application orchestration/state/transaction/protocol coupling and
  unbounded battle-session resource behavior. Existing façade, staged helpers, compiled setup, data compiler, checkpoint and
  test seams make incremental boundary replacement viable.
- 해석의 한계: A12/rewrite conclusion remains locked until the past-audit cross-check. GAP-001/002 isolate product caps and
  cross-platform replay policy without reversing the structural result.
- 연결된 map/finding/gap: all independent maps; PA-001~012, MS-001~008, MB-001~006

## E-023. Past structural audit cross-check and current-code revalidation

- 시각: `2026-08-22T07:38:00+09:00`
- 기준선: `BASELINE-20260822-064427-KST`; independent verdict E-022 was already locked.
- criterion: A00~A11 cross-check prerequisite for A12
- 질문/가설: independent review가 과거 defect audit의 important structural facts를 놓쳤거나 반대로 과대평가했는가?
- 정확한 command 또는 read scope: selected eight past documents and identities listed in `PAST_AUDIT_CROSSCHECK.md`;
  current `data/mod.rs:239-297,481-507,819-879`, `world.rs:65-85,823-826`, `battle/core/mod.rs:68-112,253`,
  `battle/enums.rs:140-168,260-270,270-554`, and StartNewGame trigger ordering.
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: mapping past findings/root causes/gaps to current independent ARCH findings, assets and scenarios.
- 변경한 내용: cross-check document, ARCH-REVIEW-006 and bounded updates to existing findings/maps only.
- 실제 결과: transaction adoption, parallel phase state, protocol concentration, unbounded scanned event history and
  application/protocol staged rewrite all agree. Current code independently reconfirmed a missed public fixture/validated-data
  type conflation, added as ARCH-REVIEW-006. Public battle mutation/float ordering risks were folded into ARCH-REVIEW-003;
  StartNewGame late failure into ARCH-REVIEW-005/006; low-severity setter was not inflated into a root finding. A00~A11 verdicts
  did not change.
- 해석의 한계: no past adjacent-server evidence was imported, no old absolute performance number was combined with E-020,
  and the past draft is not an approved architecture.
- 연결된 map/finding/gap: `PAST_AUDIT_CROSSCHECK.md`, ARCH-REVIEW-003/005/006

## E-024. Architecture option comparison and A12 rewrite decision

- 시각: `2026-08-22T07:43:00+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A12
- 질문/가설: maintain/refactor/shell rewrite/subsystem rewrite/full rewrite 중 어떤 선택이 preservation, verification,
  performance and reversibility risk를 최소화하는가?
- 정확한 command 또는 read scope: locked A00~A11 matrix, S01~S10, ARCH-REVIEW-001~006, PA/MS/MB register,
  E-023 cross-check and GAP-001/002; no new production read required.
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: five options required by criteria and subsystem-by-subsystem disposition.
- 변경한 내용: `ARCHITECTURE_OPTIONS.md`, `REWRITE_DECISION.md`, A12 matrix entry.
- 실제 결과: O1/O2 cannot remove root-enforced transaction/state/protocol/resource pressures; O4 has no evidence for replacing
  cohesive algorithms; O5 fails every mandatory full-rewrite evidence condition. O3 staged application/protocol shell plus
  bounded battle-session/archive and data-construction boundary rewrites preserves one authority and the strongest oracles.
  No subsystem is a `FULL_REWRITE_CANDIDATE`.
- 해석의 한계: relative semantic/change risk is compared without invented schedule estimates; target SLA and cross-platform
  replay guarantees remain GAP-001/002.
- 연결된 map/finding/gap: `ARCHITECTURE_OPTIONS.md`, `REWRITE_DECISION.md`, A12 PASS

## E-025. Final identity, contamination and completion audit

- 시각: `2026-08-22T07:45:04+09:00`
- 기준선: `BASELINE-20260822-064427-KST`
- criterion: A00, A12 and Goal completion conditions
- 질문/가설: all evidence still belongs to the fixed dirty baseline, required outputs are internally complete, and the Goal
  modified no production/test/live-data/canonical contract.
- 정확한 command 또는 read scope: `sha256sum -c BASELINE_MANIFEST.sha256`; exact manifest aggregate reproduction;
  `sha256sum` for Cargo.lock, candidate sources, criteria and DATA-001/002; 25-file adjacent aggregate reproduction;
  `git rev-parse HEAD`; exact `git status --short` diff against baseline; required-output/status/finding/evidence scans.
- 실행 위치: `/mnt/f/work/simulator/core_enhanced`
- 입력/workload: 184 runtime/test files, all actually used policy/contract/future sources, 25 adjacent RON/JSON files,
  Goal documents and the `/tmp` probe identities.
- 변경한 내용: completion documentation and Goal status only.
- 실제 결과: all 184 manifest entries passed; aggregate remained
  `ae1b66e9218ee493bffaa4da3f9b9d7f0bbd8849516d6e2a5f49749be78acb31`; HEAD remained
  `f318414fb0156a67e04c5479000c2a2683fc0c09`; Cargo.lock and all source hashes matched catalog values; adjacent aggregate
  remained `58fa32ef690e4f5e3d2c3062ecb184da99907b3c5c796c77b7e4e3b25cc5d0d0` across 25 files/408,400 bytes;
  current Git status was byte-identical to the 325-line baseline capture. All 22 required outputs exist, A00~A12 have verdicts,
  S01~S10 are inspected, six Active findings satisfy the contract, two open gaps are isolated, and past cross-check sequencing
  is recorded.
- 해석의 한계: broad tests/fmt/clippy were not rerun because production/test bytes matched and the Plan forbids reassurance
  repeats. `/tmp` timing remains a local characterization.
- 연결된 map/finding/gap: `COMPLETION_AUDIT.md`, final A00/A12
