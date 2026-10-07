# P05 Compendium Cutover Execution Plan

- Goal ID: `GOAL-CWBS-P05-COMPENDIUM-CUTOVER`
- Phase ID: `PHASE-CWBS-05-COMPENDIUM-CUTOVER`
- Final Execution Node: `WP-CWBS-10`
- Accepted Handoff / Goal Set: `HA-CWBS-P2G60-002 / GS-CWBS-P2G50-002`
- Run Mode / Root Target: `goal-run / GOAL-CWBS-MASTER`
- Authorization Ref: `GEXAUTH-CWBS-001`
- Current Pass: `GEX-20 complete / Audit Pass 1 / Complete / High`
- Direct Dependencies:
  - `GOAL-CWBS-P01-RUNTIME-FOUNDATION / Complete / High / Audit Pass 3`
  - `GOAL-CWBS-P04-OUTCOME-CUTOVER / Complete / High / Audit Pass 1`
- Execution Directory:
  `core_enhanced/docs/goals/conditional_wave_and_boss_summon_policy/execution/p05_compendium_cutover`
- Document Set: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`; fresh GEX-20 creates `COMPLETION_AUDIT.md`

## Objective and Scope

Collect every successfully spawned opponent Abnormality/CorrodedEmployee canonical identity in a process-local ordered
attempt ledger, retain the candidates across death, despawn and same-process reconnect, and commit the whole set exactly
once only with the final committed Victory. Replace the four discovered/defeated profile sets with one canonical
registered set and an empty replacement schema without legacy union migration.

Covered revision: `PER-CWBS-BL-05-CODE-006`; decision: `DR-CWBS-BL-05-002`.

In scope: successful-spawn candidate producer, active battle/session ledger lifetime and projection, final outcome-bound
atomic/idempotent commit, Core compendium profile/result/snapshot shape, server profile serialization/store/reset and
development profile regeneration required by the replacement schema.

Out of scope: Defeat or Retreat registration, legacy profile union migration, durable SavePoint battle/candidate state,
Unity UI, production content promotion, deployment and external environment changes.

## Target Contract

- Candidate authority is the successful opponent spawn commit, not result-time event-log reconstruction. Primary target,
  death, despawn, damage credit and outcome-specific credit do not filter a recorded candidate.
- The attempt ledger is ordered and identity-deduplicated, survives same-process detach/reconnect with the authoritative
  active battle session, and disappears on final non-Victory settlement or process death.
- A committed final Victory atomically/idempotently unions every candidate into the one permanent registered set inside
  the existing outcome/profile transaction. Defeat and Retreat leave permanent registration unchanged.
- The replacement profile has exactly one registered set. New/reset profiles are empty; old four-set fields are rejected
  and no migration or union compatibility path remains.
- Active battle and candidate ledger remain excluded from `RunCheckpointPayload`; explicit SavePoint load behavior is
  unchanged.

## Validation and Completion

- successful opponent spawn records eligible canonical identity exactly once; failed/canceled spawn records nothing
- death/despawn and same-process reconnect retain candidates; Defeat, Retreat and process death discard them
- final Victory commits all candidates regardless of primary/death/credit and duplicate completion cannot double-apply
- Core/server replacement profile round-trip and reset expose one registered set and an empty new schema
- old four-set input and legacy union migration are absent/rejected; no result-time reconstruction or non-Victory writer
- SavePoint payload remains battle/candidate-free and P01/P04 lifecycle/outcome invariants remain current
- focused invalidated seams only in P05; the inherited 1,152-pass full baseline remains reserved for P06

## Stop Conditions

- a required consumer cannot operate without the four-set profile or requires candidate state in a durable payload
- final Victory registration cannot share the existing atomic result/profile transaction without a new policy authority
- implementation requires legacy migration, Unity work, production promotion, deployment or external state mutation
- a policy/scope decision is required, or overlapping existing dirty changes cannot be preserved safely
