# SavePoint Checkpoint Support Notes

## Initial Judgment

The long-term fix is to remove the Medical support model and introduce SavePoint as a canonical run checkpoint feature.

Renaming `Medical` to `SavePoint` would be actively harmful because the old semantics are different:

- Medical is target/treatment based.
- Medical heals HP.
- Medical is treated as a possible recovery path when employees are not deployable.
- SavePoint is a checkpoint and trauma relief node, not a healer.

## Implementation Shape To Prefer

Prefer a clear split between:

1. support-node flow cleanup;
2. SavePoint node effect;
3. run checkpoint save/load state;
4. no-deployable recovery logic.

This keeps the support UI contract simple while making checkpoint load a first-class run command.

## Checkpoint Scope Implemented

The implemented checkpoint scope is a structured snapshot of canonical run/world state needed to resume from the last completed SavePoint:

- map progression;
- run progression and combat preview state;
- abnormality attempt state;
- roster run state;
- inventory/equipment/skill fragment resources;
- research progress and pending completions stored in skill fragment inventory;
- Enkephalin;
- UUID manager state.

The checkpoint payload intentionally does not store active node sessions, active battle sessions, or debug/battle record exports. Load is Safezone/ViewMap-only and clears transient node/battle state.

Confirmed checkpoint restore boundaries:

- Checkpoint load should feel like a full return to the completed SavePoint moment for user-visible run state.
- Load is Safezone-only, so active combat result screen data is not a normal load scenario.
- Debug/operational logs and battle record exports are preserved as records of what actually happened.
- Player-facing collectible documents/story unlocks are not implemented yet; when implemented, they should be preserved across checkpoint load instead of rewound.
- The per-run load count is outside the checkpoint payload and is never restored by load.
- `run_checkpoint` snapshot metadata exposes checkpoint existence, used loads, max loads, remaining loads, and current load availability for Unity UI.

## Things To Avoid

- Keeping `Medical` as a backwards-compatible alias.
- Accepting both `"Medical"` and `"SavePoint"` in live RON.
- Keeping `SelectMedicalTreatment` as a hidden no-op.
- Leaving Medical target/treatment fields nullable in support snapshots "just in case".
- Renaming `has_available_medical_support_node()` to a SavePoint helper.
- Treating future SavePoint nodes as recovery paths.
- Saving checkpoint data in Unity-facing DTOs instead of canonical runtime state.
- Implementing checkpoint as untyped `serde_json::Value` if typed state is practical.

## Resolved Policy Questions

- Checkpoint load restores the canonical user-visible run state listed above.
- UUID manager state is restored with the checkpoint payload so generated ids after load continue from the saved moment.
- Consumed anomaly attempts after the checkpoint are restored because they are user-visible run state.
- Pending research deliveries after the checkpoint are restored through the skill fragment inventory snapshot.

## Out Of Scope

- Reworking Rest beyond ensuring it is not confused with HP recovery.
- Adding multiple save slots.
- Adding autosave outside SavePoint.
- Adding Unity UI implementation.
- Rebalancing trauma formulas outside `save_point_trauma_reduction_percent`.
- Redesigning all map node session handling unless checkpoint correctness requires it.

## Follow-Up Candidates

- Add an admin/debug command to inspect checkpoint metadata, such as whether a checkpoint exists, the saved node id, and loads remaining.
- Add a probe scenario dedicated to SavePoint once Unity can route to the screen.
- Add documentation examples for checkpoint load responses after the implementation stabilizes.
