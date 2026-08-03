# Global Encounter Compendium

Goal: G29

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Implement the PD-12 game-global monotonic abnormality/corrosion compendium. Collect canonical IDs
only from enemy units that actually spawn during a combat-node attempt, and commit their union only
in the authoritative transaction that consumes the combat node.

Primary finding/policy: MSTR-04, accepted PD-12.

## Required Resume Read Set

- `../cross_layer_contract_hardening_master/STATUS.md`
- active Phase 4 `PLAN.md` and `EXPERIMENT_NOTES.md`
- this file, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`
- G16 and G15 `COMPLETION_REVIEW.md`
- PD-12 in the Master plan
- `GameCoreState` global/run/checkpoint construction, battle spawn source identities, active battle
  attempt state, victory/final-defeat node-consumption planning/commit, root snapshot/profile DTO,
  server pass-through, Unity store/readers, and canonical compendium text

## Fixed Decision

- Store only discovered canonical abnormality IDs and canonical corrosion profile IDs. Do not add
  species grouping, variants, battle history, counters, timestamps, or trace linkage.
- Preview/authored wave entries do not count. An ID qualifies only when its enemy unit actually
  spawns on the battlefield during that attempt.
- Commit only when victory or final defeat consumes the combat node. Retryable/non-consuming defeat
  and `RetreatBattle` do not register discoveries.
- Node consumption and set union are one idempotent authoritative commit.
- Committed discovery is game-global and monotonic across checkpoint load, older run-state load, new
  run, RunFailed, and ordinary rollback. Only an explicit whole-profile/compendium reset may clear it.
- Persist IDs only; current display metadata resolves from strict live catalogs.
- G16 diagnostic traces never read or mutate the compendium.

## Validation

- Exact actual-spawn versus preview/unspawned wave tests for abnormality and corrosion identities.
- Exact victory, retryable defeat, final consuming defeat, retreat, repeated completion/idempotence,
  checkpoint load, new run, RunFailed, and older-state union tests.
- Serialization/root snapshot and server/Unity contract tests if the player compendium is projected.
- Core focused, relevant server/Unity focused and broad suites for every changed external shape,
  Core all-targets, formatting, diff, alias/fallback/ignored-test searches.
- Completion review before G21 unlocks.

## Exit Gate

Complete / High when actually spawned IDs commit exactly with node consumption, all ordinary
rollback/load/new-run paths preserve the monotonic global union, current catalog metadata has one
authority, and no battle-record schema or diagnostic path participates.

Exit result: Complete / High with no immediate correction. See `COMPLETION_REVIEW.md`.
