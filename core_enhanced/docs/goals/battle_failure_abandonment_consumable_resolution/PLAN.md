# Battle Failure, Abandonment, And Consumable Resolution

Goal ID: G15

Status: complete

Implementation complete: true

Updated: 2026-07-14

Primary findings: DCA-M-17, DCA-M-20, DCA-M-31, DCA-L-04, CPR-H-02, CPR-H-09

## Required Resume Read Set

1. `../cross_layer_contract_hardening_master/STATUS.md`.
2. `../cross_layer_contract_hardening_master/phases/phase_02_combat_transaction/PLAN.md`.
3. `../cross_layer_contract_hardening_master/phases/phase_02_combat_transaction/EXPERIMENT_NOTES.md`.
4. This `PLAN.md`.
5. This Goal's `EXPERIMENTS.md`.
6. This Goal's `EXPERIMENT_NOTES.md`.
7. `../combat_employee_incapacitation_resolution/COMPLETION_REVIEW.md`.

Master Q10/PD-11 and audit sources are read only when a concrete policy detail or finding provenance
must be rechecked.

## Objective

Replace the split retry/retreat/result branches with one explicit combat-attempt transition model.
Committed victory or defeat keeps exactly one coherent result and authored consequences. Whole-stage
`RetreatBattle` restores the exact authoritative state from immediately before entry. Fatal roster
failure and capability-qualified recovery deferral are evaluated after the committed result in the
accepted priority order.

## Accepted Transition Table

| Exit | Employee condition | Attempt counter | Consumable/direct use | Node/result | Next state |
| --- | --- | --- | --- | --- | --- |
| Victory | commit G26 working state once | keep entry consumption | decrement combat duration once; keep direct use | commit one result and normal node consequences | normal completion/reward |
| Non-boss defeat, roster viable, attempts remain | commit once | keep entry consumption | decrement once | keep node unresolved | NodeConfirm retry |
| Non-boss defeat, roster viable, attempts exhausted | commit once | keep entry consumption | decrement once | consume failed node | ViewingMap |
| Any committed result, no living employees | commit once | keep entry consumption | decrement once | no retry/cancel | RunFailed immediately |
| Non-boss committed result, no deployable employees | commit once | keep entry consumption | decrement once | consume failed node | only immediately enterable Treatment/Recruitment-capable nodes, else RunFailed |
| Boss defeat | commit once | keep entry consumption | decrement once | committed boss result | RunFailed immediately; G14 adds checkpoint recovery |
| `RetreatBattle` | discard working state | restore pre-entry | restore all direct/duration use | no result, no node consumption | exact pre-entry NodeConfirm |

After a deferred recovery node resolves, re-evaluate deployability. If no employee is deployable,
enter RunFailed. During deferral no other node may be selected. Future-reachable, label-only Safe,
Shop, and non-capable nodes never qualify.

## Repository Baseline

| Repository view | Branch/HEAD | Dirty fingerprint | Initial classification |
| --- | --- | --- | --- |
| Core | `main` / `c2c5dc5` | `13408034...` (120 paths) | heavily dirty; combat/state/node-flow files contain completed Phase 0/1/G26 changes that must be preserved |
| game_server | `main` / `c2c5dc5` | `0a9b6db...` | unchanged by G26; no G15 transport shape expected |
| Unity | `master` / `7cd8df1` | `733c1986...` | unchanged by G26; consume existing state/capability/map projection |

Existing changes are user-owned. Do not stash, reset, restore, clean, or baseline-commit them.

## In Scope

- A battle-entry snapshot capturing every mutable authoritative field that battle entry/execution can
  change, including attempts, RNG/UUID continuation, roster, inventory/resources, consumables,
  node/progression/content, VN/trigger state, checkpoint state, and battle/session identity.
- Exact snapshot restoration on every `RetreatBattle`, with no exhausted-abandonment result or
  rumor/discovery mutation.
- One committed defeat plan that stages employee condition, consumable duration, event rewards,
  attempt/node transition, and failure priority before mutation.
- Fatal `NoLivingEmployees` priority over retry/cancel.
- Internal typed recovery capability evaluation and restriction to immediately enterable
  Recruitment/Treatment nodes; no label/category heuristic.
- Re-evaluation after the one recovery node resolves.
- Canonical terminology and transition documentation.

## Current Live Capability Boundary

- Recruitment is explicit only for a HeadquartersContact node whose authored candidate count and
  live recruitment catalog can produce candidates.
- No current map payload exposes a Treatment capability. G15 must not treat `Rest`, `SavePoint`,
  generic Support/Safe wording, Shop, or future reachability as Treatment.
- Adding a new public Treatment payload/DTO is outside G15 and would require a separate schema
  decision. The evaluator should remain typed and extensible without inventing that field now.

## Explicit Non-Scope

- G14 BossOmen victory Gate and RunFailed checkpoint-load command/UX.
- G08 Unity recovery/roster command implementation.
- G16 battle-record/debug export removal and G29 compendium commit.
- New DTO/RON/save schema, Treatment content, balance values, or recovery-node authoring.

## Planned Implementation

1. Add focused failing tests for attempt restoration, exhausted-abandonment removal, fatal defeat
   priority, once-only consumption, and recovery-node restriction.
2. Introduce one pre-entry authoritative snapshot captured before attempt/UUID/progression mutation
   and one restore operation consumed by `RetreatBattle`.
3. Replace retreat branches with snapshot restoration and a read-only NodePreview response.
4. Consolidate committed defeat staging and order `commit -> fatal failure -> qualified recovery ->
   retry/exhaustion` without partial mutation.
5. Add a typed internal recovery capability/restriction seam shared by map projection and
   `SelectMapNode` validation; clear/re-evaluate after recovery resolution.
6. Update canonical docs, remove exhausted retreat/result legacy, and run focused/broad validation.
7. Perform the guide completion review before unlocking G14.

## User-Visible Completion Conditions

- Repeated `RetreatBattle` never consumes an attempt and restores exact pre-entry gameplay state,
  including UUID/RNG continuation and all direct/derived consumable mutations.
- Abandonment creates no CombatResult, record/compendium commit, node completion, reward, penalty,
  or exhausted special branch.
- Victory/defeat decrements combat-duration use exactly once and never on abandonment.
- A committed fatal roster cannot reach retry or cancel.
- A no-deployable nonboss result exposes only currently enterable explicit recovery-capable nodes;
  resolving one either restores deployability or fails the run.
- Incapacitation wording distinguishes tactical death from persistent employee death.

## Planned Validation

1. Focused transition-matrix tests with full serialized pre-entry/after-restore comparisons.
2. G26 repeated-death/redeploy/commit/discard regression.
3. Attempt, consumable, node progression, recovery restriction, boss/nonboss failure tests.
4. Core check, library, and all-targets.
5. game_server check/all-targets if shared Core behavior reaches transport.
6. Unity MCP EditMode and applicable PlayMode against existing map/capability projections.
7. Formatting, scoped diff, exhausted-abandonment/partial-rollback/legacy terminology searches.

## Stop Conditions

Stop if G15 requires a new public DTO/RON/save schema, new Treatment or recruitment content, a new
failure/reward/consumption meaning, an authority change, or an unrelated unclassifiable dirty hunk.
