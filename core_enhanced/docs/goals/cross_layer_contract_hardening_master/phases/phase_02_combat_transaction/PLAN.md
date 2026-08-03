# Phase 02 Combat Transaction And Boss Recovery

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Establish one atomic combat-attempt transaction from tactical employee incapacitation through
victory, defeat, battle-stage abandonment, BossOmen transition/recovery, and the final Unity
recovery command surface.

## Required Resume Read Set

- `../../STATUS.md`
- this file
- `EXPERIMENT_NOTES.md`
- the active Child Goal's `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`
- `../phase_01_authority/COMPLETION_REVIEW.md` as the direct predecessor
- the active Child's direct predecessor completion review once one exists

Read the historical Master logs or audit source only for a concrete policy/ownership conflict,
finding provenance, Phase review, or an explicit Child Required Read entry.

## Child Order And Status

| Child/review | Status | Review | Role |
| --- | --- | --- | --- |
| G26 `combat_employee_incapacitation_resolution` | complete | Complete / High | attempt-local employee condition and aggregation |
| G15 `battle_failure_abandonment_consumable_resolution` | complete | Complete / High | exact result and pre-entry rollback table |
| G14 `boss_omen_gate_and_rollback_policy` | complete | Complete / High | dedicated exit Gate and RunFailed recovery |
| G08 `unity_recovery_and_roster_command_flows` | complete | Complete / High | final authoritative Unity command/UX flow |
| R02 `cross_layer_phase_02_combat_completion_review` | complete | Complete / High | independent Phase integration review |

## Accepted Policy

- PD-16: every tactical death applies one complete authored penalty bundle to attempt-local employee
  state; fresh redeployment reads that state; committed victory/defeat keeps it; `RetreatBattle`
  discards it with exact pre-entry restoration.
- XP, result summary rows, and MVP are employee-level outcomes, not per-runtime-instance death
  penalties.
- PD-11: G15 will consume G26's commit/discard boundary for atomic victory, defeat, retry, failure,
  and battle-stage abandonment transitions.
- PD-10: G14 will consume the stable result transaction for BossOmen Gate and checkpoint recovery.
- Phase 1 PD-01/PD-02 revision/capability authority remains fixed across every Phase 2 command.

## Phase Invariants

- Tactical runtime instances remain available for replay and event history without becoming the
  persistent roster source of truth.
- Each runtime-instance tactical death affects the matching employee exactly once.
- Redeployment derives Battle HP and eligibility from current attempt-local employee state; the
  fixed 60% override is removed without fallback.
- A committed result exposes one coherent employee-level summary and commits condition changes once.
- Battle-stage abandonment eventually restores the exact authoritative pre-entry state, including
  attempt/RNG/consumption state; G26 supplies the employee-state seam and G15 owns full restoration.
- Boss and Unity recovery consume, rather than recreate, Core transaction and capability authority.

## User-Visible Exit Conditions

- Repeated deaths of one employee accumulate exactly one authored penalty per death and affect the
  next deployment immediately.
- XP, employee result rows, and MVP remain one coherent employee-level result across redeployments.
- Victory/defeat, abandonment, retry/failure, BossOmen Gate/recovery, and Unity commands pass the
  complete Phase transition matrix.
- No fixed-60%, partial rollback, duplicate penalty, static recovery action, legacy DTO, or fallback
  path remains.

## Explicit Non-Scope

- Phase 3 Event UX/DTO work and later targeting/economy phases.
- G05 and G22 deferred content/balance work.
- New public DTO, RON, save migration, balance values, or UX meaning unless separately approved.

## Stop Conditions

Stop only if implementation requires a new policy in a protected category, conflicts with an
unrelated or unclassifiable dirty overlap, or the required external validation surface cannot be
reached after safe in-scope alternatives are exhausted.
