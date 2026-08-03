# Battle Failure, Abandonment, And Consumable Resolution Notes

Status: complete

## Fixed Decisions

- `RetreatBattle` means whole-stage abandonment; `WithdrawUnit` remains an in-battle tactical action.
- Abandonment is exact pre-entry restore and never a committed CombatResult.
- Victory/defeat are committed results and consume one attempt plus one combat-duration use.
- G26 working employee condition is committed once for results and discarded for abandonment.
- Fatal committed roster state is evaluated before retry/cancel.
- Boss defeat is immediate RunFailed; G14 owns optional checkpoint recovery from RunFailed.
- Nonboss no-deployable recovery deferral requires immediate enterability and explicit Treatment or
  Recruitment capability, and restricts selection until resolution.

## Source-Of-Truth Shape

- `BattleEntrySnapshot` owns complete authoritative restoration data.
- Active battle owns the snapshot and G26 working employee transaction.
- One committed-result planner owns employee/consumable/event reward/node/failure ordering.
- One typed recovery capability evaluator feeds both projection and selection validation.

## Completion Gate

- Review: `COMPLETION_REVIEW.md`.
- Verdict: Complete.
- Long-term fit: High.
- Immediate correction: none.
- G14 is unlocked and must preserve G15's exact abandonment, fatal priority, and typed recovery
  boundaries.

## Current Live Capability Facts

- HeadquartersContact can explicitly produce `RecruitEmployee` when candidates exist.
- Support has only SavePoint and Rest. Neither is Treatment and neither restores Run HP/dead staff.
- Shops and label-based Safe semantics do not qualify.

## G26 Handoff Invariants

- Never replay participant incapacitation penalties at result settlement.
- Never restore fixed-60% death redeploy.
- Preserve employee-level result aggregation and existing revision/capability authority.
