# Boss Omen Gate And Rollback Policy Notes

Status: complete

## Fixed Decisions

- The forced BossOmen chain capstone is not the final boss of Endless mode.
- Victory commits the G15 result, clears the chain, and opens a dedicated Gate on the same Floor.
- Only confirming that Gate applies normal Gate trauma recovery and advances the Floor.
- Defeat commits and enters RunFailed before any recovery affordance.
- A valid SavePoint checkpoint with remaining loads exposes the existing explicit load action from
  qualified BossOmen RunFailed; no capacity means no action.
- `RetreatBattle` remains G15 exact pre-entry restoration and never opens Gate or RunFailed.

## Source-Of-Truth Boundary

- `BossOmenRunState.forced_boss` identifies the special capstone node.
- G15 owns result commit/abandonment/fatal priority.
- Map/node completion owns dedicated Gate creation and ordinary Gate confirmation.
- `RunCheckpointState` owns factual checkpoint/load capacity.
- Core action capability plus execution preflight own current RunFailed load executability.

## Direct Predecessor Invariants

- Do not add attempt retry/cancel before BossOmen RunFailed.
- Do not apply result employee/consumable/Event effects again during Gate or checkpoint load.
- Do not turn map recovery-node restriction into BossOmen recovery; checkpoint load is a distinct
  explicit RunFailed action.
- Do not add a public compatibility command or duplicate Gate transition implementation.

## Completion Gate

- Review: `COMPLETION_REVIEW.md`.
- Verdict: Complete.
- Long-term fit: High.
- Immediate correction: none.
- G08 is unlocked to consume the existing Gate/checkpoint/capability contract in Unity.
