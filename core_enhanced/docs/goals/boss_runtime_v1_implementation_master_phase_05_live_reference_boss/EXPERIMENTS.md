# Boss Runtime v1 Phase 05 — Experiments

Goal: `boss_runtime_v1_implementation_master_phase_05_live_reference_boss`  
Execution Mode: `implement_and_verify`  
Production Mutation Authorized: yes — bounded P05 scope only

## 2026-08-09 — Initialization and activation

### Hypothesis

P04 is `Complete / High`, the user approved decision-packet option 1 as disposable reference content, no other
Phase is active and the P05 path is absent, so P05 can be initialized and activated without production mutation.

### Exact commands

```text
sed -n ... /home/acr0tic/.codex/attachments/740e5d2a-01fd-4751-a956-eaaee449b1b3/pasted-text-1.txt
sed -n ... core_enhanced/docs/goals/boss_runtime_v1_implementation_master/{PLAN.md,EXPERIMENTS.md,EXPERIMENT_NOTES.md}
git status --short --untracked-files=all | wc -l
git status --short --untracked-files=all | sha256sum
test ! -e core_enhanced/docs/goals/boss_runtime_v1_implementation_master_phase_05_live_reference_boss
git rev-parse HEAD
sed -n ... core_enhanced/docs/{README.md,engineering_workflow.md,goal_authoring_and_execution_guide.md,goal_completion_review_guide.md,game_rulebook.md,core_runtime_contract.md,data_loading_contract.md,skill_target_contract.md}
sed -n ... core_enhanced/docs/goals/boss_runtime_v1_implementation_master_phase_04_track_and_rejoin_vertical_slice/COMPLETION_AUDIT.md
```

Every listed document was read to EOF in bounded chunks. Ellipses denote the exhaustive recorded chunk ranges, not
unread sections.

### Actual result

- P04 prerequisite: `Complete / High`, handoff `ready_for_p05_decision`.
- User decision: option 1 approved; missing Little Red details may be agent-authored as replaceable test/reference
  content and are not final character canon.
- P05 path was absent and no other Phase was active.
- Precreation repository state: HEAD `f318414fb0156a67e04c5479000c2a2683fc0c09`, 261 sorted status entries,
  SHA-256 `832ea5972d60e3613d9dd3252dab0299853617c41f88c426f1e9f75d17ea73a8`.

### Changes and decision

- Created only this Phase's required `PLAN.md`, `EXPERIMENTS.md` and `EXPERIMENT_NOTES.md` and updated Master
  coordination/approval records.
- P05 is the sole active production Phase. P06 remains pending and cannot receive production work.
- No production source, live RON, test, canonical, server or Unity file changed in this unit.
- No test runner ran; pass/fail/skip counts are therefore not reported.

### Postcreation and external-consumer baselines

- After the three P05 documents and Master coordination records exist, Simulator status is exactly 264 sorted
  entries with SHA-256 `1b248221b4fe4dbfe57c7f76e4eb163d9863d6a4a67910c42adb15cedeed864f`.
- External Unity remains at HEAD `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf`, but its current user-owned dirty
  baseline is now 2154 sorted entries with SHA-256
  `c4c939c7b4a52c886aa00ad4c11aa68f136a72fc7fcbacc4eb28f833713e47f1`.
- The earlier 2149-entry Unity snapshot is stale and will not support P05 completion claims. The current external
  tree is read-only for P05.

## 2026-08-09 — Current external contract and renderer revalidation

### Evidence

- The two current external canonical files and `BattlefieldFieldAssembler.cs` are all user-modified. Their combined
  diff is large (`1383 insertions`, `754 deletions`), so P05 did not reuse the earlier policy-research interpretation
  without rereading current content.
- Current SHA-256 values are:
  - `unity_core_contract.md`: `acf725a23ddc29b351c4eefc3b4f7e6ac79c9d88d694eaab20d76e92753ab349`
  - `core_unity_battle_transport_contract.md`:
    `45d13f8fb1a2737ec620867912344fa868816855eefa625cc3efc94a9ca6f599`
  - `BattlefieldFieldAssembler.cs`: `24164877c3b2c9c6c57a1874ae31a6d47e5771f8e18136e79da55e0723d6c83f`
- The current battle transport still treats Boss/Boss as DefenseRoute over the same setup/update/checkpoint path.
  `unit_source`, Core-provided Boss HUD, `MovementSegmentStarted`/`MovementStopped`, leaf skill/projectile/damage/death
  events and checkpoint reconciliation are sufficient inputs for the approved reference behavior.
- The current renderer still branches every `owner == Opponent` actor to `EnsureEnemyUnitCapsule`; it does not inspect
  abnormality identity to select dedicated visuals. Therefore Little Red naturally uses the approved common enemy
  capsule and no consumer change is needed for identity-specific presentation.

### Decision

- Continue P05 as an existing-event/content-only publication. Do not mutate or normalize the concurrent Unity work.
- P06 remains pending until actual live-path tests and final source audit confirm that no additional semantic event is
  required. This read-only inspection is necessary evidence but not by itself the final disposition.

## 2026-08-09 — Publication-gate gap revalidation

### Evidence

- `validate_generated_combat_preview_contracts()` currently delegates only to
  `validate_combat_preview_threat_warning_contract()`.
- That validator loops all encounters and the fixed seeds `0, 1, 17, 41, 99`, but always calls the no-difficulty
  `try_generate_for_node_at_floor()` entry with `MapNodeCategory::Combat`. Its errors explicitly describe a
  `baseline difficulty` run. It therefore does not exercise any authored
  `MapNodeCombatDifficulty` profile/band/role/Monolith-stage representative.
- Runtime setup uses `try_generate_for_node_at_floor_with_difficulty()`, so generated-wave budgets and extra waves
  can differ from what startup currently validates. This reconfirms CRA-P01-C02 from current source.
- The live scenario path is currently assembled privately inside `CombatExecutor::build_battle_scenario_for_encounter`:
  player start group, authored/default tactical plan and win condition, defense object, resolved enemy groups,
  `resolve_boss_runtime_setup`, timed groups and final `BattleScenario`.
- Runtime then applies `validate_static_obstacles_do_not_overlap_scenario`, `BattleScenario::validate()` and, on the
  live start path, `start_battle_execution()`. Startup publication calls none of those setup/scenario predicates.
  This reconfirms CRA-P05-C02 from current source.
- `enemy_spawn_groups_from_preview()` is deterministic and derives owned UUIDs from authored base UUID, wave index
  and spawn index; it does not consume mutable world UUID or run state. An empty-player feasibility assembly can
  therefore reuse this setup authority without committing a battle.

### Decision

- CRA-P01-C02 owns one finite difficulty-representative matrix for every profile: all authored ordinary bands, all
  non-Monolith roles, and Monolith remanifestation at both Stage2 and Stage3. The existing five seeds remain exact.
- CRA-P05-C02 must consume the same generated previews, not re-enumerate a second matrix. It will assemble the same
  empty-player scenario shape and run the same setup/scenario validation predicates before candidate publication.
- Event/non-manifested baseline generation remains a separate preserved path; the new matrix does not invent a
  difficulty for Event combat.
- Before production edits, add negative characterization tests for a difficulty-only failure and for setup
  infeasibility, then capture fresh focused test counts and startup-cost evidence.

## 2026-08-09 — Resume and dirty-baseline reconciliation

### Evidence

- The resume attachment, Master required documents, P05 required documents, P04 completion audit, current workflow
  guides, approval packet and gameplay/runtime/data/targeting canonicals were read to EOF before code search, tests or
  production mutation.
- Current Simulator status contains 265 sorted entries with SHA-256
  `25a908696abc6278decb4a96eac0ecf24bdce2e7c97fef0b68e620a0453cbd1d`.
- Brute-force single-entry subtraction proves the only delta from the recorded 264-entry postcreation baseline is the
  untracked range-outside log `game_server/logs/app.log.2026-08-09`. Excluding that exact entry restores SHA-256
  `1b248221b4fe4dbfe57c7f76e4eb163d9863d6a4a67910c42adb15cedeed864f`.

### Decision

- Preserve the log as user-owned concurrent state; do not delete, truncate or absorb it into P05.
- The P05 production-owner baseline otherwise remains exact, so scoped owner diff/hash review and focused test
  baselining may continue.
- No production file or test changed in this reconciliation unit. No runner ran, so no pass/fail/skip count is
  reported.

## 2026-08-09 — Pre-mutation owner and focused baseline

### Exact commands

```text
sha256sum core_enhanced/src/game/data/mod.rs core_enhanced/src/game/data/validation.rs core_enhanced/src/game/data/live_ron.rs core_enhanced/src/game/events/combat.rs core_enhanced/src/game/combat_setup/mod.rs core_enhanced/src/game/world/tests/combat.rs core_enhanced/tests/ron_loading.rs game_resources/data/boss_runtime/base.ron game_resources/data/pve/encounters.ron core_enhanced/docs/game_rulebook.md core_enhanced/docs/core_runtime_contract.md core_enhanced/docs/data_loading_contract.md core_enhanced/docs/skill_target_contract.md
cargo test -p game_core --lib game::data::live_ron::tests:: -- --quiet
cargo test -p game_core --test ron_loading -- --quiet
cargo test -p game_core --lib game::events::combat::tests:: -- --quiet
cargo test -p game_core --lib game::world::tests::combat::track_and_rejoin_ -- --quiet
```

### Actual result

- Relevant pre-mutation hashes are: data/mod `4f51fdc3...d895`, data/validation `ed356a9a...fd8`,
  data/live_ron `7f561c6e...bbe3`, events/combat `20868201...15df`, combat_setup/mod `1d2471ba...3446`,
  world combat tests `b42e742e...383603c`, RON integration `573dbd39...d124`, Boss RON
  `265bdfd4...76ac`, PvE RON `a9d72eec...2941`, and the four canonical hashes are durably available in
  the command output from this baseline.
- Live source-set unit baseline: `59 passed`, `0 failed`, `0 ignored`, `865 filtered out`.
- The first RON integration invocation compiled successfully but the yielded output omitted the runner summary, so
  it was not counted. The immediate cached rerun reports `24 passed`, `0 failed`, `0 ignored`, `0 filtered out`.
- Combat setup baseline: `15 passed`, `0 failed`, `0 ignored`, `909 filtered out`.
- P04 vertical-slice baseline: `8 passed`, `0 failed`, `0 ignored`, `916 filtered out`.

### Decision

- The current dirty tree is green on every focused surface that P05 will extend. No baseline failure exists.
- Add the two planned negative characterization tests next, before changing validator/setup production behavior.

## 2026-08-09 — Failing publication-gate characterization

### Exact command

```text
cargo test -p game_core --lib game::data::live_ron::tests::live_candidate_rejects_ -- --quiet
```

### Actual result

- `0 passed`, `2 failed`, `0 ignored`, `924 filtered out`.
- `live_candidate_rejects_preview_failure_unique_to_manifested_difficulty` constructs a domain-valid Easy-only
  extra wave whose spawn-zone reference makes manifested preview generation fail. Current publication returns `Ok`.
- `live_candidate_rejects_boss_binding_that_cannot_build_runtime_setup` attaches a domain-valid TrackAndRejoin
  binding to the route-less live Little Red primary wave. Current publication also returns `Ok` even though runtime
  setup requires an authored route.
- Both failures are the intended red baseline. There was no compile or fixture error and no expectation was weakened.

### Decision

- Implement one shared manifested-difficulty matrix and feed each generated preview through both briefing validation
  and the actual scenario/setup feasibility predicate before the candidate can publish.
- Keep Event/non-manifested baseline validation separate and do not duplicate the scenario field assembly inside the
  data validator.

## 2026-08-09 — Manifested-difficulty and full-setup publication gate

### Changes

- The generated-preview validator now derives one matrix from each validated live difficulty profile: all three
  ordinary bands, Elite/NormalBoss/FinalBoss/BossOmen, plus MonolithRemanifestation at Stage2 and Stage3.
- Each encounter uses its actual Combat/Boss category and each existing seed `0, 1, 17, 41, 99`. Every generated
  preview feeds both briefing-warning validation and a full runtime setup feasibility wrapper.
- The wrapper delegates to the existing `CombatExecutor::resolve_live_combat_setup_plan` with an empty player group,
  so rewards, field plan, enemy groups, Boss binding, `BattleScenario` validation, `BattleCore` construction and
  `start_battle_execution` have one field-assembly authority. Data validation does not duplicate the scenario shape.
- The non-manifested baseline preview/warning path remains separate and unchanged for Event/fixture semantics.

### Exact commands and results

```text
cargo test -p game_core --lib game::data::live_ron::tests::live_candidate_rejects_ -- --quiet
cargo test -p game_core --lib game::data::live_ron::tests:: -- --quiet
/usr/bin/time -f 'elapsed=%e max_rss_kb=%M' cargo test -p game_core --test ron_loading -- --quiet
```

- Negative gate tests: `2 passed`, `0 failed`, `0 ignored`, `924 filtered out`.
- Live source-set unit surface: `61 passed`, `0 failed`, `0 ignored`, `865 filtered out`.
- RON integration: `24 passed`, `0 failed`, `0 ignored`, `0 filtered out`; measured whole command elapsed `26.09s`,
  runner test time `1.44s`, maximum RSS `1,919,916 KB`. This includes recompilation and is a correctness/startup-cost
  watch, not a production process benchmark.
- The first single Boss negative rerun exposed the runtime's exact error spelling (`requires a non-empty authored
  route`); only the test substring was corrected to that authoritative message, then the two-test filter passed.

### Decision

- CRA-P01-C02 and CRA-P05-C02 now have executable fail-closed publication gates. Proceed to the one approved live
  Little Red catalog/binding migration and prove the new production candidate through these same gates.

## 2026-08-09 — Atomic Little Red live publication and content characterization

### Changes

- `boss_runtime/base.ron` now contains exactly one Pattern, behavior and encounter: `little_red_pursuit`,
  `little_red_reference_behavior` and `little_red_reference_encounter`.
- The Pattern exhaustively maps AcquireTarget's two results, MoveToTarget's four results and
  RejoinForwardRoute's three results. Every failure/no-target outcome reaches the existing resident
  `route_combat`; the fixed sequence is non-repeating.
- Only `suppress_little_red` opts in. Its `primary_abnormality` slot binds the same wave at spawn index 0 and only
  that wave gained `route_id: "defense_main"`.
- Added an integration characterization that pins the single-catalog inventory, exhaustive transitions, exact
  binding, current two-wave composition/timing, three reward UUIDs, stats `800/70/35/35`, movement `1500` and
  `little_red_hunt_the_prey` skill.

### Exact commands and results

```text
cargo test -p game_core --test ron_loading live_little_red_is_the_only_boss_runtime_reference_and_preserves_existing_content -- --exact --nocapture
cargo test -p game_core --test ron_loading -- --quiet
```

- Focused content characterization: `1 passed`, `0 failed`, `0 ignored`, `24 filtered out`.
- Full RON integration after the world-fixture repair below: `25 passed`, `0 failed`, `0 ignored`, `0 filtered out`.
- Before the new integration assertion was added, the production RON candidate also passed the then-current full
  RON integration surface at `24 passed`, `0 failed`, `0 ignored` in `1.48s`.

### Decision

- The migration stays content-only. No stat, reward, automatic skill, support-wave, DTO, save or Unity mutation was
  needed.
- The exact live inventory assertion is intentional: a second Boss opt-in or reference graph is a future policy and
  cannot enter this bounded Phase accidentally.

## 2026-08-09 — Actual live world lifecycle, failure and ordinary-parity matrix

### Changes and coverage

- Added production-data world setup for `suppress_little_red` through an actual Boss node. It proves the internal
  setup exists before the 5-second spawn while setup/update JSON contains no `boss_runtime` field.
- The success path pins exact actor identity and target, one pursuit, rejoin, monotonic route cursor and resident
  RouteCombat. Pause freezes the full authoritative snapshot; active reconnect is read-only and sends no private
  graph state.
- Live no-target, locked-target invalidation and active-but-unusable-path cases each reach the authored ordinary
  route without implicit reacquisition. The timeout test uses the production live graph/binding and a test-only
  zero-movement actor to prove the exact 10-second Move and Rejoin battle-time deadlines and exhaustive exit.
- Actor death leaves its stable slot identity but does not schedule duplicate runtime work. Killing the remaining
  required live opponents resolves terminal state exactly once; a later advance rejects.
- `suppress_big_bad_wolf`, an actual non-opt-in live Boss encounter, retains no internal Boss runtime, no public
  Boss graph shape and read-only reconnect semantics.
- Added a production-content integration assertion for the full authored Task-result exhaustiveness. The exhaustive
  P04 runtime regression matrix remains the shared kernel proof for failure arbitration and was rerun unchanged.

### Exact commands and results

```text
cargo test -p game_core --lib game::world::tests::combat::live_little_red_ -- --quiet
cargo test -p game_core --lib game::world::tests::combat::live_non_opt_in_boss_encounter_keeps_ordinary_runtime_and_public_shape -- --exact --quiet
cargo test -p game_core --lib game::world::tests::combat::track_and_rejoin_ -- --quiet
```

- Live Little Red matrix: `3 passed`, `0 failed`, `0 ignored`, `927 filtered out`.
- Live non-opt-in parity: `1 passed`, `0 failed`, `0 ignored`, `929 filtered out`.
- P04 shared-kernel regression after fixture repair: `8 passed`, `0 failed`, `0 ignored`, `922 filtered out`.

### Observed failures and corrections

- The first live matrix run was `2 passed`, `1 failed`: the test incorrectly expected a dead RouteCombat actor's
  stable slot identity to disappear. Current runtime intentionally retains the actor slot/identity while suppressing
  further work. The assertion now proves an identical runtime snapshot across a second same-time reconciliation.
- The first P04 regression rerun was `7 passed`, `1 failed`: its raw-source fixture replaced the entire Boss catalog
  with the P04 catalog but inherited the new production Little Red PvE opt-in, creating an expected dangling
  definition. The isolated fixture now clears existing live Boss bindings before installing its own one. No
  production loader/runtime expectation was weakened; the immediate rerun passed all eight tests.

### Decision

- Existing Core events/checkpoint/reconnect are sufficient for the actual reference. There is no evidence for a new
  public semantic Boss event, so P06 should be dispositioned `not_applicable_to_v1` after the final external/source
  audit confirms unchanged consumers.
- Automatic Little Red skill execution remains leaf-runtime authority; the Boss graph owns only its one finite
  pursuit target and never observes or rewrites step-level skill retarget state.

## 2026-08-09 — Canonical alignment for the implemented live boundary

### Changes

- `game_rulebook.md` now identifies Little Red as the sole replaceable live reference and records the finite
  pursuit, preserved content and explicit absence of bounty/art/final-design meaning.
- `core_runtime_contract.md` no longer claims that live reference binding is absent; it records the exact opt-in,
  query and authored exits while retaining the private graph/public-shape boundary.
- `data_loading_contract.md` now records the one live catalog/binding and the full manifested-difficulty/setup
  publication matrix. `skill_target_contract.md` was not changed because Boss publication did not alter skill cast
  selection or step-target authority.

## 2026-08-09 — Broad validation and same-generation fixture corrections

### Exact final commands and results

```text
cargo check -p game_core --lib
/usr/bin/time -f 'elapsed=%e max_rss_kb=%M' cargo test -p game_core --all-targets -- --quiet
/usr/bin/time -f 'elapsed=%e max_rss_kb=%M' cargo test -p game_server --all-targets -- --quiet
/usr/bin/time -f 'elapsed=%e max_rss_kb=%M' cargo test --workspace --all-targets -- --quiet
cargo fmt -p game_core -p game_server -- --check
git diff --check
```

- Core check passed.
- Final Core all-targets: library `930 passed`; integration targets `3 + 4 + 25 + 13 + 4 passed`; all binary/
  generator zero-test targets passed. No failure/ignored/measured/filtered test remained. Elapsed `40.01s`, maximum
  RSS `100448 KB`.
- Final game_server all-targets: library `56 passed`; binary and three example zero-test targets passed. Elapsed
  `11.10s`, maximum RSS `816708 KB`.
- Final workspace all-targets exited `0`: Core `930 + 3 + 4 + 25 + 13 + 4`, server `56`, all zero-test targets;
  elapsed `65.17s`, maximum RSS `2087232 KB`.
- Formatter check and `git diff --check` both passed with no output.

### Broad failures found and corrected

- The first Core all-targets run stopped at library `929 passed / 1 failed`: the P05 negative setup test still
  asserted that production Little Red had no route. It now clears the production opt-in and attaches the invalid
  route-dependent fixture to route-less `suppress_big_bad_wolf`; the exact negative and all 930 unit tests pass.
- The next Core all-targets run reached `skill_runtime_contract` at `12 passed / 1 failed`, then the following run
  reached `skill_test_suite` at `0 passed / 4 failed`. Both fixture builders copied production live PvE without its
  same-generation Boss catalog. They now copy both immutable databases together; focused reruns and the final full
  run pass. This strengthens atomic-generation semantics rather than bypassing binding validation.
- The first game_server all-targets run was `49 passed / 7 failed` for the same fixture-generation mismatch in its
  defense-data builder. Copying the source live Boss database with cloned live PvE repaired all seven; final server
  is `56/0`.
- No production validation, content, DTO or behavior expectation was relaxed to address any broad failure.

## 2026-08-09 — Final external consumer and dirty-tree revalidation

### Evidence

- External Unity remains at HEAD `7cd8df1f5ae2e438e28bda2c761e7a56d21e59cf` with `2154` sorted dirty entries.
  Its whole-tree status hash evolved concurrently from the P05 baseline to
  `d5cb19cf8fbc9ef6664e8b0a0772602853e725adedccdb1f664481824fe20b69`; P05 only executed read-only git/hash/search
  commands there.
- The three exact consumer evidence owners remain byte-identical to their P05 baseline:
  `unity_core_contract.md` `acf725a2...349`, `core_unity_battle_transport_contract.md` `45d13f8f...599`, and
  `BattlefieldFieldAssembler.cs` `24164877...c83f`.
- Current external source still consumes unit identity, existing movement/death events and checkpoint, maps every
  opponent to the common enemy capsule and contains no `boss_runtime` consumer. No external file was changed.
- Existing producer DTO/server mapping is unchanged; actual live world tests prove serialized setup/update/reconnect
  omit private Boss graph state and final server all-targets passes.

### P06 disposition

- P06 is `not_applicable_to_v1`. The actual approved live reference is fully representable by existing event and
  checkpoint semantics; no synchronized public lifecycle field/variant or Unity inference is needed.
- This disposition does not prohibit a future public Boss presentation contract. It means the bounded v1 reference
  supplies no evidence that would authorize or require P06 production mutation.

### Simulator preservation

- Before the completion-audit artifact, Simulator remains exactly `265` sorted status entries with SHA-256
  `25a908696abc6278decb4a96eac0ecf24bdce2e7c97fef0b68e620a0453cbd1d`, identical to the recorded resume state.
- The only delta from the 264-entry P05 postcreation baseline remains the preserved untracked
  `game_server/logs/app.log.2026-08-09`; P05 production edits changed contents of already-dirty owner paths and did
  not add, delete, clean, reset or normalize unrelated status entries.
