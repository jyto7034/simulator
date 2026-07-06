# Run Floor Progression Contract Notes

## Initial Runtime Observations

- `src/game/map/progression.rs` currently defines `RunProgression { run_seed, game_mode, act_index, max_acts }`.
- `RunProgression::terminal_node_category()` already distinguishes:
  - Standard final act -> `Boss`;
  - Standard non-final and Endless -> `Gate`.
- `RunProgression::advance_act()` still blocks when `act_index + 1 >= max_acts`, which conflicts with Endless having no max Floor cap.
- `RunProgression::current_act_seed()` is used for map generation and encounter selection. It needs a Floor-based replacement.
- `src/game/world/node_flow.rs` currently uses `generate_current_act_map()`, `advance_act()`, and `BehaviorResult::ActComplete` for Gate transitions.
- Gate transition already applies `StagedSupportEffect::GateTransition`.
- Gate trauma recovery is already live-RON driven through `run_policy.support.gate_transition_trauma_recovery_percent`.
- `src/game/world/support.rs` applies Gate trauma recovery to living employees and does not heal HP.
- `src/game/world/combat.rs` uses `run.run_progression.is_final_standard_act()` for final boss run failure semantics.
- `src/game/world/snapshot.rs` currently builds `RunProgressionSnapshotDto` with `act_index`, `max_acts`, and `current_act_seed`.
- `src/game/behavior.rs` exposes `RunProgressionSnapshotDto`, `MapViewDto`, and command result payloads with Act-centric names.
- `src/game/world/map_encounters.rs` uses `run_progression.act_index` to scale encounter difficulty.
- Live RON currently has `setup.default_max_acts: 3` in `../game_resources/data/run/policy.ron`.
- Map generation policy is already data-driven in `../game_resources/data/map/generation_policy.ron`, but not mode-specific yet.

## Long-Term Direction

Prefer a mode-aware progression model instead of forcing Endless into `max_acts`.

Preferred shape:

```rust
RunProgression {
    run_seed: u64,
    game_mode: GameMode,
    mode_state: RunProgressionModeState,
}

RunProgressionModeState {
    Standard { floor_index: u8, max_floors: u8 },
    Endless { floor_index: u32 },
}
```

This prevents `Endless` from carrying meaningless max-cap data and makes Standard final-Floor logic explicit.

## Risk Notes

- DTO change is Unity-facing. Removing `act_index/max_acts/current_act_seed` can break Unity if it still parses those fields.
- If Unity needs a transition window, document it in `PLAN.md` with an explicit removal condition and ask the user before implementing a dual schema.
- Encounter difficulty scaling currently uses `act_index`. A pure rename to `floor_index` may change Endless scaling semantics. If a non-mechanical balance decision is needed, stop and ask.
- Save/checkpoint payload stores `RunProgression`. If current save data compatibility matters, ask before designing migration.
- Existing tests and docs may still use Act names. Update tests to assert the new behavior, not just renamed fields.
- There may be pre-existing dirty worktree changes. Do not revert unrelated changes.

## Out-Of-Scope Follow-Up Candidates

- Endless abnormality research and response completion state.
- Repeated abnormality encounter weighting using recent Floor and response-complete status.
- Boss omen/provisional boss chains.
- Mode-specific map generation policy beyond terminal category and end conditions.
- Multiple Gate candidates per Floor.
- Conditional/locked/secret Gate data.
- Account-wide abnormality encyclopedia progress.

## Policy Questions To Escalate If Found

- Should `Standard` keep exposing `max_floors` to Unity, or should Unity infer Standard has 3 Floors from mode data?
- Does Unity require a short transitional DTO with old Act fields?
- Should encounter difficulty scaling in Endless be linear by Floor, capped, or data-authored per Floor band?
- Should existing checkpoint/save payloads be migrated, or can this project discard old save compatibility?
- Should `ActComplete` result type be renamed to `FloorAdvanced`, or should result type names be preserved until Unity catches up?

## Implementation Resolution

- Chose the mode-aware progression model from the long-term direction:
  - `Standard { floor_index: u8, max_floors: u8 }`
  - `Endless { floor_index: u32 }`
- Public runtime/snapshot/result fields now use Floor terminology:
  - `floor_index`
  - `max_floors` only when applicable
  - `current_floor_seed`
  - `FloorAdvanced`
- Removed public `MapViewDto` Act fields instead of keeping a compatibility layer.
- `Endless` no longer carries a max cap in the core progression model or snapshot DTO.
- `Standard` continues to expose `max_floors` because Unity can use it for tutorial progress display; `Endless` omits it.
- `current_floor_seed()` preserves the former map seed namespace to avoid accidental first-Floor map churn while changing the public terminology.
- Encounter difficulty currently uses `difficulty_floor_index()` with a u8 cap. This is a mechanical continuation of the previous difficulty offset and should be revisited in the later mode-specific map policy goal if Endless pacing needs different balance.
- `handle_complete_combat_result` now validates node session before final boss checks and uses `node_session.node_id` as the combat node identity source.
- No transitional old Act DTO schema was added.
- Save/checkpoint payload migration was not handled in this slice; existing in-memory checkpoint tests pass. External persistent save compatibility remains a follow-up only if the project needs to preserve old saves.

## Final Phase 1 State

- Runtime code, live RON, Unity-facing snapshot/result DTOs, canonical non-goal docs, and tests now agree on Floor terminology for run progression.
- `docs/probe` is absent in this workspace, so no probe document was updated.
- Phase 2 must not start until the user explicitly asks to continue.
