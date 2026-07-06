# Run Floor Progression Contract Plan

## Objective

Implement the long-term run progression contract for Standard and Endless modes by replacing the current public Act-centric progression surface with a Floor-centric, mode-aware model.

This goal covers run progression, map generation terminal selection, Gate transition behavior, and Unity-facing snapshot/result DTO shape. It does not implement Endless abnormality research/response completion, boss omen chains, or new encounter weighting systems.

## Source Of Truth Order

Verify facts in this order during implementation:

1. Runtime code for `RunProgression`, Gate transitions, map generation, node completion, battle completion, and snapshots.
2. Live RON/data for run policy and map generation policy.
3. Unity-facing snapshot/command contracts and live WebSocket JSON.
4. Latest policy docs, especially `docs/game_rulebook.md` and `docs/endless_mode_research_policy_draft.md`.

Do not trust this plan over runtime behavior. If code reading shows a better long-term model, record the evidence in `EXPERIMENT_NOTES.md` and apply it unless it requires a user policy decision.

## Current Policy To Implement

- Long-term run progression terminology is `Floor`, not `Act`.
- `Standard` is a fixed 3-Floor mode.
- `Standard` Floor 1 and Floor 2 terminal nodes are `Gate`.
- `Standard` Floor 3 terminal node is the final boss.
- `Standard` Floor 3 final boss victory completes the run.
- `Standard` Floor 3 final boss defeat immediately fails the run.
- `Endless` has no `max_acts`/`max_floors` cap.
- `Endless` advances by entering `Gate` nodes.
- `Endless` Gate transition creates the next Floor map.
- `Endless` Gate transition itself never completes the run.
- `Endless` boss/elite encounters are not automatically run-ending.
- Gate is an independent node category, not a property attached to elite/boss nodes.
- Gate is not a save point.
- Gate transition applies the live RON configured trauma recovery to living employees only.
- Gate transition does not heal HP, save a checkpoint, or affect dead/missing/incapacitated employees.
- Unity handles the local confirm/cancel UX before sending the final Gate command. Core stores no separate Gate confirmation state.
- Balance values and floor setup values must be configurable from live RON where they are balancing policy, not permanent code constants.

## In Scope

- Audit current `RunProgression`, `RunProgressionSnapshotDto`, `MapViewDto`, `BehaviorResult::ActComplete`, Gate transition, and node completion flow.
- Replace or refactor Act-centric names in the core public contract:
  - `act_index`
  - `max_acts`
  - `current_act_seed`
  - `ActComplete`
  - `advance_act`
  - `is_final_standard_act`
- Introduce a Floor-centric runtime model with mode-aware state.
- Ensure `Standard` carries fixed floor count policy from live RON.
- Ensure `Endless` has no max floor cap.
- Ensure seed derivation is Floor-based and deterministic.
- Ensure map generation terminal category is mode-aware:
  - `Standard` final Floor -> `Boss`
  - `Standard` non-final Floor -> `Gate`
  - `Endless` default Floor -> `Gate`
- Ensure Gate transition advances to the next Floor for both Standard non-final Floors and Endless.
- Ensure Gate transition cannot accidentally complete an Endless run.
- Ensure Standard final boss completion still produces `RunComplete`.
- Ensure Standard final boss defeat still produces `RunFailed`.
- Ensure non-final and Endless boss defeat follows existing node-failure/retry policy instead of run failure.
- Update Unity-facing snapshot DTOs to expose Floor-centric fields.
- Update command/result DTOs so Gate/Floor advancement results no longer use Act-centric names.
- Update live RON schema if needed:
  - Standard floor count.
  - Gate trauma recovery remains data-driven.
- Update docs and probe docs after runtime shape is final.
- Add focused tests for:
  - Standard Floor 1/2 terminal Gate.
  - Standard Floor 3 terminal Boss.
  - Standard Gate transition advances Floor and applies trauma recovery.
  - Endless Gate transition advances Floor without max cap.
  - Endless Gate transition does not complete run.
  - Standard final boss defeat fails run.
  - Endless boss defeat does not use Standard final boss failure path.
  - Snapshot JSON shape for Standard and Endless `run_progression`.
  - Live RON loading for the new run policy fields.

## Out Of Scope

- Endless abnormality research/response completion state.
- `RunAbnormalityResearchState`.
- PveEncounter bonus objectives.
- Repeated abnormality encounter weighting.
- Boss omen chains.
- Provisional boss selection.
- Forced boss rooms.
- Multiple Gate candidates.
- Conditional, hidden, or locked Gate content.
- Unity implementation.
- Backward compatibility layers that keep old Act DTOs as official public fields.
- Long-lived dual schema where both Act and Floor fields are equally canonical.

## Target Runtime Model

Exact Rust names may change after code reading, but the public meaning should be:

```rust
RunProgression {
    run_seed: u64,
    game_mode: GameMode,
    mode_state: RunProgressionModeState,
}

RunProgressionModeState {
    Standard {
        floor_index: u8,
        max_floors: u8,
    },
    Endless {
        floor_index: u32,
    },
}
```

Allowed implementation variants:

- A single `floor_index` field plus `max_floors: Option<u8>` is acceptable only if it keeps `Endless` max-free and does not leak `max_acts`.
- If a simpler representation is chosen, record the reason in `EXPERIMENT_NOTES.md`.

## Target Snapshot DTO Shape

Standard:

```json
{
  "run_progression": {
    "game_mode": "Standard",
    "floor_index": 0,
    "max_floors": 3,
    "current_floor_seed": 123
  }
}
```

Endless:

```json
{
  "run_progression": {
    "game_mode": "Endless",
    "floor_index": 0,
    "current_floor_seed": 123
  }
}
```

Notes:

- `max_floors` must not be present for Endless unless implementation evidence shows an unavoidable serialization constraint. If that happens, it must be `null`, documented as transitional, and have a removal condition.
- `act_index`, `max_acts`, and `current_act_seed` must not remain as canonical public DTO fields.
- `MapViewDto` should not expose Act-centric fields. If map views need progression labels, use Floor-centric fields or rely on top-level `run_progression`.

## Target Result Contract

Gate transition should report Floor advancement, not Act completion.

Suggested result:

```json
{
  "result_type": "FloorAdvanced",
  "payload": {
    "game_mode": "Endless",
    "floor_index": 12,
    "map": {}
  }
}
```

Exact nesting may follow existing command result conventions, but the meaning must be Floor-based.

## Initial Code Reading Targets

Read before implementation:

- `src/game/map/progression.rs`
  - `RunProgression`, `GameMode`, terminal category, seed derivation, advancement.
- `src/game/world/node_flow.rs`
  - map generation, Gate transition, node completion, `ActComplete`/`RunComplete`.
- `src/game/world/combat.rs`
  - final boss failure/completion checks.
- `src/game/world/snapshot.rs`
  - `RunProgressionSnapshotDto` construction.
- `src/game/behavior.rs`
  - public snapshot/result DTOs and command result payloads.
- `src/game/map/types.rs`
  - `MapViewDto` Act fields.
- `src/game/world/map_encounters.rs`
  - difficulty offset currently using `act_index`.
- `src/game/data/run_policy_data.rs`
  - `default_max_acts`, Gate trauma recovery policy, validation.
- `../game_resources/data/run/policy.ron`
  - live run policy values.
- `../game_resources/data/map/generation_policy.ron`
  - generated map policy.
- Relevant tests under:
  - `src/game/world/tests/map_flow.rs`
  - `src/game/world/tests/snapshots_and_start.rs`
  - `src/game/world/tests/combat.rs`
  - `tests/ron_loading.rs`

## Implementation Sketch

1. Audit runtime and live data.
   - Record current Act-based fields and behavior in `EXPERIMENT_NOTES.md`.
   - Confirm current Gate transition behavior and Standard final boss behavior with focused tests or code reading.
2. Refactor progression domain.
   - Replace `act_index/max_acts` with Floor-centric mode-aware state.
   - Replace `current_act_seed()` with `current_floor_seed()`.
   - Replace `advance_act()` with `advance_floor()` or equivalent.
   - Preserve deterministic seed behavior for existing Standard Floor 1 maps unless a deliberate seed migration is documented.
3. Refactor terminal selection.
   - Standard final Floor uses Boss.
   - Standard non-final and Endless default Floor use Gate.
4. Refactor node/Gate flow.
   - Gate transition advances Floor and regenerates map.
   - Endless Gate transition never returns `RunComplete`.
   - Standard final boss still completes the run through node/battle completion.
5. Refactor snapshot/result DTOs.
   - Public JSON becomes Floor-centric.
   - Remove public Act fields rather than keeping a long compatibility layer.
6. Refactor encounter difficulty use of progression.
   - Replace `act_index` difficulty offset with Floor-aware policy.
   - Initial simple policy may use `floor_index` with a capped or mode-specific offset if supported by live RON/policy.
   - If this requires user balance decisions, stop and report the question.
7. Update live RON and validation.
   - Replace `default_max_acts` with Standard floor setup policy.
   - Keep Gate trauma recovery as existing data-driven value.
8. Update tests after behavior is implemented.
   - Prefer user-visible behavior, JSON shape, live RON loading, and gameplay flow tests.
9. Update docs/probes.
   - Update canonical docs and probe expectations only after the runtime contract is final.
10. Run focused checks after each small change, then broader checks at the end.

## Completion Criteria

- Core runtime no longer uses Act-centric public progression fields as the canonical model.
- `Standard` fixed 3-Floor behavior is tested.
- `Endless` can advance through Gate beyond the old max-act boundary.
- `Endless` snapshots do not expose a real max cap.
- Gate transition applies trauma recovery and does not save a checkpoint.
- Standard final boss win/loss semantics remain correct.
- Endless boss win/loss does not accidentally trigger Standard run-ending semantics.
- Unity-facing snapshot JSON uses `floor_index`/`current_floor_seed`.
- Live RON loading validates the new run policy shape.
- Legacy Act public DTO/result fields are removed or have a short, explicitly documented removal condition approved by the user.
- Relevant docs/probes are updated.
- Focused tests and final broad checks are run and recorded in `EXPERIMENTS.md`.

## Stop Conditions

Stop and report questions instead of deciding silently if implementation discovers:

- Unity cannot accept removal of Act DTO fields without a transitional schema.
- Save data migration must preserve existing live saves.
- Encounter difficulty scaling needs new balance rules beyond a mechanical Floor rename.
- Endless boss failure should have a special failure/retreat policy beyond current node-failure policy.
- Live RON lacks enough policy structure to express Standard and Endless setup cleanly.
- Any change would delete or replace user-authored content outside this goal.
