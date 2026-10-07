# Game Rulebook Policy Implementation Master Phase 02 — Experiments

## 2026-08-12 — Phase Activation And Document Initialization

### Hypothesis

P01 `Complete / High` closes the only production prerequisite, so the user's Master execution request permits P02
activation as the only active Phase after its required documents are initialized.

### Performed work

- Validated P01 required documents, complete/true status, `Complete / High` audit and Master handoff.
- Read P01 completion audit as the direct production dependency.
- Confirmed P02 child path does not exist.
- Captured scoped Core/live-data/server status showing broad pre-existing dirty ownership.
- Created P02's three required documents and changed Master registry from P02 ready to P02 active.

### Exact commands

```text
validate P01 PLAN/EXPERIMENTS/EXPERIMENT_NOTES/COMPLETION_AUDIT and Master registry
sed -n '1,260p' <P01>/COMPLETION_AUDIT.md
test ! -e <P02 child directory>
git status --short -- core_enhanced/src/game game_resources/data game_server/src
mkdir -p <P02 child directory>
```

### Result

- P01 dependency: complete/true, audit `Complete / High`.
- P02 path collision: none.
- Core/live-data/server scope: broadly dirty before P02, including relevant world/data/store paths.
- Tests/probes: 0; pass/fail/skip 0/0/0 in this activation unit.
- Production changes: 0; Child/Master documents only.

### Interpretation

P02 is active, but production mutation is still gated by exact ownership classification, current baseline,
bootstrap referential validation and current authority mapping.

## 2026-08-12 — Activation Baseline And Bootstrap Revalidation

### Hypothesis

P02 may proceed with target-first work if current Core/server runners remain green and all approved bootstrap IDs
exist, while missing target fields and live research are recorded as implementation gaps rather than blockers.

### Performed work

- Reused the immediately preceding P01 final full Core run because no production code changed between P01 exit and
  P02 activation.
- Executed the current full game_server baseline.
- Re-ran the exact 29-row encounter/abnormality/fragment reference validator.
- Located current target source-field absence, research live-data hits and sole WhiteNight chain.

### Exact commands

```text
cargo test -p game_server -- --quiet
extract 29 manifest rows from CONTENT_INPUT_CANDIDATES.md and exact-search production encounter/abnormality/fragment catalogs
rg -c 'reward_source_id|fragment_id|repeat_reward' game_resources/data/pve/encounters.ron
rg -c 'suppression_research|GrantResearch|research' game_resources/data/pve/encounters.ron game_resources/data/events/rewards/base.ron game_resources/data/run/policy.ron
rg -n 'white_night|BossOmenChain' game_resources/data core_enhanced/src/game/data
```

### Result

- Core activation baseline: P01 final current tree, 1065 passed, 0 failed, 0 ignored.
- game_server: 63 passed, 0 failed, 0 ignored; main/doc tests 0.
- Manifest: 29 rows; missing encounter/abnormality/fragment 0/0/0.
- Explicit target source fields in production encounters: 0.
- Research remains in run policy and reward live data.
- Live BossOmen chain count: 1, `white_night`.
- Production changes in this unit: 0.

### Interpretation

Runner and content prerequisites are green. Source fields and research gaps are the approved P02 target. The next
gate is exact current authority and dirty-hunk classification, not a policy rewind.

## 2026-08-12 — Authority Inventory And Atomic Cutover

### Hypothesis

Research/pending delivery, encounter cycles and active-run V3 can be replaced without a parallel authority if one
`EncounterProgressionState` is embedded in `RunState` and the Core/server reader and writer change to V4 together.

### Performed work

- Removed `abnormality_research`, PVE bonus-objective and pending research-delivery modules, state, DTO/result,
  reward effects, live RON and fixture generator paths.
- Added the strict encounter source schema and one progression graph for completed source IDs, mastered Elite and
  NormalBoss cycles, Monolith and BossOmen.
- Replaced Core Active Run payload and game_server envelope with exact version 4 readers/writers; retained no
  V2/V3 transform or dual reader.
- Made inventory, fragment and checkpoint bodies reject unknown removed fields.

### Result

- Production source/data search for removed research authority: zero; remaining hits are strict rejection tests.
- Core/server compile and their library suites passed during each bounded unit.
- Production mutation stayed within Core/live RON/game_server; Unity was not inspected or changed.

### Interpretation

Run-local encounter progression now has one serializable Core owner. The server stores opaque strict V4 Core bytes
plus monotonic host metadata and does not infer cycle or reward state.

## 2026-08-12 — Exact Bootstrap Sources And Cycle Runtime

### Hypothesis

The approved minimum content can exercise the generic progression graph with exactly 29 source rows and one live
WhiteNight chain while leaving final content design outside this Phase.

### Performed work

- Authored 13 Elite, 13 NormalBoss and 3 FinalBoss source rows with exact encounter/fragment references and
  5/10/20 fragment-dust repeat rewards.
- Added uniqueness, class, cross-catalog fragment and ordinary Reward exclusion validation.
- Implemented unmastered-first Elite assignment, deterministic mastered shortage/refill, NormalBoss reserve/
  consume/release/refill and ordinary-combat downgrade when no distinct Elite candidate remains.
- Kept natural Elite at victory count 1, Monolith Stage 2/3 at counts 2/3 and made exhausted remanifestation failure
  run-local permanent without reward/mastery.

### Exact commands and results

```text
rg -c '^\s*PveEncounterRewardSource\(' game_resources/data/pve/encounters.ron
=> 29
rg -c '^\s*repeat_reward: GrantFragmentDust\(amount: 5\)' ... => 13
rg -c '^\s*repeat_reward: GrantFragmentDust\(amount: 10\)' ... => 13
rg -c '^\s*repeat_reward: GrantFragmentDust\(amount: 20\)' ... => 3
cargo test -p game_core encounter_progression --lib -- --quiet
=> 6 passed, 0 failed, 0 ignored
cargo test -p game_core monolith --lib -- --quiet
=> 22 passed, 0 failed, 0 ignored
```

### Interpretation

The live fixture is deliberately a bootstrap, not final content. Strict validation fixes identity/class/repeat
meaning while future balance/content can replace rows without changing runtime ownership.

## 2026-08-12 — BossOmen DAG And FinalBoss Lifecycle

### Hypothesis

Per-chain conditions and an authored dependency DAG can replace the global awakened-fragment gate while the sole
live one-step WhiteNight chain proves the complete provisional→active→capstone→Gate path.

### Performed work

- Added optional exact fragment/awakening conditions and dependencies to strict BossOmen RON.
- Added unknown/self/cycle/unreachable validation and removed global eligibility fallback.
- Implemented provisional failure requeue, active-step retention, cycle consumption/refill and forced FinalBoss
  completion that reveals the preauthored Gate without advancing the Floor.
- Verified defeat uses common run-failure/checkpoint semantics and retreat restores pre-entry state.

### Exact commands and results

```text
rg -c '^\s*BossOmenChain\(' game_resources/data/boss_omen/chains.ron
=> 1
cargo test -p game_core boss_omen --lib -- --quiet
=> 15 passed, 0 failed, 0 ignored
```

### Interpretation

WhiteNight is a development bootstrap only. Generic multi-chain and dependency behavior is protected with
synthetic fixtures without publishing dormant FinalBoss membership.

## 2026-08-12 — Integration-Test Legacy Correction

### Hypothesis

The exact broad package command may expose old integration contracts that library-only runs cannot see.

### Performed work

- Ran `cargo test -p game_core --no-run` and the production RON integration binary.
- Removed stale fixture fields and assertions for direct fragment/research Reward IDs.
- Replaced them with exact encounter-source resolution and ordinary Reward bypass exclusion assertions.

### Red and green evidence

```text
cargo test -p game_core --test ron_loading -- --quiet
=> initial: 24 passed, 1 failed, 0 ignored
   failure: stale freischutz_black_round_fragment_reward assertion
=> final:   25 passed, 0 failed, 0 ignored
```

### Interpretation

The package integration suite now exercises the production source-ledger contract instead of preserving removed
research/direct-source rewards.

## 2026-08-12 — Source Reward/Floor-Advance Atomicity Correction

### Hypothesis

A user-visible direct NormalBoss victory test should prove fragment/repeat reward, source ledger and cycle cursor
survive the same terminal Floor transition.

### Performed work

- Added direct first and repeat NormalBoss victory tests and a repeat-reward overflow rollback test.
- The target-first run showed fragment grant succeeded but the source ledger disappeared after Floor advance.
- Traced the defect to `plan_complete_current_node`: it embedded a pre-victory encounter progression snapshot in
  `FloorAdvanced`, overwriting the post-victory ledger during commit.
- Added an explicit encounter-progression planning input so terminal map generation and completion use the same
  post-victory graph; retained the outer Core transaction rollback.
- Extended the forced FinalBoss test through dedicated Gate confirmation to prove fragment and source ledger remain.

### Red and green evidence

```text
cargo test -p game_core normal_boss_ --lib -- --nocapture
=> red: 2 passed, 1 failed; first fragment existed but source_boss_risk ledger was absent
=> green after correction: 3 passed, 0 failed, 0 ignored
cargo test -p game_core forced_boss_omen_victory_opens_dedicated_gate_before_endless_floor_advance --lib -- --nocapture
=> 1 passed, 0 failed, 0 ignored
```

### Interpretation

Reward inventory, source completion, cursor and Floor transition now share one authoritative post-victory graph.
The overflow test proves a failed repeat effect restores the exact CombatResult/cycle/ledger/inventory state.

## 2026-08-12 — V4, Persistence And Final Broad Validation

### Performed work

- Verified the exact 18-field V4 checkpoint document, semantic round trip, one-edge corruption rejection,
  V2/V3 and removed-shape rejection before mutation.
- Verified game_server V4 envelope/checksum/current-backup, monotonic load/peak merge, write failure rollback and
  reconnect notification idempotence.
- Updated gameplay, runtime and data canonical documents from V3/research language to the current V4/source model.
- Ran focused and exact broad package commands on the final P02 candidate tree.

### Exact commands and results

```text
cargo test -p game_core current_v4 --lib -- --quiet
=> 7 passed, 0 failed, 0 ignored
cargo test -p game_core live_ron --lib -- --quiet
=> 75 passed, 0 failed, 0 ignored
cargo test -p game_server active_run --lib -- --quiet
=> 16 passed, 0 failed, 0 ignored
cargo test -p game_core -- --quiet
=> unit 1004 + integration 4 + 4 + 25 + 14 + 4 = 1055 passed; 0 failed; 0 ignored; doc 0
cargo test -p game_server -- --quiet
=> library 63 passed, 0 failed, 0 ignored; main/doc 0
cargo fmt --all -- --check
=> pass
scoped git diff --check
=> pass
```

### Interpretation

P02's supported boundary is fully runnable as Core + production live RON + game_server V4. Unity remains an
explicitly unverified development consumer and no compatibility shim was introduced for it.
