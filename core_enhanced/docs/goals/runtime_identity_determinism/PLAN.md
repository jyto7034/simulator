# Runtime Identity Determinism

Goal: G17

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Reproduce or disprove two latent identity risks before changing design: checkpoint rewind reusing a
still-live VN session identity, and observation/event-log sequence changing delivery identity or
same-timestamp gameplay ordering.

## Required Resume Read Set

- `../cross_layer_contract_hardening_master/STATUS.md`
- active Phase 4 `PLAN.md` and `EXPERIMENT_NOTES.md`
- this file, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`
- Phase 3 and R03 `COMPLETION_REVIEW.md`
- exact DCA-M-02 and DCA-M-22 audit sections
- current checkpoint payload/load, UUID manager, VN active/queue/trigger history, delivery identity,
  event ordering, crit/proc RNG, and existing determinism tests

## Fixed Decision

- Reproduction first. Do not redesign UUID, save/checkpoint, VN queue, event log, delivery, or RNG
  authority from structural suspicion alone.
- If the claimed user-visible/state divergence is not reproducible under the actual current
  contracts, record `Not applicable` with a durable regression test and make no production change.
- If reproduced, implement the smallest complete deterministic identity boundary without aliases,
  random fallback, log-order dependence, or save-policy expansion.

## Experiments

1. Create a checkpoint, activate/retain a post-checkpoint nonblocking VN, load the checkpoint, and
   fire a rewindable trigger again. Compare active/queued session ids, consumed history, command
   targeting, and queue promotion.
2. Build identical battle inputs with an inserted observation-only event before a projectile/area
   delivery. Compare delivery ids, same-timestamp ordering, damage/outcome, and RNG results.
3. Trace whether current code has already decoupled the cited sequence identities since the audit.

## Validation

- Focused deterministic tests must execute nonzero cases and compare complete relevant state, not
  only UUID inequality.
- If production changes, rerun affected battle/VN/checkpoint, Core all-targets, server if DTO/save
  shape changes, formatting, and identity/fallback/ignored searches.
- Completion review guide before G23 unlocks.

## Exit Gate

Complete / High if proven defects are fixed and protected, or Not applicable / High if both latent
claims are disproved on current runtime with durable regressions and no unnecessary production code.

Result: Complete / High. See `COMPLETION_REVIEW.md`.
