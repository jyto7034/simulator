# Phase 06 CRA-P05-C01 Experiments

Status: complete
Implementation Complete: true

## 2026-08-09 — E00 Activation And Required-Document Initialization

### Authorization And Dependency

- The user's Master execution request remains active and authorizes the single active Child.
- Phase 05 CRA-P04-C01 has a current `Complete / High` audit with improvement class `none`.
- Phase 06 is next in the exact registry; Phases 07~44 remain pending and unauthorized.
- CRA-POL-006 is the direct and global compatibility policy for this candidate.

### Evidence Carried Into Initialization

- CRA-P05-C01 remains a correctness correction with maximum known priority `3/3/3/3/3/3/unknown = 18/18`.
- Historical defect: V3 carries preview caches, a skipped/defaulted preview scale can drift on deserialize, and a
  world cache fast path/enemy setup can trust that presentation cache.
- Required first proof: preview→cancel→SavePoint→durable Continue→start with a mutated serialized cache must produce
  the same authoritative setup as an unmodified control after correction.
- Preservation boundary: V3/public/wire field shape, live balance, typed resolved setup plan and normal/Boss/Event/
  Monolith behavior.

### Initialization Result

- Required Child documents initialized: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`.
- Master Phase 06 coordination changes from `ready` to `active`; no other Child is active.
- Phase 06 production status/source/data investigation: not started.
- Test executions: none.
- Production/test/data/canonical mutation before initialization: none.

### Next Experiment

Read all three Child documents completely, capture expanded dirty status and prospective target hashes, then trace
the current preview/cache/V3/SavePoint/Continue/resolved-setup path and establish nonzero baselines.

## 2026-08-09 — E01 Dirty Baseline, Authority Trace And Focused Baseline

### Dirty Baseline

- Repository commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded status after Child initialization: 399 entries; status SHA-256
  `73566cb33b0767cb61a2a18bdf5625d9c3cd1b279b08de4651452ebb266c9c37`.
- Prospective overlapping targets were captured before Phase mutation. In particular,
  `src/game/world/combat.rs` was already modified with SHA-256 `ce43e9...`, and
  `src/game/world/tests/combat.rs` was already modified with SHA-256 `4cd00a...`; Phase work is limited to new,
  separable resolver/test hunks. No reset, restore, clean, stash or unrelated formatting was used.

### Current Authority Trace

- `RunState::combat_previews` is copied into `RunCheckpointPayload`, V3 JSON export and Continue restore unchanged.
- `CombatPreview::enemy_stat_scale` is `serde(skip)`, so durable deserialization changes a manifested Hard preview's
  `110/110/110` scale to `100/100/100` while retaining the rest of the cache.
- `combat_preview_for_node` owns presentation generation/cache publication and legitimately returns the cached UI
  preview at node selection.
- `explicit_combat_preview_for_node` is the battle-start seam and currently returns the same cache before consulting
  run-node identity, manifested difficulty or live data.
- `plan_explicit_combat_node_with_state` passes that returned preview into
  `CombatExecutor::resolve_live_combat_setup_plan`; `enemy_spawn_groups_from_preview` consumes both its serialized
  waves and skipped/defaulted scale. Commit then stores the planned preview in the active session/cache.
- Ordinary, Boss and Monolith nodes carry manifested difficulty. Event-started combat supplies its authored
  encounter/category and can be deterministically regenerated even though the map node retains Event category.

### Nonzero Baseline

| Command family | Result |
| --- | --- |
| combat preview unit family | 27 passed, 0 failed |
| combat setup module | 11 passed, 0 failed |
| checkpoint validation module | 4 passed, 0 failed |
| active-run durable round trip exact test | 1 passed, 0 failed |

## 2026-08-09 — E02 Exact Durable Red Proof

Added one exact production-sequence regression in `src/game/world/tests/combat.rs`:

1. generate a manifested Hard combat preview (`110/110/110`);
2. cancel node selection;
3. complete a real SavePoint and export V3;
4. keep an unmodified control and mutate only the serialized cached wave's valid abnormality id in the second V3
   document;
5. Continue each document into a fresh `GameCore` and start the target battle.

The pre-correction test selected exactly 1 test and failed at the control battle-start assertion: the active setup
used the deserialized cache's default `100/100/100` instead of the authoritative Hard preview's `110/110/110`.
The mutation case additionally remains capable of substituting `low_risk_abno` for the live encounter's
`defense_risk_abno`. This isolates cache authority through the real preview→cancel→SavePoint→V3 Continue→start
boundary; it is not a serialization-only unit proof.

### Next Experiment

Remove only the battle-start cache fast path, retain presentation-cache publication and all V3/public fields, then
rerun the exact proof and normal/Boss/Event/Monolith parity families.

## 2026-08-09 — E03 Narrow Authority Correction And Focused Green Proof

### Production Correction

- Removed only the `combat_previews.get(node_id)` early return from
  `GameCore::explicit_combat_preview_for_node`.
- The battle-start resolver now always regenerates from current run node, passed encounter/category, manifested
  difficulty, live data and deterministic node seed before entering the existing typed setup plan.
- `combat_preview_for_node` still owns UI cache reuse/publication; checkpoint capture/restore and V3 shape are
  untouched; the active battle/cache still receives the newly planned authoritative preview at commit.

### Focused Result

- Exact durable regression: 1/1 passed after failing before correction.
- Boss ordinary-runtime/public-shape guard: 1/1 passed.
- Event-started defeat/retry guard: 1/1 passed. The guard now mutates the retained Event presentation cache before
  retry and proves the authored encounter is regenerated.
- Typed Monolith stage checkpoint/Continue/rollback guard: 1/1 passed.
- Combat preview: 27/27; combat setup: 11/11; checkpoint validation: 4/4.
- Formatting and scoped whitespace checks: pass.

## 2026-08-09 — E04 Broad, Compatibility And Static Validation

| Validation | Result |
| --- | --- |
| full Core library | 941 passed, 0 failed, 0 ignored, 0 filtered |
| live RON validation | 62 passed, 0 failed |
| facility/SavePoint world family | 11 passed, 0 failed |
| node-session/Monolith family | 18 passed, 0 failed |
| server active-run boundary | 16 passed, 0 failed; zero-test main binary reported separately |
| workspace all-target check | pass for `game_core` and `game_server` |
| `cargo fmt --check` | pass |
| repository `git diff --check` | pass |

### Static Compatibility Audit

- Cache field inventory remains present in `RunState` and `RunCheckpointPayload`, with the same checkpoint copy/
  restore sites and V3 version 3 envelope.
- `CombatPreview::enemy_stat_scale` remains skipped; no RON, DTO, wire, result, public helper or server shape changed.
- The only remaining world cache read is the presentation resolver at node selection. The battle-start resolver has
  one caller and no cache read; enemy setup consumes only the freshly resolved preview.
- Post-change hashes: `world/combat.rs`
  `aa08c5f088350915b8adcf87ffe349d26030a613af43763dd5ebd90c3a0e6f5e`, `world/tests/combat.rs`
  `9bebdf426aa9a3e56ea6413d0806ea6a4a9366874f616e995a1470abd921574c`, canonical
  `b9f00eba0d93acee63090cac03e65935a253adf3bf17aa571155329afecf2621`.
- Expanded status remains 399 entries after implementation, so no unrelated path was added/removed by the Phase.
- The nearest canonical now states the implemented deterministic regeneration rule while retaining presentation
  cache compatibility.

### Exit Readiness

Implementation and validation conditions P06-C01~C07 pass. Read the completion-review guide in full and perform the
independent Phase-exit audit; Phase 07 remains unauthorized until `Complete / High`.
