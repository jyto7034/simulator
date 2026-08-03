# Runtime Identity Determinism Completion Review

Review date: 2026-07-14

Review procedure: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

Immediate correction: none

## Item Audit

| Finding | Integrated result | Verdict | Fit | Correction |
| --- | --- | --- | --- | --- |
| DCA-M-02 checkpoint/VN identity reuse | checkpoint streams rewind while top-level VN session identity remains monotonic | Complete | High | None |
| DCA-M-22 observation/log identity coupling | log parent seq and gameplay cast/delivery identity are separate domains | Complete | High | None |

## Runtime Review

- `UuidManager` has one explicit checkpoint restore operation. It replaces checkpoint-owned
  counters and preserves only named monotonic namespaces by maximum counter; checkpoint load names
  the top-level VN session namespace because Phase 3 keeps that runtime outside the run checkpoint.
- The implementation does not preserve the whole UUID manager. Equipment, consumable, employee,
  battle-session, and future unnamed streams retain exact checkpoint rewind behavior.
- `BattleCore::skill_cast_seq` is a battle-local gameplay occurrence counter with the same battle
  reset boundary as projectile, area, and damage-source counters.
- AbilityCast event seq remains append-only observation identity and the exact parent/cancellation
  reference. `ActiveSkillCast::ability_event_seq` preserves that relation without using it as the
  runtime map key or delivery seed.
- Basic projectile identity composes the battle seed with the stable damage-source occurrence UUID.
  Skill projectile and area identity compose battle seed, gameplay cast occurrence, and their
  gameplay delivery counters. No observation-only log append can alter those inputs.

## External Contract Review

- No behavior DTO, JSON/RON field, save schema, game_server mapping, Unity reader, or presentation
  contract changed.
- Existing event-log `seq`, `cause`, and `interrupted_cast_seq` semantics remain intact; cancellation
  continues to reference the AbilityCast log entry rather than the internal gameplay cast number.
- The public debug cast-state lookup continues to accept the AbilityCast event seq used by existing
  integration harnesses, but runtime identity has one internal gameplay owner.

## Test Review

- The checkpoint regression creates a checkpoint, retains a post-checkpoint top-level VN, rewinds
  trigger history, queues the replayed trigger, proves distinct session ids, promotes the queued
  session, and proves a stale old-id command is rejected without mutation.
- The UUID manager regression proves ordinary checkpoint-owned stream rewind and global VN stream
  monotonicity in the same restore.
- Observation regressions prove basic projectile identity, skill gameplay cast identity, skill
  projectile delivery identity, normalized same-time queue order, area identity, and damage outcome
  remain equal while AbilityCast/event-log sequence differs.
- Existing crit, delayed-hit, and proc observation-independence regressions remain green and cover
  RNG results from stable occurrence identities.

## Validation Review

- Exact G17 tests: checkpoint/VN 1, UUID restore 1, observation identity 3; all passed.
- Core library: 694 passed, 0 failed, 0 ignored.
- Skill runtime integration: 13 passed, 0 failed, 0 ignored.
- Core all-targets: 735 passed, 0 failed, 0 ignored across nonzero targets.
- Core all-targets check, formatting, diff check, identity/parent-seq search, and ignored-test search
  passed. The first post-format test attempt was an environment-only parent-target lock failure and
  was rerun successfully with a writable isolated target directory.
- game_server and Unity were not rerun because runtime review proves no serialized, transport, or
  client surface changed.

## Legacy And Fallback Audit

- No production projectile or area UUID seed reads `recording_cause.parent_seq`.
- No production AbilityCast event seq is reused as gameplay cast identity.
- No random UUID fallback, duplicate-session repair, whole-manager preservation, compatibility
  alias, dual counter, ignored regression, or server/client normalization was introduced.

## Cross-Component Findings

- Phase 2 checkpoint rewind and Phase 3 top-level VN persistence now compose without overlapping
  identity lifetimes.
- Battle event logs remain observational/causal records; simulation identity and ordering use
  gameplay occurrence state. Existing event-log validation and cancellation contracts still pass.

## Long-Term Direction Review

- Fit: High.
- Improvement class: none.
- Reason: both defects are corrected at their owning lifetime boundaries, preserving intended
  rewind semantics and existing external contracts while removing observation-order dependence.
  Future UUID namespaces and battle delivery kinds extend the same explicit mechanisms without
  compatibility or handler-specific patches.

## Review Conclusion

G17 is Complete / High with no immediate correction. G23 may begin.
