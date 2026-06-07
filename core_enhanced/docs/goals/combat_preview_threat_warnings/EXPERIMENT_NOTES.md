# Combat Preview Threat Warnings Experiment Notes

## 2026-06-07

- Avoid adding a manual warning field to live PVE RON in this pass. It would be a live schema change and is not necessary for the first typed DTO.
- Generate `Briefing` warnings from actual materialized enemy entries so Unity does not recalculate warnings and the preview remains deterministic.
- Keep deferred tags as typed enum candidates only. Do not emit dummy `air`, `shield`, `regenerating`, or `hard_to_block` warnings until runtime/data can prove them.
- Initial thresholds are based on current live data ranges:
  - high armor: `defense >= 50`
  - high magic resist: `magic_resist >= 50`
  - fast breakthrough: `movement.speed_units_per_ms >= 3000`
- Rumor chance is currently a fixed 10 percent. If this becomes a balance/UX tuning issue, move it to data/config in a later goal.
- The canonical Unity contract already documents `threat_warnings`, so no external Unity doc write was needed for this sub-goal.
