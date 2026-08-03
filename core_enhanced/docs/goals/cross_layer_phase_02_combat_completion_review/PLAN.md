# Cross-Layer Phase 02 Combat Completion Review

Review: R02

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Independently verify that Phase 2 forms one coherent transaction and recovery contract across Core,
game_server, Unity, tests, and canonical documentation. The review must cover employee tactical
incapacitation/redeployment, committed result settlement, exact battle-stage abandonment, BossOmen
Gate/failure recovery, and actual Unity recovery/roster commands without relying on Child verdicts
alone.

## Required Resume Read Set

- `../cross_layer_contract_hardening_master/STATUS.md`
- `../cross_layer_contract_hardening_master/phases/phase_02_combat_transaction/PLAN.md`
- `../cross_layer_contract_hardening_master/phases/phase_02_combat_transaction/EXPERIMENT_NOTES.md`
- this file
- `EXPERIMENTS.md`
- `EXPERIMENT_NOTES.md`
- `../unity_recovery_and_roster_command_flows/COMPLETION_REVIEW.md`
- the conclusions and correction sections of G26, G15, and G14 completion reviews when checking the
  combined transition matrix
- `../cross_layer_phase_01_authority_completion_review/COMPLETION_REVIEW.md` for the fixed revision/
  capability predecessor boundary

Read full historical Master/audit logs only for a concrete ownership conflict or finding provenance.

## Review Scope

- PD-16 per-tactical-death attempt-local employee condition and fresh redeployment.
- Employee-level XP/result/MVP aggregation across runtime instances.
- PD-11 atomic victory/defeat/result settlement and exact `RetreatBattle` restoration.
- Fatal NoLiving/boss priority, retry semantics, and typed nonboss recovery restriction.
- PD-10 forced BossOmen committed defeat, dedicated same-Floor Gate, and qualified checkpoint load.
- Phase 1 battle revision/capability authority across every Phase 2 mutation/recovery command.
- Unity setup-loss, checkpoint, roster, Gate, and incomplete-runtime consumption without local rules.
- Full cross-layer transition matrix, broad regressions, canonical alignment, and legacy/fallback
  searches.

## Independence Rules

- Child completion reviews are leads, not proof. Reinspect current runtime seams and rerun the
  combined matrix against the current diff.
- Do not implement a correction inside R02. If the Phase is not Complete / High, record exact
  evidence and create/activate one bounded correction owner before Phase 3.
- Do not reopen G05/G22 or absorb Event/targeting/economy/persistence work.
- Preserve all unrelated dirty-worktree changes.

## Required Evidence

1. Current code trace from tactical death through attempt state, redeployment, result commit/discard,
   node completion/failure, BossOmen transition, and Unity command presentation.
2. Focused combined Core tests for repeated death/redeploy, employee aggregation, victory/defeat,
   retreat exactness, no-deployable recovery, forced Gate, qualified load, and negative controls.
3. Server command/result/snapshot/side-message evidence for nonbattle and battle recovery.
4. Unity focused and full EditMode, ordinary PlayMode, and applicable explicit live-server evidence.
5. Core/game_server all-targets, formatting, scoped diff checks, and zero-test accounting.
6. Searches for fixed-60% redeployment, partial rollback, direct forced Floor advance, static recovery,
   root battle fallback, local roster reordering, ignored focused tests, and compatibility aliases.
7. Guide-based Phase completion review with Verdict, long-term fit, correction classification, and
   next-Phase handoff.

## Exit Gate

Only `Complete / High / no immediate correction` completes Phase 2 and unlocks Phase 3 G02. Any
contradictory or missing evidence keeps Phase 2 active and enters a documented correction loop.

## Stop Conditions

Stop only for a protected new policy decision, an unclassifiable dirty overlap that cannot be
avoided, or unavailable required external validation after safe alternatives are exhausted.
