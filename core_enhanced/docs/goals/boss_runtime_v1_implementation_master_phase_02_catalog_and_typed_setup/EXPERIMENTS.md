# Boss Runtime v1 Phase 02 — Experiments

## 2026-08-09 — Activation baseline

### Required input checkpoint

- P01 completion audit is `Complete / High` with `ready_for_p02`.
- Rechecked the current Master P02 boundary, migration/validation rules, P00 CRA registry, current data canonical,
  readiness boundary/report and approved BP definition/setup policies before creating this child.
- The current source-of-truth discrepancy remains explicit: runtime/audit inventory is manifest + 22 gameplay files,
  while `data_loading_contract.md` still says 21. P02 must resolve this through code-derived inventory and canonical
  correction, not by copying another manual count.
- No production source was searched, tested or mutated in this activation unit.

### Exact baseline

```text
git status --short --untracked-files=all | tee /tmp/boss_p02_precreation_status.txt | sha256sum
wc -l /tmp/boss_p02_precreation_status.txt
```

- Pre-creation status: 235 entries.
- SHA-256: `773eb0975112655528635ff7fcb6e38b0e690ded945ba4d86127f06bc704c3cd`.
- The complete status snapshot is retained at `/tmp/boss_p02_precreation_status.txt` for exact Phase-exit comparison.

### Decision

Activate one P02 child with four sequential milestones: current characterization → strict definition compiler →
atomic source/inventory integration → typed setup plan. Production mutation is authorized only after this activation
record and only inside that scope.

## 2026-08-09 — M1 initial owner inventory

### Current source findings

- `live_ron.rs` declares `LIVE_GAMEPLAY_RON_SOURCE_COUNT = 22` and manually fields one manifest plus 22 gameplay
  strings through `GameplayRonSourceSet`, parsing and candidate parts. The canonical's “21” is confirmed stale.
- `PveEncounter` is strict and has no Boss runtime reference/binding.
- `BattleScenario`/`ScenarioUnitSpawn` have no Boss setup binding. `enemy_spawn_groups_from_preview` resolves authored
  route cells per group, and `world/combat.rs` currently constructs `ActiveBattleSession` field-by-field.
- All nine likely existing production/test owners except `data/mod.rs`, most `combat_setup` helpers and `bundle.ron`
  are already modified in the user baseline. P02 will prefer new Boss-domain files and narrow symbol-level edits.

### Pre-mutation owner hashes

- `data/live_ron.rs`: `98142491ec2f61a73a255cd4168a318fac8b914fae9ddf771acc1f5388e6a2ba`
- `data/mod.rs`: `4ef94fcf8ea74ea36bdf9d2267b21ab60b56165bee3b2b0dc09d994f02dbe46b`
- `data/pve_data.rs`: `73f3bc6ad0d23fd4fe5384974fea4ccb89f459bb5b205ea8e5a97db27d32d013`
- `data/validation.rs`: `ed356a9a23e478419b30501372c9d6f566b6a49d588cee42c567438c250fcfd8`
- `battle/scenario.rs`: `a7d166ff80d48a7dab49a7597f87f1bdd99f44fe767452517ce13114411c98a4`
- `combat_setup/mod.rs`: `3bcb82dd49b94cf96b4c76af63bbd80a2d0c91f50ff4e95f4d9110984b530f1f`
- `combat_setup/enemy_spawns.rs`: `e3c15adc672d98e981d5648e33232637c09d87fcda6b24aef3add585e28af0b0`
- `world/combat.rs`: `6074ce6beb6723ab938947e6eef7ff4ca5eb7615392df126a620342548426db6`
- `world/state.rs`: `74c3dabb70b44b8d6192e663e942663584afad9484a971351d74ed5573f58811`
- `tests/ron_loading.rs`: `573dbd399679042af501dc9035b97e9f9f90e90f8afd40a84234e4cb8536d124`
- `data/bundle.ron`: `06b0f18905acf5efb75e3c790be85d66879735614c3b2e9ee1aa42ac81056d4d`
- `data/pve/encounters.ron`: `a9d72eec286215b05c029e4583fe619ace088a505323749593d2b11f37082941`

The remaining `combat_setup` helper hashes were captured in the command output and are unchanged-status baseline
owners. No P02 production mutation has occurred yet.

## 2026-08-09 — M1 retained characterization

Four fully-qualified exact baseline tests ran before P02 production mutation:

- embedded source-set parse/count;
- preview wave → scenario spawn-time mapping;
- authored ProtectUnit tactical-plan override;
- survival timer → protected timer objective.

Each command executed exactly one test and passed: aggregate `4 passed`, `0 failed`, `0 ignored`; each run reported
`875 filtered out`. These fixtures establish the current inventory/parser and representative no-binding scenario
semantics. They do not prove the new Boss foundation yet.

Additional discovered owner: `events/combat.rs` (already dirty), SHA-256
`7b70c36582e44af0ee3842367c3ace9b508c841cb575edc3a5bdcbd10bbca936`. `BattleCore` retains the complete
`BattleScenario`, so an internal resolved Boss setup carried by the scenario can survive Core construction without a
second raw catalog lookup; no public DTO or `ActiveBattleSession` field is required in P02.

## 2026-08-09 — M2 first compiler probe

The first focused Boss compiler test command did not run tests because compilation failed with one local type error:
`task_result_set` returned fixed-array iterators of lengths 2, 3 and 4 from different match arms (`E0308`). The same
probe reported one unused `BTreeMap` import. This is a new-module implementation correction, not validation evidence;
no existing owner failed. Replace the helper with per-arm collected sets and remove the unused import before rerun.

The corrected compiler module then passed all four focused tests (`4 passed`, `0 failed`, `0 ignored`, `876
filtered out`). A subsequent first inventory-test compile exposed the expected migration surface of the new optional
`PveEncounter::boss_runtime` field: 34 existing test fixture literals plus one inventory test helper needed explicit
`None`/non-Debug error matching. No runtime assertion ran in that attempt. All fixture additions are neutral
no-binding values and the inventory test now uses explicit result matching.

## 2026-08-09 — M3 inventory probes and integration compile retry

- The exact physical gameplay-source inventory test passed: `1 passed`, `0 failed`, `0 ignored`, `881 filtered
  out`. The single macro-owned inventory now contains 23 gameplay RON paths plus `bundle.ron`.
- The empty Boss source rejection test passed with the same exact-test counts.
- The embedded production source parse/count test passed with the same exact-test counts.
- A subsequent library `--no-run` integration compile produced output beyond the available capture limit. Because
  its final exit state and diagnostics were truncated, it is deliberately recorded as **unverified**, not as a pass
  or failure. It will be repeated with quiet output before any M3/M4 conclusion is made.
- The quiet retry failed compilation with exactly 14 `E0063` diagnostics: library-test `BattleScenario` literals
  lacked the newly added optional `boss_runtime` field. No test binary ran. Those fixtures are being migrated with
  the neutral value `None`; the three integration-test literals found by the same owner search receive the same
  compatibility value before the next compile.
- The library-test compile then passed. The broader integration-test compile found three additional shared
  `PveEncounter` fixture literals in `tests/common/mod.rs`; all five integration targets failed at that same shared
  compile site and did not run. These are likewise migrated with `boss_runtime: None` before retry.
- The integration-test compile passed after the three neutral fixture migrations.

## 2026-08-09 — M4 typed setup aggregate implementation

- Added one private `ResolvedCombatSetupPlan` on the production world-start path. It resolves the encounter and
  preview contract, mission rewards and multiplier, battlefield/player/enemy/defense setup, optional Boss actor
  binding, final scenario validation, `BattleCore` and initial execution before the existing world commit receives
  anything. World-owned attempt/UUID/entry-snapshot/session lifecycle remains outside the aggregate.
- The aggregate uses a resolved-preview battlefield constructor and an encounter-taking scenario constructor, so
  its path does not repeat raw encounter lookup. `BattleScenario` carries compiled Boss definition `Arc`s and
  immutable per-call argument frames; no raw Boss catalog or argument map is placed in battle runtime state.
- `cargo check -q -p game_core --lib` passed, followed by `cargo test -q -p game_core --lib --no-run` passing. These
  are compile gates only; dedicated setup behavior and atomic failure tests remain required before M4 closes.
- The first attempt to run three newly added atomic-candidate tests passed multiple positional test filters to
  `cargo test`; Cargo rejected the command-line shape before compilation or test execution. This is a runner
  invocation error, not product evidence. The tests are rerun below with valid filters.
- A first compile of the added cross-phase cooldown/phase-cycle test failed with one test-only `E0502`: the fixture
  cloned phase bindings inside the same expression that mutably pushed a phase. No production code or assertion ran;
  the clone is staged before the push and the exact test is rerun.
- The first compile of the remaining schema/reference matrix test hit the same test-only borrow pattern while
  appending a cloned argument (`E0502`). No product assertion ran. The argument clone is staged first and the test is
  rerun; this does not change compiler behavior.
- The first no-binding Boss-class aggregate parity run failed during fixture database construction because its
  primary abnormality used the shared helper's `Elite` threat class, while the existing cross-catalog contract
  correctly requires `Boss`. No setup assertion ran. The fixture is corrected to `Boss` and rerun.

## 2026-08-09 — M2/M4 focused evidence and formatting checkpoint

- Boss compiler matrix: `4 passed`, then the added cross-phase cooldown/phase-cycle exact case `1 passed`, and the
  remaining ID/reference/outcome/variant/argument matrix exact case `1 passed`; all had zero failed/ignored. The
  compiler correction also treats loop exhaustion targets as unguarded paths, preventing an exhausted edge from
  hiding a second unbounded cycle.
- Boss setup resolver: `3 passed`, `0 failed`, `0 ignored`, `882 filtered out`. This covers no binding, immutable
  compiled definition/frame retention after database drop, valid actor binding, and invalid route/index rejection.
- Atomic candidate validation: `2 passed`, `0 failed`, `0 ignored`, `886 filtered out` for compiler and dangling PvE
  reference rejection; strict Boss unknown-field parse separately passed `1/0/0/887`.
- Typed aggregate: Boss-class no-binding parity passed `1/0/0/890`; deterministic bound actor identity passed
  `1/0/0/892`; late setup failure full world/session snapshot preservation passed `1/0/0/891`.
- The first `cargo fmt --all -- --check` failed only with rustfmt diffs across the newly added P02 code/test hunks; no
  compiler/test assertion ran. `cargo fmt --all` was applied as a mechanical formatting correction. A fresh check is
  still required for evidence.

## 2026-08-09 — focused closure before broad gates

- Fresh `cargo fmt --all -- --check`: pass.
- Full Boss compiler module: `6 passed`, `0 failed`, `0 ignored`, `887 filtered out`.
- Post-format embedded parse, exact physical inventory and empty-source rejection each passed exactly one test with
  `892 filtered out`.
- Representative no-binding paths each passed exactly one test with `892 filtered out`: normal preview→battle
  layout, authored ProtectUnit tactical override, survival timer objective and Event-started combat victory.
- Previously recorded focused P02 additions also remain green: Boss-class no-binding aggregate parity, bound actor
  deterministic identity, resolver invalid route/index, strict/atomic candidate failures and late setup full-state
  no-mutation. Broad gates now start; focused results do not substitute for them.

## 2026-08-09 — broad gates

- `cargo test -p game_core --lib`: `893 passed`, `0 failed`, `0 ignored`, `0 filtered out`.
- `cargo test -p game_core --test ron_loading`: `24 passed`, `0 failed`, `0 ignored`, `0 filtered out`.
- `cargo test -p game_server`: library `56 passed`, `0 failed`, `0 ignored`; main and doc-test targets each ran
  zero tests and passed.
- The first direct capture of `cargo test --workspace --all-targets` exceeded the model-output window while the same
  live process was still running. That truncated capture was not counted as a result. Polling that exact process
  (`session_id 45984`) returned authoritative exit `0`: Core `893`; integration targets `3 + 4 + 24 + 13 + 4`;
  server `56`; all binary/example zero-test targets passed. No inferred or replacement result is used.
- `git diff --check`: pass.

## 2026-08-09 — completion-boundary dirty audit

- The precreation baseline remains the exact sorted 235-entry status snapshot at
  `773eb0975112655528635ff7fcb6e38b0e690ded945ba4d86127f06bc704c3cd`.
- The completion-boundary status has 250 entries and sorted SHA-256
  `25c9f488727456c4081ec6b668005a23db790d0985c9c3178e00513413d5954b`.
- The 15-entry delta is exactly the expected P02 surface: nine previously clean fixture/setup/data owners, the three
  P02 Goal artifacts, the two new Boss source modules and the new embedded Boss RON source. There is no removal or
  unexpected path in the baseline comparison.
- Workspace validation did not leave `game_server/logs/app.log.2026-08-09` or another newly visible generated status
  entry. No user-owned path was reset, restored, stashed, deleted or cleaned.

## 2026-08-09 — exit audit interpretation

- BP-26 deadline arithmetic remains a P03 admission/runtime responsibility: P02 provides positive, bounded typed
  `DurationMs` signatures and arguments, while P03 must form the absolute battle-time deadline with checked addition
  and reject overflow atomically. Adding a guessed global duration cap to the standalone catalog compiler would
  duplicate the run-policy/runtime admission authority and is therefore not a P02 correction.
- Target-query eligibility is explicitly authored as a closed filter list, including `ActiveHostile` in the reference
  fixture; P02 does not infer hidden eligibility filters. P03 owns evaluation and immediate invalidation semantics.
- No exit condition requires scheduler, task leaf execution, public Wire, official live opt-in or a full refactor
  catalog cutover. Those surfaces remain assigned to P03/P04/P05/P06 or independent Goals.
