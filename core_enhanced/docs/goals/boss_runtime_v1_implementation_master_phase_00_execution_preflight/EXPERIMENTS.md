# Boss Runtime v1 Phase 00 — Experiments

Goal: `boss_runtime_v1_implementation_master_phase_00_execution_preflight`  
Execution Mode: `investigation_only`  
Production Mutation Authorized: no

## 2026-08-09 — Phase initialization and activation

### Hypothesis

The completed readiness audit and active Master authorize a read-only P00, and the current path/status gate should allow
initializing exactly one active child without production investigation or mutation.

### Exact commands

```text
wc -l /home/acr0tic/.codex/attachments/740e5d2a-01fd-4751-a956-eaaee449b1b3/pasted-text-1.txt
sed -n '1,2000p' /home/acr0tic/.codex/attachments/740e5d2a-01fd-4751-a956-eaaee449b1b3/pasted-text-1.txt
wc -l core_enhanced/docs/goals/boss_runtime_v1_implementation_master/PLAN.md core_enhanced/docs/goals/boss_runtime_v1_implementation_master/EXPERIMENTS.md core_enhanced/docs/goals/boss_runtime_v1_implementation_master/EXPERIMENT_NOTES.md
sed -n '1,350p' core_enhanced/docs/goals/boss_runtime_v1_implementation_master/PLAN.md
sed -n '351,694p' core_enhanced/docs/goals/boss_runtime_v1_implementation_master/PLAN.md
sed -n '570,694p' core_enhanced/docs/goals/boss_runtime_v1_implementation_master/PLAN.md
sed -n '1,239p' core_enhanced/docs/goals/boss_runtime_v1_implementation_master/EXPERIMENTS.md
sed -n '1,121p' core_enhanced/docs/goals/boss_runtime_v1_implementation_master/EXPERIMENT_NOTES.md
wc -l core_enhanced/docs/README.md core_enhanced/docs/engineering_workflow.md core_enhanced/docs/goal_authoring_and_execution_guide.md core_enhanced/docs/goal_completion_review_guide.md core_enhanced/docs/game_rulebook.md core_enhanced/docs/core_runtime_contract.md core_enhanced/docs/data_loading_contract.md core_enhanced/docs/skill_target_contract.md core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/PLAN.md core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/BOUNDARY_AUDIT.md core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/POLICY_COVERAGE_MATRIX.md core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/IMPLEMENTATION_READINESS_REPORT.md core_enhanced/docs/goals/boss_runtime_implementation_readiness_audit/COMPLETION_AUDIT.md core_enhanced/docs/tmp/boss_behavior_state_machine_policy_notes_2026-08-07.ko.md core_enhanced/docs/goals/core_enhanced_refactoring_audit_master/PLAN.md core_enhanced/docs/goals/core_enhanced_refactoring_audit_final_catalog_consolidation/FINAL_CANDIDATE_CATALOG.md
sed -n '<recorded ranges>' <required workflow/canonical/dependency files>
rg -n '^### BP-' core_enhanced/docs/tmp/boss_behavior_state_machine_policy_notes_2026-08-07.ko.md
rg --files -g 'AGENTS.md' -g '!target' -g '!Library'
test ! -e core_enhanced/docs/goals/boss_runtime_v1_implementation_master_phase_00_execution_preflight
git rev-parse HEAD
git status --short --untracked-files=all | wc -l
git status --short --untracked-files=all | sha256sum
git status --short --untracked-files=all -- core_enhanced/docs/goals/boss_runtime_v1_implementation_master core_enhanced/docs/goals/boss_runtime_v1_implementation_master_phase_00_execution_preflight
apply_patch  # create the P00 required documents and activate only P00 in the Master
```

`sed -n '<recorded ranges>'` summarizes the exact range commands used above this entry; no secret-bearing command or
runtime runner was used.

### Actual result

- The attachment was read to EOF (198 lines) before execution continued.
- The existing Master required documents were read to EOF: PLAN 694, EXPERIMENTS 239 and EXPERIMENT_NOTES 121 lines.
- Required workflow, completion, canonical, target-policy, readiness and refactor evidence was read before current
  source investigation.
- Master status was `active`, with no active Phase and P00 the only ready candidate.
- No applicable `AGENTS.md` was found and the P00 path did not exist.
- Simulator commit is `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Pre-creation full status is 221 entries with SHA-256
  `41f02b5b73145453f032c3dfcf2931f1225db3225cb70a7ccaf82de5e882bd8d`.

### Changes, failures and decision

- Created only P00 `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md` and updated the Master activation records.
- No production, live data, test, canonical, server or Unity file was modified.
- No command or patch failed and nothing was reverted.
- No test runner was executed, so no pass/fail/skip count is reported.
- P00 is active and may begin read-only current-tree investigation. P01 remains pending.

## 2026-08-09 — Oversized first inventory attempt

### Hypothesis

A single combined read-only command could inventory active Goal statuses, Master phase registries, relevant dirty paths
and changed-file names without losing evidence.

### Exact command shape

```text
rg -n '<active/status patterns>' core_enhanced/docs/goals/*/PLAN.md
rg -n '<active phase/phase table patterns>' core_enhanced/docs/goals/*master*/PLAN.md
git status --short --untracked-files=all -- <boss/runtime/refactor/server/data/Unity relevant paths>
git diff --name-only -- <same relevant paths>
```

This was submitted as one compound read-only inspection. The terminal transcript did not preserve the expanded
pattern/path argument list because the aggregate result exceeded the tool-output budget.

### Actual result

- The command returned more output than the available result window and the whole useful result was truncated.
- No status, ownership, collision or drift conclusion is accepted from this attempt.
- No file was changed and no test runner was executed.

### Failure and next action

This evidence unit failed due to excessive output scope. Replace it with bounded commands and record each usable
result immediately: first active Goal identities, then relevant dirty-path ownership, then source drift by subsystem.

## 2026-08-09 — Active Goal inventory and invalid cross-repository status query

### Exact commands

```text
rg -l '^Status: (active|policy_decision_required|scope_decision_required|externally_blocked|correction_required)$' core_enhanced/docs/goals/*/PLAN.md
rg --files core_enhanced/docs/goals | rg 'defense_route_tactical_command|boss_runtime_v1_implementation_master' | sort
rg -n '^(Status|Active Phase|Implementation Complete|Confidence|Completion Gate|Current Phase|Next Phase Candidate):' core_enhanced/docs/goals/defense_route_tactical_command_master/PLAN.md core_enhanced/docs/goals/defense_route_tactical_command_master/COMPLETION_AUDIT.md core_enhanced/docs/goals/defense_route_tactical_command_phase_*/PLAN.md core_enhanced/docs/goals/defense_route_tactical_command_phase_*/COMPLETION_AUDIT.md
git status --short --untracked-files=all -- core_enhanced/src core_enhanced/data core_enhanced/tests game_server/src game_server/data game_server/tests mock_client '/mnt/f/unity projects/ark/Assets'
```

### Actual result

- Exact-status search returned only the Boss Runtime Master and its P00 child as active/nonterminal Goals.
- Defense Route Tactical Command Master reports `Status: complete`, `Implementation Complete: true` and
  `Current Phase: none — all phases complete`; its phase plans found by the query also report complete/true, and the
  queried completion audits report `Complete / High`.
- The combined `git status` command failed with exit code 128 because Unity is outside the simulator Git repository.
  It produced no usable simulator dirty-path list, so the query must be split by repository/root.
- No file was changed and no test runner was executed.

## 2026-08-09 — Bounded simulator and Unity collision inventory

### Exact commands

```text
git status --short --untracked-files=all -- core_enhanced/src core_enhanced/data core_enhanced/tests game_server/src game_server/data game_server/tests mock_client
find core_enhanced -maxdepth 2 -type d | sort | sed -n '1,160p'
find game_server -maxdepth 2 -type d | sort | sed -n '1,120p'
git rev-parse --show-toplevel
git -C core_enhanced rev-parse --show-toplevel
git -C game_server rev-parse --show-toplevel
git status --short --untracked-files=all | sed -n '1,260p'
git -C '/mnt/f/unity projects/ark' rev-parse --show-toplevel
git -C '/mnt/f/unity projects/ark' status --short --untracked-files=all -- Assets | sed -n '1,220p'
sed -n '<focused ranges>' core_enhanced/docs/goals/defense_route_tactical_command_master/PLAN.md
```

### Actual result

- Simulator/Core/game_server are one Git worktree rooted at `/mnt/f/work/simulator`; Unity is a separate repository
  rooted at `/mnt/f/unity projects/ark`.
- All three P01 G0 production owners are already dirty: `battle/core/mod.rs`, `battle/enums.rs` and
  `world/combat.rs`; relevant world test files are dirty as well.
- P02~P05 data/setup/movement/target paths and P06 server producer paths are also dirty. Four tactical runtime
  modules are untracked.
- Unity has extensive pre-existing dirty work, including the relevant wire/state/store classes and contract/live-flow
  tests. P00 did not modify or validate them.
- The completed Defense Route Master explicitly owns the current Core/live-data/server vertical slice and excludes
  Unity; its final registry reports all Phase 0~9 and Master audits Complete/High.
- Created `ACTIVE_WORK_COLLISION_AUDIT.md` immediately from this bounded evidence. No test runner was executed.

### Decision

There is no second active production Goal, but there is direct user-owned path overlap across every future Boss Phase.
Treat current files as the baseline, patch at symbol/hunk granularity only, and re-run scoped collision checks at each
Phase activation. P01 is not blocked solely by the dirty tree.

## 2026-08-09 — Current P01 G0 source revalidation

### Exact commands

```text
rg -n 'enum BattleEvent|impl Ord for BattleEvent|impl PartialOrd for BattleEvent|BattleEnd|BattleResult|Terminal|MaxTime|time_limit|until|step|advance' core_enhanced/src/game/battle/enums.rs core_enhanced/src/game/battle/core/mod.rs core_enhanced/src/game/world/combat.rs | sed -n '1,320p'
sed -n '1,900p' core_enhanced/src/game/battle/enums.rs
rg -n 'pub fn (start_battle_execution|step_battle_execution|step_battle_execution_until|step_battle_execution_by)|fn (finish|final|terminal|max_time)|BattleExecutionStep|BattleExecutionStatus|BattleStep' core_enhanced/src/game/battle/core -g '*.rs' | sed -n '1,320p'
sed -n '<execution and terminal ranges>' core_enhanced/src/game/battle/core/sim.rs
sed -n '1180,1505p' core_enhanced/src/game/world/combat.rs
rg -n 'fn battle_update_dto_after|ensure_battle_projection_commit_available|commit_battle_projection|reconcile_employee_attempt|after_seq|revision' core_enhanced/src/game/world/*.rs | sed -n '1,360p'
sed -n '<active-session and projection ranges>' core_enhanced/src/game/world/state.rs core_enhanced/src/game/world/combat.rs
rg -n 'CRA-P06-C0[123]|total order|terminal|atomic' <readiness and refactor Phase 06 artifacts>
sed -n '<C01-C03 and test-audit ranges>' <readiness and refactor Phase 06 artifacts>
```

### Actual result

- C01 remains reproducible by source inspection: elapsed/until stepping can reach the configured cap and return
  `Running`, while direct next-event stepping returns terminal Draw for an event beyond the cap.
- C02 remains reproducible: unequal current `BattleEvent` values can compare `Equal`; the catch-all and incomplete
  variant/payload keys remain, and current unit coverage is representative rather than totality/permutation coverage.
- C03 remains reproducible: both world advance entry points mutate the active Core before later fallible trigger,
  reconcile, projection, cursor and transition work; future cursor validation is still after revision commit.
- `BattleCore`/`ActiveBattleSession` have no existing full clone seam that by itself closes C03.
- Created `CURRENT_TREE_DRIFT_AUDIT.md` and recorded these findings before moving to other subsystems.
- No test runner was executed in this unit.

### Decision

No material drift supersedes the G0 prerequisite set. Keep C01 → C02 → C03 as the provisional P01 milestone order,
with C01 state-owned terminal authority feeding both later acceptance suites.

## 2026-08-09 — Data/setup, leaf runtime and producer-consumer drift

### Exact bounded commands

```text
sed -n '<source/parsed/validation ranges>' core_enhanced/src/game/data/live_ron.rs
sed -n '<load/publish ranges>' core_enhanced/src/game/data/mod.rs
sed -n '<PveEncounter ranges>' core_enhanced/src/game/data/pve_data.rs
sed -n '<build/setup ranges>' core_enhanced/src/game/events/combat.rs core_enhanced/src/game/world/combat.rs
sed -n '1,180p' core_enhanced/src/game/combat_setup/enemy_spawns.rs
rg --files core_enhanced/src game_resources/data | rg -i 'boss.*runtime|runtime.*boss|boss_behavior|boss_pattern' || true
rg -n 'boss_runtime|BossEncounterRuntime|BossActorBehaviorRuntime|TrackAndRejoin|RejoinForwardRoute' core_enhanced/src game_resources/data game_server/src '/mnt/f/unity projects/ark/Assets/Scripts' -g '*.{rs,ron,cs}'
sed -n '<planner/output/path/target ranges>' core_enhanced/src/game/battle/core/movement/*.rs core_enhanced/src/game/battle/core/targeting.rs
sed -n '<Core DTO/server projection ranges>' core_enhanced/src/game/behavior.rs core_enhanced/src/game/world/state.rs game_server/src/game/player_game_actor/messages.rs
sed -n '<Unity update/reconcile/readiness/movement ranges>' '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/BattleRuntimeState.cs' '/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Battlefield/BattlefieldFieldModel.cs'
sed -n '40,95p' game_server/src/main.rs
sed -n '110,150p' game_server/src/lib.rs
sed -n '110,150p' game_server/src/game/player_game_actor/session.rs
sed -n '120,175p' core_enhanced/src/game/world/state.rs
sed -n '120,140p' core_enhanced/docs/core_runtime_contract.md
```

### Failed/truncated exploratory commands

- Initial broad `rg` commands over data/setup, movement/target, server DTO and Unity consumer returned truncated
  output. No conclusion relies on an omitted line; the relevant definitions were subsequently read with the bounded
  `sed` commands above.
- One exploratory command attempted `sed` on nonexistent `core_enhanced/src/game/combat_preview.rs`; the module is a
  directory, and that failed path produced no evidence or mutation.

### Actual result and decision

- Current loader/source count, startup-only atomic publication and absence of Boss catalog/binding match readiness.
- P02/P03 remain separate atomic review lifetimes: compiled data/setup first, runtime kernel second. P05 retains the
  all-difficulty/full-feasibility publication gate.
- Existing movement goals/output and target eligibility are reusable leaves, but Boss intent/result, static-grid path
  cost, target lock and monotonic rejoin cursor remain missing.
- Server manual top-level projection and Unity lifecycle/facing gaps remain first-public-contract risks; P06 stays
  conditional, while P04 remains explicitly Wire-neutral.
- Startup-shared immutable data plus non-persisted in-memory active battle supports the no-in-place-migration
  assumption. No policy stop is needed for P01.
- Updated `CURRENT_TREE_DRIFT_AUDIT.md` immediately. No test runner was executed in this unit.

## 2026-08-09 — Refactor execution registry closure

### Exact commands

```text
rg -n '^### CRA-(P01-C01|P01-C02|P05-C02|P05-C04|P06-C01|P06-C02|P06-C03|P07-C01|P07-C02|P07-C03|P07-C04|P08-C01|P08-C05|P09-C03|P09-C05|P09-C06|P10-C02|P10-C03|P10-C04)' core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_*/CANDIDATE_CATALOG.md
sed -n '<relevant candidate ranges>' core_enhanced/docs/goals/core_enhanced_refactoring_audit_phase_{01,05,06,07,08,09,10}_*/CANDIDATE_CATALOG.md
rg -n '<tactical final-state and movement backend/ghost symbols>' core_enhanced/src/game/battle/core -g '*.rs'
rg -n '<trigger and committed damage symbols>' core_enhanced/src/game/battle -g '*.rs'
rg -n '<tile reach/projectile duplication symbols>' core_enhanced/src/game/{battle,range_preview.rs} -g '*.rs'
sed -n '210,250p' core_enhanced/docs/goals/boss_runtime_v1_implementation_master/PLAN.md
sed -n '620,695p' core_enhanced/docs/goals/boss_runtime_v1_implementation_master/PLAN.md
rg -o 'CRA-P[0-9]{2}-C[0-9]{2}' core_enhanced/docs/goals/boss_runtime_v1_implementation_master_phase_00_execution_preflight/REFACTOR_EXECUTION_REGISTRY.md | sort | uniq -c
rg -o 'CRA-P[0-9]{2}-C[0-9]{2}' core_enhanced/docs/goals/boss_runtime_v1_implementation_master_phase_00_execution_preflight/REFACTOR_EXECUTION_REGISTRY.md | sort -u | wc -l
```

### Actual result

- Current symbol checks confirm the cataloged seams still exist: split tactical-effect mutation, player-command
  post-mutation cursor risk, Direct/Rapier dual test surface and ghost movement fields; split trigger/damage paths;
  duplicated projectile/tile-reach adapters; and manually reauthored server DTO fields.
- The registry contains exactly 19 unique candidate IDs. No requested ID is missing or additional candidate added.
- Seven candidates are unconditional Boss-phase implementation: P06-C01/C02/C03, P01-C01, P05-C04, P01-C02 and
  P05-C02. P10-C03 is conditional on P06. Three are deferred after P04 seam proof and eight remain independently
  owned, outside Boss completion.
- Created `REFACTOR_EXECUTION_REGISTRY.md` with current problem, authority, invariant, overlap, entry/exit,
  validation/policy gate and final disposition per candidate.
- No production file or test was changed and no runner was executed.

### Decision

The approved Boss Master does not authorize the full refactor catalog. Dependency-coupled corrections/refactors land
in their Boss Phase; independent and cleanup candidates require separate Goal authorization and cannot block v1.

## 2026-08-09 — P01 test inventory oversized attempt

### Attempted command shape

```text
sed -n '1,120p' core_enhanced/Cargo.toml
sed -n '1,100p' Cargo.toml
rg -n '^\s*(pub\s+)?fn ...' <P01 source and test owners> | rg <step/order/advance/cursor/revision filters>
```

### Actual result

- The combined output exceeded the available result window and was fully truncated before any bounded evidence could
  be retained.
- This attempt therefore proves neither package/test names nor P01 test coverage and is not used for a conclusion.
- The command was read-only: it did not mutate source, tests, canonical data, server code or Unity code, and it did
  not execute a test runner.

### Decision

Repeat the inventory as separate bounded reads: manifests first, then one owner file and one symbol filter at a time.
Only the bounded reruns may be cited by `P01_G0_HANDOFF.md`.

## 2026-08-09 — P01 parallel source-read aggregation overflow

### Exact attempted reads

```text
sed -n '1030,1215p' core_enhanced/src/game/battle/core/mod.rs
sed -n '2190,2415p' core_enhanced/src/game/battle/core/mod.rs
sed -n '2470,2575p' core_enhanced/src/game/battle/core/mod.rs
sed -n '1,500p' core_enhanced/src/game/battle/enums.rs
```

### Actual result

- The four read-only commands completed, but their aggregated response was truncated (`10213` original response
  tokens). Some fragments were visible while other requested ranges were omitted.
- No conclusion may rely on the incomplete aggregate. Each required range must be rerun independently or narrowed
  further before it is cited in the P01 handoff.
- No mutation and no test execution occurred.

### Decision

Do not aggregate multiple source bodies into one result. Retain one bounded owner/range per evidence call.

## 2026-08-09 — P01 path/filter corrections

### Failed read paths

- `core_enhanced_refactoring_audit_phase_06_correctness_runtime/CANDIDATE_CATALOG.md` does not exist. Bounded
  discovery resolved the authority to
  `core_enhanced_refactoring_audit_phase_06_battle_lifecycle_and_result/CANDIDATE_CATALOG.md`; its C01~C03 bodies
  were then read successfully.
- `core_enhanced/src/game/world/mod.rs` does not exist. Bounded file discovery resolved the module root to
  `core_enhanced/src/game/world.rs`; `world/state.rs` remains the state owner.
- Both failures were read-only and produced no mutation.

### Exact initial runner attempt

```text
cargo test -p game_core battle_event_heap_orders_by_time_then_priority_then_ids -- --exact --nocapture
```

### Actual result

- Exit code was `0`, but the exact filter matched no fully qualified test: `0 passed`, `0 failed`, `0 ignored`,
  `862 filtered out` in the library target. All other enumerated targets also ran zero tests.
- This is a filter-feasibility failure, not a passing ordering test.
- `cargo test -p game_core --lib -- --list` then resolved the exact P01 paths, including
  `game::battle::enums::tests::battle_event_heap_orders_by_time_then_priority_then_ids`,
  `game::battle::core::tests::battle_runtime_policy_controls_max_battle_time`, and the current future-cursor and
  revision-overflow world tests.

### Decision

Use fully qualified names together with `--lib -- --exact` for focused P01 commands, and report zero-match runs as
failures rather than green evidence.

## 2026-08-09 — P01 bounded owner/test feasibility closure

### Exact bounded evidence

```text
sed -n '1,140p' Cargo.toml
sed -n '1,160p' core_enhanced/Cargo.toml
sed -n '1,140p' game_server/Cargo.toml
rg -n '<P01 function names>' <one P01 owner at a time>
sed -n '<bounded owner ranges>' core_enhanced/src/game/battle/core/sim.rs
sed -n '1,220p' core_enhanced/src/game/battle/enums.rs
sed -n '220,475p' core_enhanced/src/game/battle/enums.rs
sed -n '475,620p' core_enhanced/src/game/battle/enums.rs
sed -n '<bounded advance/test/state/trigger ranges>' core_enhanced/src/game/world/{combat.rs,state.rs,visual_novel_triggers.rs}
cargo test -p game_core --lib game::battle::enums::tests::battle_event_heap_orders_by_time_then_priority_then_ids -- --exact --nocapture
cargo test -p game_core --lib game::battle::core::tests::battle_runtime_policy_controls_max_battle_time -- --exact --nocapture
cargo test -p game_core --lib game::world::tests::combat::live_defense_battle_state_rejects_future_since_seq -- --exact --nocapture
```

### Actual result

- Workspace package names are `game_core` and `game_server`; P01 unit owners compile in the `game_core` library
  target. Fully-qualified `--exact` filtering is feasible.
- The three exact baseline tests each ran exactly once and passed: aggregate `3 passed`, `0 failed`, `0 ignored`.
  Each run reported `861 filtered out` in the library target.
- Existing max-time coverage proves only `run_battle`; existing ordering coverage is one representative heap case;
  existing future-seq coverage calls read-only resync. Their green status does not close C01~C03.
- C01 has no external `BattleFinishSignal` consumer: all references are inside Core, world orchestration and Core unit
  tests. A state-owned finalization signature is therefore a local correctness change.
- C02 has 19 live event variants. Current priority bands are complete, but same-priority Auto/Manual cast end lacks a
  cross-variant discriminator and many same-variant payloads/cause fields are absent from ties.
- C03's fallible surfaces are concrete: future `after_seq`; revision overflow; invalid trigger lookup/repeat context;
  missing employee-attempt roster entry during incapacitation reconciliation; and DTO construction following a
  revision commit. `transition_to` currently cannot fail, so no invented transition failpoint is required.
- `BattleExecutionState` is cloneable, but `BattleCore`, `ActiveBattleSession` and `GameCoreState` have no full clone
  implementation. P01 must introduce a scoped staging/rollback seam rather than claiming an existing transaction.

### Decision

Use one P01 child with three reviewable milestones C01 → C02 → C03. They share execution/event/world-test authority,
and splitting them into simultaneously active children would make the dirty-tree owner ambiguous. Write failing
characterization tests before each milestone, preserve the current priority bands, and build the world transaction
around the settled C01/C02 semantics.

## 2026-08-09 — P00 completion precheck

### Exact commands

```text
git status --short --untracked-files=all | rg -v '^\?\? core_enhanced/docs/goals/boss_runtime_v1_implementation_master_phase_00_execution_preflight/' | tee /tmp/boss_p00_status_excluded.txt | sha256sum
wc -l /tmp/boss_p00_status_excluded.txt
git status --short --untracked-files=all | wc -l
test -s <each of the seven pre-audit required P00 artifacts>
rg -o 'CRA-P[0-9]{2}-C[0-9]{2}' REFACTOR_EXECUTION_REGISTRY.md | sort -u | wc -l
rg -o 'BP-[0-9]{2}' core_enhanced/docs/tmp/boss_behavior_state_machine_policy_notes_2026-08-07.ko.md | sort -u | wc -l
rg '^\| P0[0-7] \|' core_enhanced/docs/goals/boss_runtime_v1_implementation_master/PLAN.md | wc -l
rg -l '^Status: (active|policy_decision_required|scope_decision_required|externally_blocked|correction_required)$' core_enhanced/docs/goals/*/PLAN.md
```

### Actual result

- All seven required pre-audit artifacts are nonempty; this completion audit will be the eighth.
- The refactor registry has exactly `19` unique CRA IDs. The target policy source has exactly `34` unique IDs,
  `BP-01` through `BP-34`. The Master registry has exactly `8` Phase rows, P00 through P07.
- Only the Master and P00 are active/nonterminal; no separate production Goal appeared during P00.
- Excluding only P00's new untracked files, status is exactly `221` entries with SHA-256
  `41f02b5b73145453f032c3dfcf2931f1225db3225cb70a7ccaf82de5e882bd8d`, identical to P00's pre-creation baseline.
  Full status is `228` before adding `COMPLETION_AUDIT.md`, exactly baseline plus seven P00 files.
- Cargo created no tracked or untracked status entry outside the already ignored build output.

### Decision

No collision, scope, access, migration or policy blocker prevents P01. Perform the independent requirement-by-
requirement completion review, then update P00 and Master status before creating P01.
