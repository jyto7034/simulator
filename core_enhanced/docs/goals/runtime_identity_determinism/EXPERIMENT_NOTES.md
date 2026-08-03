# Runtime Identity Determinism Notes

Goal: G17

Status: active

## Fixed Predecessors

- Phase 2 exact checkpoint/retreat and RNG continuation are closed.
- Phase 3 top-level VN remains outside run checkpoint; rewindable trigger history remains inside the
  checkpoint payload by policy.

## Resolved Findings

- Whole-manager checkpoint restore did reuse a still-active top-level VN session id. The corrected
  boundary rewinds checkpoint-owned streams while keeping the VN session namespace monotonic.
- Duplicate identity made active and queued command targets ambiguous. The regression now proves
  distinct promotion and stale-id rejection without mutation.
- Basic and skill projectile identities consumed event-log parent sequence; skill cast/area identity
  consumed AbilityCast event sequence. They now use stable damage occurrence or a separate gameplay
  cast counter.
- An observation-only log insertion changed delivery ids before correction. After correction it does
  not change gameplay cast identity, projectile/area identity, queue order, or area damage outcome.

## Follow-Up Candidates

- None inside G17. Event-log sequence remains intentionally authoritative only for append-only log
  parent relations and cancellation references.
