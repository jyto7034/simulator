# Reward And Growth Canonicalization

Goal: G23

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Make current reward execution, XP distribution, level progression, battle tier derivation, and
GrowthStack behavior canonical without changing balance, reward contents, or progression semantics.

Primary findings: DCA-M-27 and DCA-M-29.

## Required Resume Read Set

- `../cross_layer_contract_hardening_master/STATUS.md`
- active Phase 4 `PLAN.md` and `EXPERIMENT_NOTES.md`
- this file, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`
- G17 `COMPLETION_REVIEW.md`
- current reward executor, reward sessions, combat-result commit, employee growth, run policy,
  live reward/run-policy RON, snapshots/results, server pass-through, Unity readers, and canonical
  reward/growth sections

## Fixed Decision

- Document current runtime/live-data behavior. Do not change reward policy, contents, chances,
  balance values, XP amounts, target distribution, level thresholds, tier bands, or stat scaling.
- If runtime and live data disagree or an actual correctness defect is found, classify it before
  implementation. A user-visible policy change requires a policy gate.
- Canonical text must distinguish generic representable reward effects from effects actually
  authored in current live pools and from automatic post-battle survival XP.

## Scope

- `RewardEffect`, `ExperienceTargetPolicy`, grant staging/atomicity, and result diffs.
- Live combat/reward pools and reward-mode validation.
- Employee experience accumulation, multi-level progression, threshold policy, battle tier, and
  GrowthStack stat scaling.
- Combat participant/survivor aggregation and non-combat reward-session targets.
- Canonical Core/rulebook documentation and exact regression evidence.

## Out Of Scope

- New rewards, content probabilities, inventory admission redesign (G24), Artifact removal (G27),
  Shop economy (G28), global compendium (G29), or research-delivery timing (G21).
- Equipment/fragment balance or deferred G22 content.

## Validation

- Focused reward executor, reward session, combat result XP, employee level/tier, and GrowthStack
  tests with nonzero counts.
- Live RON loading/validation and searches for stale random-completed-equipment or undocumented
  target/level claims.
- Core all-targets only if runtime or tests change; documentation diff/format checks otherwise.
- Completion review before G16 unlocks.

## Exit Gate

Complete / High when canonical documents exactly describe current runtime and live content,
meaningful tests prove the described behavior, and no unsupported future reward/growth claim remains.

Result: Complete / High. See `COMPLETION_REVIEW.md`.
