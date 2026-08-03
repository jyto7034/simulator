# Safe Zone Research Delivery Policy Notes

Goal: G21

Status: complete

## Fixed Guardrails

- Queue during node activity; deliver on authoritative planning return only.
- No category allowlist, node-entry hook, Unity screen-enter command, or reconnect grant.
- Atomic all-pending delivery and pending removal; idempotent repeated rendering.
- Nested Event/combat waits for owning-node resolution; Gate arrival qualifies; RunFailed does not.
- Preserve G29 global compendium, G23 reward/growth, G15 result/rollback, and Phase 3 Event/VN
  ownership.

## Final Findings

- Planning return is a semantic boundary shared by typed completion paths, not a map-node category.
- `NodeCompleted` and `FloorAdvanced` are the complete public delivery surface. Typed wrappers carry
  one of those completions rather than creating another authority.
- Terminal RunComplete/RunFailed results intentionally leave pending research queued. A future
  post-run settlement policy would require an explicit new decision; it must not be inferred here.
- SavePoint checkpoint ordering, Event ownership, combat retry/retreat/failure, and Gate arrival now
  compose existing transaction seams without a category or Unity compatibility layer.
- Final verdict: Complete / High / no immediate correction.
