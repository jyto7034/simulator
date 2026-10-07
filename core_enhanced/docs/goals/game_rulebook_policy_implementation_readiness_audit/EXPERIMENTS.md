# Experiments

## 2026-08-12 — Goal Initialization

### Purpose

코드 수정 없는 구현 준비도 조사 Goal을 저장소 가이드와 기존 evidence에 맞게 초기화한다.

### Commands and reads

- git rev-parse --show-toplevel
  - 결과: /mnt/f/work/simulator
- git rev-parse HEAD
  - 결과: f318414fb0156a67e04c5479000c2a2683fc0c09
- git status --short
  - 결과: Core, live data, game_server, canonical과 다수 Goal 경로에 기존 dirty/untracked 변경이 있음
- 생성 예정 경로 존재 여부 검사
  - 결과: game_rulebook_policy_implementation_readiness_audit 경로 충돌 없음
- goal_authoring_and_execution_guide.md를 L1부터 L874까지 읽음
- README.md, engineering_workflow.md와 goal_completion_review_guide.md를 EOF까지 읽음
- 선행 game_rulebook_runtime_alignment_audit/PLAN.md와 MISMATCH_CATALOG.md를 EOF까지 읽음
- 유사 조사 Goal인 employee_domain_implementation_difficulty_audit/PLAN.md와
  boss_runtime_implementation_readiness_audit/PLAN.md를 EOF까지 읽음

### Changes

- PLAN.md, EXPERIMENTS.md와 EXPERIMENT_NOTES.md를 신규 초기화했다.
- Goal은 single, investigation_only, Production Mutation Authorized: no로 설정했다.
- 1~5 난이도를 여섯 evidence 축에서 산정하고 individual finding과 work package를 분리하도록
  계획했다.
- 정책 결정, risk, confidence, migration, dependency와 test-first safety net을 별도 산출물로
  요구했다.

### Validation result

- Runtime 조사, test 또는 probe: 실행하지 않음
- 실행 test 수: 해당 없음
- pass/fail/skip: 해당 없음
- Production 변경: 없음
- 다음 단계: 사용자가 이 Goal의 실행을 지시할 때 baseline과 input revalidation부터 시작

## 2026-08-12 — Goal Document Structure Validation

### Validation

- Goal directory의 top-level file 목록을 검사했다.
- 선행 audit의 PLAN.md, MISMATCH_CATALOG.md와 RULEBOOK_CLAIM_MATRIX.md 링크 대상을 확인했다.
- PLAN.md의 primary input registry에서 GRB-M ID unique count를 검사했다.
- completion matrix의 GRB-IRA-C ID unique count를 검사했다.
- goal_authoring_and_execution_guide.md가 요구하는 단일 Goal 필수 section 19개의 존재를 검사했다.
- 세 Markdown 문서의 trailing whitespace를 검사했다.
- Goal-scoped git status를 확인했다.

### Result

- 필수 Goal 문서: 3개 존재
- input finding primary rows: 15개
- completion evidence rows: 14개
- 필수 PLAN section: 19개
- broken required input path: 0개
- trailing whitespace: 0건
- scoped status: 신규 Goal directory만 untracked
- test/probe pass/fail/skip: 해당 없음
- 판정: STRUCTURE_VALIDATION_PASS

## 2026-08-12 — Safe Implementation Sequence Method Amendment

### Purpose

정책/finding 번호 순서가 아니라 실제 dependency를 기반으로 안전한 구현 순서를 산정하도록 재사용
가이드와 이 Goal을 강화한다.

### Changes

- 재사용 가이드에 typed dependency graph, cycle/SCC disposition, topological layer,
  activation/completion gate, 병렬 실행과 intermediate-state safety 절차를 추가했다.
- IMPLEMENTATION_SEQUENCE.md를 필수 대형 readiness artifact로 추가했다.
- 이 Goal에 Implementation Sequence Audit section과 별도 실행 stage를 추가했다.
- Completion condition과 evidence matrix에 graph coverage, cycle disposition, sequence와 stage safety를
  추가했다.
- README의 준비도 가이드 설명에 dependency graph와 안전한 구현 순서를 반영했다.

### Structural validation

- 재사용 가이드 numbered section: 21개
- dependency edge type: 8개
- readiness gate: 4개
- Goal input finding rows: 15개
- Goal invariants: 14개
- Goal completion evidence rows: 17개
- Investigation stage heading: 0~8, 9개
- 필수/추가 PLAN section 검사: 20개
- IMPLEMENTATION_SEQUENCE.md requirement/reference: 존재
- trailing whitespace: 0건
- test/probe pass/fail/skip: 해당 없음
- 판정: SEQUENCE_GUIDE_STRUCTURE_PASS

## 2026-08-12 — Execution Baseline And Evidence Freeze

### Purpose

조사 시점 working tree, 입력 문서와 접근 가능한 외부 Unity consumer를 고정하고 선행 감사 뒤
generation이 달라졌는지 판정한다.

### Commands and reads

```text
git rev-parse --show-toplevel
git rev-parse HEAD
git status --short
sha256sum core_enhanced/docs/game_rulebook.md \
  core_enhanced/docs/goals/game_rulebook_runtime_alignment_audit/PLAN.md \
  core_enhanced/docs/goals/game_rulebook_runtime_alignment_audit/MISMATCH_CATALOG.md \
  core_enhanced/docs/goals/game_rulebook_runtime_alignment_audit/RULEBOOK_CLAIM_MATRIX.md
git -C '/mnt/f/unity projects/ark' rev-parse HEAD
git -C '/mnt/f/unity projects/ark' status --short
sha256sum '/mnt/f/unity projects/ark/docs/unity_core_contract.md' \
  '/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md'
command -v Unity
command -v unity-editor
command -v Unity.exe
git status --short | sha256sum
git -C '/mnt/f/unity projects/ark' status --short | sha256sum
```

### Result

- Simulator root/commit: `/mnt/f/work/simulator`,
  `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Unity root/commit: `/mnt/f/unity projects/ark`,
  `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`.
- 두 commit은 선행 alignment audit generation과 같다. 양쪽 working tree에는 이번 Goal 시작 전부터
  광범위한 tracked/untracked 사용자 변경이 있다.
- Simulator initial `git status --short` fingerprint:
  `66335ad0811e5045f14a17150c483065d9eab8e8532af641a30291974604509b`.
- Unity initial `git status --short` fingerprint:
  `5c3529fa6ba1617bc53c6b4f765b9f2a221ad7bc8e98d0205a6b1c700efeb663`.
- Input hashes:
  - `game_rulebook.md`: `c11a09402febbb6b16dcc98a11dd1364bac8dfe53645627f1670e6fbd7380f1e`
  - input audit `PLAN.md`: `13709bae9f398daf3b7dd68057c398c54db01b8d22faedac49ba2bd66d3fbd62`
  - `MISMATCH_CATALOG.md`: `ae3c9aa02b7b43d3bd866a0db009a2356223e4ab1a7ac30b11c3625624b88531`
  - `RULEBOOK_CLAIM_MATRIX.md`: `2cdd9d618f15c6de5d0018219482b22b3d50c5ed62984a4a0833ce8819e29cf8`
- Unity canonical hashes:
  - `unity_core_contract.md`: `a3224e244f9577fbb6a4d6d01e44b7ea85f3ba1969f656a22712b0ba972063f5`
  - `core_unity_battle_transport_contract.md`:
    `11609cd0240759784c0b441b3c6bfd6b17219bc71e0f0c672c309f763e072618`
- `Unity`, `unity-editor`, `Unity.exe`는 실행 shell에서 모두 unavailable이다.
- Test/probe 실행: 0개. pass/fail/skip: 해당 없음.
- Production/canonical/Unity mutation: 없음.

## 2026-08-12 — Bounded Unit B: Research Removal And Encounter Fragment Economy

### Hypothesis

`GRB-M-002`, `GRB-M-008`, `GRB-M-009`는 하나의 run/checkpoint/result cutover를 공유하며, ordinary
Reward source는 별도 source-aware validation으로 닫아야 한다.

### Commands and reads

- Core/live data/server/Unity에서 `abnormality_research`, `pending_research_deliveries`,
  `GrantSkillFragmentResearch`, `response_complete_skill_fragment_id`, `suppression_research`,
  `fragment_id`, `repeat_reward`, `default_treasures`, 관련 result diff를 `rg -n`으로 검색.
- `world/state.rs`, `world/active_run.rs`, `skill_fragment.rs`, `abnormality_research.rs`,
  `world/combat.rs`, `world/node_flow.rs`, `reward.rs`, `world/reward.rs`, `data/pve_data.rs`,
  `data/validation.rs`, live run/abnormality/PVE/reward/node RON을 line-level로 읽음.
- `game_server/src/game/player_game_actor/state.rs`의 planning-return mapping test와 Unity
  `ResearchDeliveryNotificationModel.cs`/EditMode contract test를 읽음.
- Exact class count 검색 결과: current PVE definitions는 Normal 3, Elite 13, NormalBoss 12,
  FinalBoss 5; `fragment_id`/`repeat_reward` field 0건.

### Result

- 세 finding 모두 current validity `confirmed`.
- Legacy research는 RunState, inventory, V3 checkpoint/active-run, DTO, server와 Unity까지 live다.
- Encounter-grade acquisition/repeat schema와 ledger는 없고 Monolith Stage 3만 old abnormality field를
  사용하는 partial special case다.
- Ordinary Reward node의 live pool이 direct fragments와 research rewards를 실제 포함한다.
- 상세 boundary와 target acceptance는 `IMPLEMENTATION_BOUNDARY_AUDIT.md` Bounded Unit B에 기록.
- Test/probe 실행: 0개. pass/fail/skip: 해당 없음. Existing test source만 inspection.
- Production/canonical/Unity mutation: 없음.

### Interpretation

HEAD가 아니라 위 working tree를 current evidence로 사용한다. Commit과 접근 경계는 선행 감사와
같지만 input과 production 경로가 dirty하므로 과거 finding 결론을 proof로 복사하지 않고 reachable
path와 live data를 다시 읽는다. Unity runner 부재는 source/기존 assertion inspection의 한계로
기록하며 fresh Unity test 통과를 주장하지 않는다.

## 2026-08-12 — Bounded Unit A: Standard Lifecycle And Global Profile

### Hypothesis

`GRB-M-001`, `GRB-M-003`, `GRB-M-004`의 current code/data/consumer anchor가 baseline working tree에서도
유효하며, 후속 구현은 Standard map authority, legacy completion 제거와 global profile cutover를 함께
고려해야 한다.

### Commands and reads

- `rg -n`으로 `generate_current_floor_map`, `generate_with_requirements`,
  `encounter_class_for_map_node`, `all_response_complete`, `RunComplete`, `StartNewGame`,
  `EncounterCompendiumProfileState`와 Unity mode consumer를 Core/test/live data/server/Unity에서 검색.
- `nl -ba ... | sed -n ...`으로 다음 current source를 직접 읽음:
  - `core_enhanced/src/game/world.rs`
  - `core_enhanced/src/game/world/node_flow.rs`
  - `core_enhanced/src/game/map/progression.rs`
  - `core_enhanced/src/game/map/facility_generator.rs`
  - `core_enhanced/src/game/world/map_encounters.rs`
  - `core_enhanced/src/game/world/combat.rs`
  - `core_enhanced/src/game/encounter_compendium.rs`
  - `game_resources/data/map/node_assignment_profiles.ron`
  - `game_server/src/persistence/encounter_compendium_store.rs`
  - Unity `GameCommandBus.cs`, `RunFlowScreenComposer.cs`, `ArkClientContractTests.cs`

### Result

- 세 finding 모두 current validity `confirmed`.
- Standard는 shared generator/adaptive power와 FinalBoss를 계속 사용한다.
- Endless research completion은 reachable `RunComplete` transition이다.
- 영구 profile은 compendium 네 set만 가지며 Core/Unity에 tutorial gate가 없다.
- 상세 boundary와 target acceptance는 `IMPLEMENTATION_BOUNDARY_AUDIT.md`의 Bounded Unit A에 기록.
- Test/probe 실행: 0개. pass/fail/skip: 해당 없음. Existing test source만 inspection.
- Production/canonical/Unity mutation: 없음.

## 2026-08-12 — Bounded Unit C: Elite Pool, Monolith And Sweep

### Hypothesis

`GRB-M-005`, `006`, `007`, `014`는 current Monolith/Elite state와 combat-result seam을 공유하지만,
stat 배율 correction만 wire/save 변화 없이 독립 구현 가능하고 나머지는 encounter ledger cutover에
합쳐야 한다.

### Commands and reads

- `monolith.rs`, `world/map_encounters.rs`, `world/combat.rs`, `data/run_policy_data.rs`와 live
  `run/policy.ron`에서 natural exclusion, repeat weighting, remanifestation begin/victory/failure,
  stage multiplier 적용·validation을 line-level로 읽음.
- Core/server/live data/Unity production source에서 gameplay `Sweep|sweep|소탕`을 exact 검색하고,
  `behavior.rs`의 ActionKind/PlayerBehavior/BehaviorResult 및 combat staged economy/XP/result path를 읽음.
- 관련 current unit/map selection tests를 inspection함.

### Result

- 네 finding 모두 current validity `confirmed`.
- Current Elite는 첫 승리/실패 뒤 자연 pool에서 영구 제외되고 legacy research/recent weighting을 쓴다.
- Current live Monolith stat 배율은 Stage 2 100%, Stage 3 125%이며 approved initial target 150%/200%와
  safe tuning invariant validation이 없다.
- Gameplay sweep command/result/consumer는 0건이고 battle-independent synthetic transaction이 없다.
- Remanifestation exhausted failure는 active node만 해제하여 use가 남으면 같은 stage를 다시 만들 수 있다.
- 상세 boundary와 target acceptance는 `IMPLEMENTATION_BOUNDARY_AUDIT.md` Bounded Unit C에 기록.
- Test/probe 실행: 0개. pass/fail/skip: 해당 없음. Existing test source만 inspection.
- Production/canonical/Unity mutation: 없음.

## 2026-08-12 — Bounded Unit D: NormalBoss And FinalBoss/BossOmen Cycles

### Hypothesis

`GRB-M-010`~`013`은 현재 BossOmen state가 이미 가진 partial chain 경계를 재사용할 수 있지만,
NormalBoss/FinalBoss cycle과 outcome-aware source resolution을 하나의 replacement encounter ledger로
도입해야 안전하다.

### Commands and reads

- `boss_omen.rs`, `data/boss_omen_data.rs`, live `boss_omen/chains.ron`, abnormality omen reference validation,
  `world/map_encounters.rs`, `world/node_flow.rs`, `world/combat.rs`, `world/event_node.rs`를 line-level로 읽음.
- Current NormalBoss/FinalBoss live class 수, cycle/condition/completion field와 Core/server/Unity omen consumer를
  `rg -n`으로 검색.
- Event check failure completion, BossOmen overlay/capstone, forced-boss defeat/victory tests를 inspection함.

### Result

- 네 finding 모두 current validity `confirmed`.
- NormalBoss는 12개 live identity를 가진 반면 cycle ledger 없이 generic deterministic selection을 쓴다.
- BossOmen chain schema는 per-chain fragment/awakening condition이 없고 runtime은 any-awakened-fragment gate다.
- FinalBoss victory는 active state를 clear하지만 completed chain/cycle/reward ledger를 남기지 않는다.
- Event failure와 exhausted Combat failure도 ordinary node completion을 통해 omen source를 성공 처리한다.
- 상세 boundary와 target acceptance는 `IMPLEMENTATION_BOUNDARY_AUDIT.md` Bounded Unit D에 기록.
- Test/probe 실행: 0개. pass/fail/skip: 해당 없음. Existing test source만 inspection.
- Production/canonical/Unity mutation: 없음.

## 2026-08-12 — Bounded Unit E: Skill Fragment Placeholder Conversion

### Hypothesis

`GRB-M-015`는 active-skill framework 부재보다 29개 개별 content design/publication이 readiness를 막는
항목이며 하나의 generic runtime patch로 닫을 수 없다.

### Commands and reads

- Production skill-fragment RON의 `BasicAttackModifier`/`ActiveSkill` exact count와 placeholder identity를
  검색하고 top/bottom entries를 line-level로 읽음.
- `data/skill_fragment_data.rs`, cross-catalog validation, `skill_fragment.rs` equip/profile application과
  active-skill tests를 읽음.
- Core battle targeting/activation 및 접근 가능한 Unity skill-fragment/active-skill consumer를 검색.

### Result

- Current production count는 BasicAttackModifier 29, ActiveSkill 2다.
- ActiveSkill schema, one-slot equip, upgrade/awakening skill selection, validation, battle/Unity consumer
  foundation은 이미 존재한다.
- 29개 dedicated mechanics/compatibility/presentation/balance design은 canonical에 없고 룰북도 이번
  correction에서 live conversion을 제외한다.
- 이 unit 시점의 initial disposition은 `clarification_required`였으나, 뒤의 durable decision-record
  reconciliation에서 룰북의 explicit current-wave exclusion을 적용해 `future_or_dormant`로 supersede했다.
- 상세 boundary는 `IMPLEMENTATION_BOUNDARY_AUDIT.md` Bounded Unit E에 기록.
- Test/probe 실행: 0개. pass/fail/skip: 해당 없음. Existing test source만 inspection.
- Production/canonical/Unity mutation: 없음.

## 2026-08-12 — Focused Structure, Graph And Completion Validation

### Hypothesis

Readiness artifacts가 15개 finding, 10개 decision과 5개 package를 정확히 연결하고, 점수/graph/무변경
주장을 재현 가능하게 검산할 수 있으며 open decision을 `Complete` 뒤에 숨기지 않는다.

### Commands and results

- `rg -c`와 expected-ID `comm` 검사:
  - assessment primary rows 15, detailed sections 15, boundary rows 15
  - decision sections 10, work-package sections 5, sequence stages 5
  - M001~M015 primary/detail missing/extra 0
- Markdown table `awk -F'|'` 재계산:
  - fixed individual scores 14/14 declared sum 일치
  - base difficulty 14/14 일치, M014 sum 9의 save-migration minimum override 4 확인
  - M015는 numeric 확정값으로 강제하지 않고 8–17 / difficulty 3–5 range로 유지
- Work-package exact coverage table의 GRB-M ID를 추출해 `sort | uniq -c`: 15개 모두 count 1.
- PD001~007의 decision section↔assessment↔work-package 양방향 link check: 7/7 exit 0.
- WP001~005의 assessment↔package section↔sequence three-way check: 5/5 exit 0.
- Directed edges를 `tsort`: exit 0; package-level multi-node cycle 0. Standalone WP002는 node registry에서
  별도 exact coverage.
- Source anchor existence: Core/data/server/Unity 20개 파일 checked, 20 pass, 0 fail.
- Required artifact/headings/trailing whitespace: required 9개 artifact 존재, trailing whitespace 0.
- Completion guide가 함께 확인하도록 지정한 `docs/README.md` 159줄과 `engineering_workflow.md` 407줄을
  EOF까지 읽고 source-of-truth, external Unity canonical, no fallback/dual schema, change-type validation과
  user-decision gate가 proposed package/sequence와 일치함을 확인.
- Initial/final integrity:
  - simulator status hash initial/final
    `66335ad0811e5045f14a17150c483065d9eab8e8532af641a30291974604509b` / same
  - Unity status hash initial/final
    `5c3529fa6ba1617bc53c6b4f765b9f2a221ad7bc8e98d0205a6b1c700efeb663` / same
  - rulebook, three input audit artifacts와 두 Unity canonical hash 6/6 unchanged.

### Result and interpretation

- Completion evidence C01~C17은 investigation deliverable 관점에서 모두 pass.
- Runtime test/probe 실행: 0개; pass/fail/skip 0/0/0. Production diff가 없고 static trace로 current fact가
  닫혀 full suite는 PLAN의 non-required omitted validation이다.
- Unity runner unavailable을 fresh pass로 계산하지 않았다.
- Independent primary verdict는 `Policy decision needed`, long-term fit `High`.
- 이 validation 시점에는 `GRB-PD-001`~`007`을 모두 open으로 분류했으나 뒤의 durable-record
  reconciliation이 4개를 close했다. 남은 3개 때문에 Status는 계속 `policy_decision_required`다.
- Production/canonical/server/Unity mutation: 없음.

## 2026-08-12 — Durable Decision Record Reconciliation

### Hypothesis

최초 decision ledger가 이미 확정된 save/profile lifetime, implementation-owned identity schema와 룰북의
명시적 future scope까지 사용자 결정으로 과대 분류했을 수 있다. Durable canonical과 live membership을
다시 대조하면 진짜 content input만 남길 수 있다.

### Commands and reads

- Attachment와 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`를 상태 gate 전에 다시 EOF까지 읽음.
- SavePoint/active-run, local compendium, facility map, Elite Monolith의 완료된 decision record를 EOF까지
  읽고 현재 `data_loading_contract.md`, `core_runtime_contract.md`와 룰북 target 절을 대조함.
- Skill fragment system/index와 룰북 948~963행을 대조해 29개 placeholder의 current-wave scope를 확인함.
- Live PVE 30개 eligible encounter의 encounter class/ID/primary abnormality를 추출하고 catalog reference를
  비교함. Facility Standard pool/template/profile과 BossOmen live chain도 EOF까지 읽음.

### Result

- `GRB-PD-001`은 exact V3 rejection/no migration canonical로 Option A `closed_by_canonical`.
- `GRB-PD-002`는 committed compendium set의 monotonic preservation/confirmed-reset-only lifetime 때문에
  bounded preservation Option A `closed_by_canonical`.
- `GRB-PD-003`은 stable authored source key를 strict schema detail로 선택해 `closed_by_audit`. Final
  candidate validation에서 live 30개 encounter ID와 primary abnormality ID가 각각 30/30 unique이고
  `suppress_king_of_greed` reference도 correct임을 확인했다. Explicit key는 current collision fix가 아니라
  future variants/multiple fragments와 long-lived reward identity를 content layout에서 분리하는 선택이다.
- `GRB-PD-007`은 룰북이 이번 correction의 live conversion을 명시적으로 제외하므로 Option A
  `closed_by_canonical`; M015/WP005는 `future_or_dormant`.
- 남은 material input은 `GRB-PD-004` exact fragment/repeat manifest, `GRB-PD-005` live BossOmen chain,
  `GRB-PD-006` fixed Standard slot/content/tuning manifest 세 개다.
- Existing Standard topology는 각 pool에 template 하나씩 있어 재사용 후보가 명확하지만 node content,
  F3 terminal class, exact encounter/tuning/script는 아직 fixed authoring이 아니다.
- 검산 중 사용자 소유 `docs/skills/lobotomy_content_catalog.md`가 동시 변경됐고, Moonlit Wail을 boss
  후보에서 제외하는 current content decision을 확인했다. 이를 반영한 target roster candidate는 Elite 13,
  NormalBoss 13, FinalBoss 3 = 29 identities이며 live 13/12/5와 구분한다.
- Runtime test/probe 실행: 0개; pass/fail/skip 0/0/0. 이번 unit도 policy-independent static reconciliation이다.
- Production/canonical/server/Unity mutation: 없음. Goal artifact만 갱신함.
- Simulator status hash는 initial
  `66335ad0811e5045f14a17150c483065d9eab8e8532af641a30291974604509b`에서
  `d9d030a77d67cf93a52b5e401df34ff17a87dce228eb44ab3b177edafd402e1c`으로 바뀌었지만 final stream에서
  catalog 한 줄만 제거하면 initial hash가 exact 재현된다. 이 외부 동시 변경은 보존했고 agent write
  scope에는 없다.
- Revalidation: artifacts 9; assessment/boundary 15/15/15; decisions 7 = canonical-closed 3 + audit-closed 1 +
  content-required 3; packages/current stages/future nodes 5/4/1; fixed scores 14/14; package finding coverage
  15 exact; trailing whitespace 0. Unity status hash는 baseline과 동일하다.

## 2026-08-12 — Content Input Candidate Reduction

### Hypothesis

Open content input을 승인 없이 결정하지 않더라도 current catalogs와 durable future boundaries에서 exact
fragment rows, BossOmen dormancy, fixed Standard의 bounded approval candidate까지 유도할 수 있다.

### Commands and reads

- Attachment와 이 Goal의 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`를 코드 검색 전에 다시 EOF까지
  읽음.
- PVE encounter, abnormality/skill-fragment catalog와 target roster에서 Elite 13 + NormalBoss 13 +
  FinalBoss 3의 exact encounter/primary/fragment tuple을 read-only `awk`/`rg`로 추출함.
- `run/policy.ron`의 current repeat dust 5/10/20과 runtime `GrantFragmentDust` support를 대조함.
- 완료된 `boss_omen_chain`, `boss_omen_phase5_completion_hardening`, placeholder content canonicalization
  기록과 current `boss_omen/chains.ron`, rulebook, final-boss skill design notes를 읽음.
- Standard facility templates/profiles, run difficulty policy, PVE encounters, node/event/reward/shop/VN RON,
  starter employee loadouts와 두 active fragment compatibility를 대조함.

### Result

- `CONTENT_INPUT_CANDIDATES.md`에 29개 exact source→encounter→corrected primary→fragment table을 기록했다.
  PD004의 unbounded fragment identity는 닫혔고 사용자 입력은 explicit dust 5/10/20, `None`, 또는 cutover
  defer로 줄었다.
- King of Greed exact existing primary reference validation, Moonlit Wail exclusion, Queen/Mountain NormalBoss
  reclassification을 candidate validation에 포함했다.
- Rulebook이 detailed WhiteNight chain과 additional final-boss authoring을 future로 둔다. Apocalypse Bird
  역시 exact source/event/tuning/reward가 없다. 이후 사용자 최종 검토에서 PD005는 current one-step
  WhiteNight를 generic engine/all-current-FinalBoss record용 sole live development bootstrap으로 유지하는
  `closed_by_user`로 supersede됐다. 이 fixture는 final WhiteNight content가 아니다.
- `ST1`은 existing three topology의 모든 slot/content를 고정하고 F1 Normal, F2 Porcubbus Elite, F3 Funeral
  Butterfly NormalBoss, neutral 100% non-adaptive tuning, Standard-only non-research rewards와
  `scripted_fragment: None`을 제안한다. 두 active fragment는 Ranged Bow/Gun only이고 모든 starter가 Melee
  Sword라 fragment-only tutorial 지급은 compatibility 위반이다.
- PD006은 `ST1` 승인/수정 또는 whole-package defer로 bounded됐지만 content/balance approval은 남는다.
- Runtime test/probe 실행: 0개; pass/fail/skip 0/0/0. Production/canonical/server/Unity mutation은 없고 Goal
  artifact만 갱신했다.
- Candidate 검산 중 같은 user-owned content catalog가 다시 갱신되어 King of Greed design note가
  확장됐다. Latest hash/mtime는
  `67a3207ff22de2e2afc81414809bb1bcc7a8e5f68ae60c2e21b6a4472b394348` /
  2026-08-12 14:53:15 +0900이며 target membership에는 변화가 없다. 파일은 수정하지 않고 보존했다.
- Superseding status: open candidate 2, closed candidate 5, overall `policy_decision_required` 유지.

### Final revalidation

- Goal Markdown 10개 = required 9 + supplemental candidate 1.
- Assessment/detail/boundary finding counts 15/15/15; decisions 7 = canonical-closed 4 + audit-closed 1 +
  content-required 2; packages/current stages 5/4.
- Candidate rows Elite/NormalBoss/FinalBoss 13/13/3, duplicate source/encounter/primary/fragment 0. Encounter,
  abnormality와 fragment catalog missing reference도 각각 0이며 current encounter-primary와
  fragment-origin pair가 29/29 일치한다.
- Trailing whitespace 0. Simulator status hash는
  `d9d030a77d67cf93a52b5e401df34ff17a87dce228eb44ab3b177edafd402e1c`; content catalog status line을
  제거하면 initial `66335ad0811e5045f14a17150c483065d9eab8e8532af641a30291974604509b`를 exact 재현한다.
  Unity status와 protected six hashes도 baseline과 동일하다.

## 2026-08-12 — User Resolution And Final Readiness Closure

### Hypothesis

사용자의 PD004 R1 승인과 PD006 수정 승인을 exact implementation input으로 기록하면 남은 material gate가
zero가 되고, production을 변경하지 않은 채 investigation Goal을 `complete`로 닫을 수 있다.

### Commands and reads

- Attachment와 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`를 이번 decision turn의 첫 search/action
  전에 다시 EOF까지 읽었다.
- User message를 PD004 `R1-continuity` 승인과 “나머지는 제안대로, F3 boss만 reduced Nothing There”인
  modified ST1 승인으로 해석했다.
- Nothing There base HP/ATK/DEF/MR `1200/88/60/55`에 60%와 existing integer percentage convention을
  적용해 boss-only exact target `720/53/36/33`을 계산했다. 이는 기존 ST1의 F3 후보 envelope와 유사하면서
  Nothing There identity를 보존하는 bounded reduction이다. Current generic `BattleUnitStatScale`은
  HP/ATK/DEF 세 축뿐이고 encounter-wide이므로 approved target은 F3 fixed slot의 explicit four-axis actor
  override로 기록했다.
- Goal artifact 10개에 decision status, finding readiness, package/sequence gates와 completion verdict를
  일관되게 전파했다. Production/canonical/server/Unity 파일은 수정하지 않았다.

### Result and interpretation

- `GRB-PD-004`: `closed_by_user`, exact 29-row manifest + repeat dust Elite 5/NormalBoss 10/FinalBoss 20.
- `GRB-PD-006`: `closed_by_user`, `ST1-NT60`. Existing ST1 topology/slot/content를 유지하고 F2 Porcubbus,
  F3 Nothing There를 사용한다. Exact four-axis override는 F3 boss actor에만 적용하며 other
  actors/nodes/rewards와 other Nothing There appearances는 100%, adaptive scaling과 scripted fragment reward는 없다.
- 당시 decision count는 7 total이었고, 이후 final policy review에서 PD008~010과 PD005 disposition 변경이
  반영되어 current count는 10 total = canonical 3 + audit 1 + user 6; open/content-input 0이다.
- Current package order는 `WP002 → WP001 → WP003 → WP004`, WP005는 future/dormant로 유지한다.
- Overall verdict는 `ready_for_implementation`, investigation status는 `complete`다. 이는 별도 승인 없는
  production mutation을 허용하지 않으며 implementation complete claim도 아니다.
- Runtime test/probe 실행은 0개다. 이번 pass는 read-only decision/artifact closure이고 target behavior는 아직
  구현되지 않았으므로 package-specific test와 balance/Unity runner validation은 future implementation
  completion gate에 남는다.
- Final static validator at this closure pass: Goal Markdown 10, assessment/detail/boundary finding 15/15/15,
  decisions 7 all closed. This historical count is superseded by the later 10-decision final-policy review;
  packages/current stages/future nodes 5/4/1, stale matrix gates 0, trailing
  whitespace 0. Candidate rows Elite/NormalBoss/FinalBoss 13/13/3, four identity columns duplicate 0, package
  coverage 15/15 exact. Simulator status hash는 이전 audit-final
  `d9d030a77d67cf93a52b5e401df34ff17a87dce228eb44ab3b177edafd402e1c`와 동일하다.

## 2026-08-12 — Bootstrap-Only Content Authority Clarification

### User clarification

사용자는 실제/최종 콘텐츠를 이 readiness audit에서 정하지 않으며, 앞서 선택한 값은 게임 구동을 위한
최소 콘텐츠인지 확인한 뒤 문서 반영을 요청했다.

### Superseding interpretation

- PD004의 29 rows와 dust 5/10/20은 acquisition schema, strict loader와 first/repeat transaction을 빈 참조
  없이 실행·검증하기 위한 minimum bootstrap fixture다. Final source roster, pairing과 balance 승인이 아니다.
- PD006 `ST1-NT60`은 fixed Standard generator, no-adaptive path와 F3 completion/profile transaction을
  end-to-end 검증하는 minimum bootstrap fixture다. Final tutorial layout/encounter/tuning/reward/script/
  narrative/assets 승인이 아니다.
- Current one-step WhiteNight만 live development bootstrap이고 detailed/final BossOmen chains와 dedicated
  fragment skills/WP005는 future이므로 실제 final content authoring은 포함되지 않는다.
- Separate content Goals may replace bootstrap concrete values without reopening this readiness audit when they
  preserve system invariants and strict validation. Changing first/repeat semantics, Standard fixed three-Floor
  lifecycle or terminal/profile atomicity remains a policy change.

### Artifact result

PLAN, decision log, bootstrap manifest, boundary/assessment, package map, sequence and completion audit now label
PD004/006 values as `minimum bootstrap`, not final content authority. Readiness remains
`ready_for_implementation`; `Open Bootstrap Inputs: none`, `Final Content Authority:
deferred_to_separate_content_goals`. Production files remain untouched.

## 2026-08-12 — Final Policy Review Reconciliation

### User resolutions

- Development-only project이므로 placeholder content에 별도 public-release gate를 두지 않는다.
- 29 BasicAttackModifier fragments는 intentional placeholders이며 dedicated skill conversion은 future다.
- Elite slots는 distinct unmastered candidates를 먼저 배정하고 부족분만 mastered repeat pool로 채운다.
- Monolith Stage 2/3 stat multiplier initial values는 150%/200%이며 development RON tuning을 허용한다.
- Current one-step WhiteNight는 generic BossOmen/all-current-FinalBoss record를 위한 sole live development
  bootstrap이다. Detailed final chain content는 future다.
- Mastered Elite sweep에서 source fragment count가 0이면 repeat reward 대신 one copy를 재획득하고 count가
  1 이상이면 repeat reward를 지급한다. Dismantle→reacquire cycle은 제한 없이 허용하며 identity upgrade/
  awakening progress를 보존한다.
- Full authored base reward, alive-roster XP, active FinalBoss non-abandonment, strict V3 rejection, checkpoint-load
  deathless-record non-rollback과 easy F3 Nothing There damage-meter role은 모두 intentional이다.

### Current result

- Decision count는 10 total = `closed_by_canonical` 3 + `closed_by_audit` 1 + `closed_by_user` 6; open 0이다.
- Canonical `docs/game_rulebook.md`와 readiness artifacts를 위 결정에 맞춰 갱신했다.
- Production code/RON/server/Unity files는 수정하지 않았고 runtime behavior는 아직 implementation-pending이다.
