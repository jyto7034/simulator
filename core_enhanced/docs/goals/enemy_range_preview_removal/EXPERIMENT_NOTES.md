# Enemy Range Preview Removal Notes

Goal: G11

Status: complete

## Guardrails

- Enemy preview removal is not general range-preview removal.
- Classify each `range_previews` occurrence by actor side, request/query, active telegraph, mock,
  test, or canonical use before editing.
- Keep G10/G09 semantics unchanged until G11 review closes.
- Preserve dirty worktrees and all closed-goal changes.

## Final Findings

- Normal updates, recovery, and resync share one `LiveBattleUnitCheckpointDto` builder, so one
  side-aware projection closes every enemy checkpoint path.
- Unity rendering already selected by employee UUID; the remaining defect was parsing/storing a
  preview object for every checkpoint actor and old mocks omitting the now-required player object.
- The deployment-facing query and shared preview value types remain valid player-only surfaces.
- Final verdict: Complete / High / no immediate correction.
