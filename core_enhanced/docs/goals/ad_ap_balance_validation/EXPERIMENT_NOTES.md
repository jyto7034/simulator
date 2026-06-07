# AD AP Balance Validation Notes

## Initial Notes

- This goal should not prove every node is mathematically clearable.
- The useful first target is static validation that catches obvious hard counters or misleading warnings.
- If current live data does not define boss immunity exceptions, do not invent a broad exception system unless current data needs it.

## Runtime Findings

- Current defense/magic resist stats are an efficiency axis, not a permanent immunity axis.
- Because `calculate_damage` applies `minimum_damage` after mitigation, basic damage requests with minimum damage stay at least 1 damage even against extreme resistance.
- A separate immunity exception schema would be premature right now. If actual type immunity is introduced later, it should be explicit data with a reviewed boss/elite policy.
- AD/AP warning thresholds already existed in preview code. The long-term cleanup is to keep them in one policy module so preview, validation, and damage feedback cannot drift.

## Follow-Up Candidates

- Add explicit enemy rank/category metadata if future validation needs different normal/elite/boss thresholds.
- Add authored warning history/disproved-state persistence when node re-entry memory is implemented.
- Add real immunity metadata only when a boss mechanic or skill effect requires it.
