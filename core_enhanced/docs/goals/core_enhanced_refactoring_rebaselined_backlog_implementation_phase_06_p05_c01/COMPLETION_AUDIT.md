# Phase 06 CRA-P05-C01 Completion Audit

Audit Mode: phase_exit
Audit Target: Phase 06 — CRA-P05-C01 Keep Serialized Preview Cache Out Of Setup Authority
Current Status At Audit Start: active
Implementation Complete Claim: true
Policy Source: CRA-POL-006; Master Phase 06 registry; Final exact linear order row 6
Goal Documents: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
Repository/Commit/Dirty State: `/mnt/f/work/simulator` at
`f318414fb0156a67e04c5479000c2a2683fc0c09`; expanded status remained 399 entries across implementation

## Completion Evidence Matrix

| ID | Completion condition or invariant | Policy basis | Runtime/data/contract evidence | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- |
| P06-C01 | current cache, persistence and setup authorities are identified | Master Phase 06; CRA-POL-006 | `world/combat.rs`, `world/node_flow.rs`, `world/active_run.rs`, `world/state.rs`, `combat_preview/types.rs`, `combat_setup/enemy_spawns.rs` | static producer/consumer inventory | pass |
| P06-C02 | dirty user work is preserved and Phase hunks are separable | Goal dirty strategy | pre-status 399 entries and target fingerprints; post-status still 399; no reset/restore/clean/stash or broad formatter | status, hashes, scoped diff and repository diff check | pass |
| P06-C03 | real durable sequence reproduces cache authority before correction | Final order row 6 | new world test uses preview, cancel, completed SavePoint, V3 export, valid cache mutation, fresh Continue and actual start | pre-correction exact test: 1 selected, failed on `100` cached scale versus authoritative `110` | pass |
| P06-C04 | battle setup recomputes from authoritative run/live inputs | CRA-POL-006; combat preview canonical | battle-start `explicit_combat_preview_for_node` has no cache read and regenerates before `ResolvedCombatSetupPlan`; commit stores the fresh plan | exact durable regression: 1/1; active preview, enemy identity and runtime stats equal control | pass |
| P06-C05 | deterministic ordinary/Boss/Event/Monolith behavior remains exact | existing typed setup and manifested-difficulty policy | ordinary/Boss/Monolith retain the same generator/difficulty path; Event retry regenerates from authored encounter and explicit combat category | Boss 1/1, Event retry 1/1, Monolith checkpoint 1/1, preview 27/27, setup 11/11, full Core 941/941 | pass |
| P06-C06 | V3, RON, DTO, wire and public compatibility are unchanged | CRA-POL-006 | V3 remains version 3 with `combat_previews`; `enemy_stat_scale` remains `serde(skip)`; no serializer/server/public type changed | checkpoint 4/4, live RON 62/62, server active-run 16/16, static field inventory | pass |
| P06-C07 | focused and affected broad validation passes | completion review guide | Core and server compile against the unchanged cross-component contract | focused families, Core 941/941, workspace all-target check, format and diff checks pass | pass |
| P06-C08 | independent Phase-exit review is Complete / High | completion review guide | this audit | all rows pass; no material finding | pass |

## Runtime And State Evidence

- Node selection still calls `combat_preview_for_node`, which may reuse or publish `RunState::combat_previews` for
  presentation. Cancellation clears the node session but deliberately retains the cache.
- Battle admission reaches `plan_explicit_combat_node_with_state`, whose first setup input now comes from
  `explicit_combat_preview_for_node`. That resolver reads current run node identity/difficulty, encounter/category,
  live game data and the deterministic node seed; it has no persisted-cache read.
- The regenerated preview enters the existing `CombatExecutor::resolve_live_combat_setup_plan`. No parallel setup
  authority, cache repair or client-derived fallback was introduced.
- Commit remains ordered after complete planning: attempt/power state is updated, the fresh preview replaces the
  cache, the active session is installed, and battle time advances. Existing transaction/retreat snapshots remain
  unchanged.
- Event-started combat retains Event node/session ownership while passing its authored encounter and explicit combat
  category to the same deterministic regeneration path. A retry test corrupts the retained cache and still starts
  `defense_risk_abno`.

## Data Evidence

- No production RON, schema, balance tuning, encounter, difficulty or map data changed.
- Live candidate validation remains green at 62/62, including manifested difficulty preview/setup feasibility and
  Boss binding checks.
- The mutated test cache names another valid live fixture abnormality. Continue acceptance therefore demonstrates
  cache non-authority rather than relying on invalid-data rejection or repair.

## DTO, Transport And Persistence Evidence

- `CombatPreview` retains every serialized field and keeps internal `enemy_stat_scale` skipped/defaulted.
- `RunCheckpointPayload::combat_previews`, SavePoint capture, restore and strict `ActiveRunCorePayloadV3` remain
  present and unchanged; the active-run payload version stays 3.
- The exact regression crosses real `serde_json` V3 bytes and `continue_from_active_run_checkpoint` in two fresh
  Core instances. Both the unmodified skipped-field drift and an explicit serialized wave mutation are neutralized
  only at battle setup, not repaired during Continue.
- `game_server` has no preview mapping of its own. Its 16 active-run tests pass and the workspace all-target check
  compiles Core and server together.

## Consumer Evidence

- Confirmed workspace consumers: Core world node confirmation, battle start/setup, SavePoint/Continue and
  game_server active-run persistence.
- Canonical external consumers: Unity contracts referenced by `core_runtime_contract.md`. They continue receiving
  the same preview/result/update shapes; no external edit or migration is required.
- Inaccessible/unknown consumers: standalone deployed revisions remain unknown, but this Phase removes no field,
  type, symbol or serialized value and therefore does not require their availability for CRA-POL-006 compliance.
- Explicitly excluded: public-helper/resolver cleanup and external transport consolidation remain later Master
  candidates.

## Legacy And Compatibility Audit

- The retained V3 cache is approved compatibility, not gameplay authority. It remains covered as a presentation
  cache by the durable regression.
- The only `combat_previews.get` in the battle world is in the presentation resolver. The battle-start resolver has
  one caller and no cache/fallback branch.
- No dual version, alias, migration, repair, silent invalid-graph recovery, ignored test or debug bypass was added.
- Duplicate mapping/resolver and ghost Floor cleanup remain explicitly assigned to CRA-P05-C03; mission-wrapper
  cleanup remains CRA-P05-C05. They are not hidden prerequisites for this correction.

## Scoped Diff And Dirty Preservation

- The tree was extensively dirty before activation. Prospective source/test/canonical targets were fingerprinted
  before Phase mutation; the source and test already contained unrelated user changes.
- Phase production change is one separable early-return removal/comment in `world/combat.rs`, one exact durable test
  plus an Event retry assertion in `world/tests/combat.rs`, and one nearest-canonical clarification.
- Post hashes are recorded in `EXPERIMENTS.md`. Expanded status stayed at 399 entries, unrelated untracked files were
  not removed or moved, `cargo fmt` was not run broadly, and repository `git diff --check` passes.

## Canonical Alignment

- `core_runtime_contract.md` now matches the implemented boundary: cached previews survive node confirmation and V3,
  but battle start regenerates preview-derived setup input from current authoritative sources for ordinary, Boss,
  Event-started and Monolith combat.
- The canonical retains skipped internal scale and external preview-shape statements. No Goal document is the sole
  owner of the implemented contract.

## Validation Results

- Baseline `cargo test --lib game::combat_preview::tests`: 27 passed, 0 failed.
- Baseline/final `cargo test --lib game::combat_setup`: 11 passed, 0 failed.
- Baseline/final `cargo test --lib game::world::tests::checkpoint_validation`: 4 passed, 0 failed.
- Exact durable test before correction: 1 selected, 0 passed, 1 failed at authoritative Hard scale equality.
- Exact durable test after correction: 1 passed, 0 failed.
- Exact Boss guard: 1 passed, 0 failed.
- Exact strengthened Event retry guard: 1 passed, 0 failed.
- Exact typed Monolith stage guard: 1 passed, 0 failed.
- `cargo test --lib game::data::live_ron::tests`: 62 passed, 0 failed.
- `cargo test --lib game::world::tests::facility`: 11 passed, 0 failed.
- `cargo test --lib game::world::tests::node_sessions`: 18 passed, 0 failed.
- Final `cargo test --lib`: 941 passed, 0 failed, 0 ignored, 0 filtered.
- `cargo test -p game_server active_run`: library 16 passed, 0 failed, 43 filtered; main binary selected 0 tests
  and is recorded separately rather than counted as proof.
- `cargo check --all-targets`: passed for `game_core` and `game_server`.
- `cargo fmt --check`: passed.
- `git diff --check`: passed.

## Baseline Failures

- No pre-existing focused or broad failure remains.
- The sole red execution was the newly added exact regression before production correction. It failed on the
  intended cache-authority defect and passed after the one production change.

## Cross-Component Findings

- None. Core V3 writer/reader, world setup and server host persistence continue to share the same version/shape.
- Phase 06 supplies the required cache-non-authority seam for later CRA-P05-C03 resolver consolidation without a
  temporary compatibility layer.

## Long-Term Direction Review

| Candidate ID | Work type | Current-to-target authority result | Preserved invariants | Removal/consolidation result | Validation | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| CRA-P05-C01 | correctness correction | persisted presentation cache no longer feeds authoritative setup | deterministic setup, normal/Boss/Event/Monolith paths and V3/public shape retained | one unsafe early return removed; existing typed plan retained | exact durable red/green plus Core/server broad suites | pass |

- Fit: High.
- Improvement class: none.
- Reason: the change removes a competing source of truth at the existing battle-start seam, creates no new state or
  abstraction, preserves approved compatibility, and protects the durable user-visible sequence rather than a
  private helper layout.

## Primary Verdict

- Complete.

## Status Transition

- from: active
- to: complete
- next Phase activation: allowed after Master records this `Complete / High` audit and marks Phase 07 ready

## Remaining Risk And Required Action

- No material Phase 06 finding remains and no policy question is open.
- Known resolver/mapping cleanup is non-blocking and remains owned by CRA-P05-C03 in its existing exact order.
- Phase 07 must initialize and fully read its own required documents before production inspection or mutation.
