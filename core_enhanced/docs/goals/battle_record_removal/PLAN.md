# Battle Record Removal

Goal: G16

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Remove player/gameplay battle history from run state, persistence, checkpoint/floor copies, public
queries, and legacy tests while retaining only explicitly non-authoritative developer combat traces
that cannot fail or alter gameplay transitions.

Primary findings/policy: DCA-M-01, DCA-L-05, accepted PD-12 battle-history removal.

## Required Resume Read Set

- `../cross_layer_contract_hardening_master/STATUS.md`
- active Phase 4 `PLAN.md` and `EXPERIMENT_NOTES.md`
- this file, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`
- G23 `COMPLETION_REVIEW.md`
- PD-12 in the Master plan
- `RunState`, run checkpoint/floor/run-complete construction, live battle-to-result transition,
  active CombatResult event-log attachment, public getters, debug exporters, tests, and canonical
  battle record/result text

## Fixed Decision

- Delete gameplay battle record state without save migration, compatibility field, replacement
  history, or retained identity/dedupe policy.
- Active battle and active CombatResult may retain event/unit facts only for their runtime/result
  purpose. They are discarded with that content lifecycle.
- Developer traces live outside gameplay/save/checkpoint state, use actual battle identity, and are
  best-effort: filesystem/serialization failure may warn but must never reject a battle transition.
- G29, not G16, owns the new permanent encounter compendium.

## Scope

- Remove `RunState.battle_records`, constructors/copies/checkpoint handling, public getter, store
  method, abnormality representative-record identity, and legacy assertions.
- Replace abnormality-keyed `target/battle_records` export with battle-keyed
  `target/combat_traces` diagnostic output isolated from authoritative state.
- Preserve active CombatResult attachment, result stats, bonus objectives, debug event-log exports,
  and live battle event logs.
- Update canonical docs and remove tracked legacy generated battle-record artifacts already marked
  deleted in the dirty baseline.

## Validation

- Focused live battle-to-result trace test proves actual battle UUID output and result continuity.
- Floor/run-complete/checkpoint/retreat tests prove no retained record state or replacement history.
- Search for `battle_records`, representative codex wording, abnormality-keyed trace identity,
  repository-root output, serde aliases, and ignored tests.
- Core all-targets, formatting, diff check; server/Unity only if public serialized shape changes.
- Completion review before G29 unlocks.

## Exit Gate

Complete / High when no gameplay battle-history state or contract remains and diagnostic traces are
battle-keyed, external to authoritative state, and unable to fail gameplay transitions.

Achieved: `COMPLETION_REVIEW.md` returned Complete / High with no immediate correction.
