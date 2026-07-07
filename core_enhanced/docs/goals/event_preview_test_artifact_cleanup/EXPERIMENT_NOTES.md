# Experiment Notes: Event Preview Test Artifact Cleanup

## Policy Decisions

- Event nodes must explicitly author `event_id`; no first-event fallback.
- `CombatPreview.enemy_stat_scale` must not be an ambiguous public/storable field that resets through serde round-trip.
- Tests and debug helpers must not write generated output into repository root by default.
- Test names should describe current runtime contracts rather than old refactor phases.

## Initial Risk Notes

- Some debug artifact cleanup may already be partially complete under `battle_record_debug_export_boundary`; verify before editing.
- `skill_runtime_contract.rs` is the current contract-test filename. Historical goal docs may still mention the old `skill_refactor_validation.rs`; do not churn stale completed goal logs unless they are still used as commands.
- `CombatPreview` may be both internal preview model and serialized snapshot payload today. Do not guess; inspect `behavior.rs`, snapshot building, probes, and Unity-facing docs before changing shape.

## Follow-Up Candidates

- A broader “test harness naming and ownership” cleanup may be useful if many refactor-era test names remain.
- A separate CombatPreview DTO split may belong with the wire boundary goal if this cleanup discovers that preview shape is deeply shared.
- Existing tracked repo-root debug artifacts under `battle_records/`, `debug_event_log_exports/`, and `logs/` still exist. This goal verified new default output paths but did not delete historical tracked artifacts.
- `CombatPreview.enemy_stat_scale` remains available to core battle setup before serialization. If a future Unity preview needs floor scaling display, add a separate presentation field rather than exposing this raw internal scale.
