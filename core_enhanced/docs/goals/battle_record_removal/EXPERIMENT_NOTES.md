# Battle Record Removal Notes

Goal: G16

Status: active

## Fixed Guardrails

- No gameplay history replacement and no migration alias.
- No compendium implementation; G29 follows after this removal.
- Debug traces are not save/profile/checkpoint state and cannot be a completion dependency.
- Active result/event-log data remains only through its existing result lifecycle.

## Implementation Boundary

- Remove record storage from `RunState` and every copy/preserve path.
- Add a diagnostic-only module that writes actual `battle_uuid` traces under `target/combat_traces`.
- Call diagnostics best-effort before/after result transition without returning its failure.
- Tests assert trace contents directly and assert gameplay result independently; they do not assert
  retained history.

## Final Result

- Gameplay battle-history state, queries, copy/preserve paths, and abnormality-keyed diagnostic
  identity are removed.
- Current-result data remains lifecycle-local; no replacement history or G29 compendium surface was
  introduced.
- Forced diagnostic filesystem failure now has direct nonzero regression evidence that CombatResult
  still commits.
- Checkpoint/retreat focused validation, Core 738 all-targets, formatting, diff, external legacy,
  alias, and ignored-test audits passed.
- Completion review: `COMPLETION_REVIEW.md`, Complete / High, no immediate correction. G29 owns all
  subsequent permanent encounter knowledge and must not reuse this removed schema.
