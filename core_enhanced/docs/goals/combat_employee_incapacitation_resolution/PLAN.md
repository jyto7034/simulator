# Combat Employee Incapacitation Resolution

Goal ID: G26

Status: complete

Implementation complete: true

Updated: 2026-07-14

Primary findings: CPR-H-01, MSTR-01

## Required Resume Read Set

1. `../cross_layer_contract_hardening_master/STATUS.md`.
2. `../cross_layer_contract_hardening_master/phases/phase_02_combat_transaction/PLAN.md`.
3. `../cross_layer_contract_hardening_master/phases/phase_02_combat_transaction/EXPERIMENT_NOTES.md`.
4. This `PLAN.md`.
5. This Goal's `EXPERIMENTS.md`.
6. This Goal's `EXPERIMENT_NOTES.md`.
7. `../cross_layer_contract_hardening_master/phases/phase_01_authority/COMPLETION_REVIEW.md`.

Read Master Q10/Q12 or the audit finding only when a concrete policy detail or provenance must be
rechecked; the accepted contract below is the normal resume authority.

## Objective

Replace the fixed-60%-HP redeployment shortcut and result-time per-instance roster mutation with one
world-owned attempt-local employee-condition state. Apply the authored incapacitation bundle once
per tactical death, use the resulting state for redeployment and eligibility, and resolve XP,
summary rows, and MVP once per employee while retaining instance-level replay facts.

## Accepted Contract

- Every tactical `UnitDied` applies the full authored incapacitation bundle exactly once for that
  runtime instance: trauma, Run HP loss, injury, trust/consumable mitigation, death-prevention
  consumption, and resulting life-state transition.
- The working state begins from the battle-entry roster and changes during the active attempt. A
  fresh deployment derives stats and deployability from that current working employee.
- A working employee that becomes persistently dead cannot redeploy in the attempt.
- Victory/defeat commits accumulated working conditions; battle-stage abandonment discards them.
  G26 owns the employee-state commit/discard seam; G15 owns exact whole-stage rollback and transition
  ordering.
- Runtime units and events retain per-instance history. Survival XP, employee result rows, and MVP
  aggregate by employee. Repeated tactical deaths remain repeated penalties, not duplicate rows or
  XP awards.
- Remove fixed `DeathRedeployPercent(60)` behavior and its policy tests; do not retain aliases,
  compatibility, or fallback behavior.

## Repository Baseline

| Repository view | Branch/HEAD | Dirty fingerprint | Initial classification |
| --- | --- | --- | --- |
| Core | `main` / `c2c5dc5` | `21175490...` (118 paths) | heavily dirty; target files contain completed predecessor changes and require scoped hunk preservation |
| game_server | `main` / `c2c5dc5` | `0a9b6db...` (12 paths) | dirty but no expected G26 runtime ownership |
| Unity | `master` / `7cd8df1` | `733c1986...` (106 tracked paths) | dirty; no public DTO or Unity change expected for G26 |

Existing changes are user-owned. Do not stash, reset, restore, baseline-commit, or clean them. Before
each target edit, preserve unrelated and completed predecessor hunks.

## In Scope

- Active-battle world/session state needed for employee working conditions and processed-death
  identity.
- Reconciliation of newly dead player runtime instances into the working state exactly once.
- Deployment preview, eligibility, and actual draft creation from working employee state.
- Result commit from working conditions without replaying death penalties at completion.
- Employee-level survival XP, outcome rows, and battle result statistics across redeployments.
- Focused repeated-death, trauma-aware redeploy, persistent-death lockout, victory/defeat commit, and
  aggregation tests.
- Canonical rulebook/runtime wording that still asserts fixed 60% or result-time death penalties.

## Explicit Non-Scope

- G15's full victory/defeat/abandonment/failure transition table and exact whole-world snapshot
  restore.
- BossOmen Gate/checkpoint recovery and Unity recovery UX.
- New DTO/RON/save shape, balance constants, authored penalty values, or event/reward timing.
- Deep deployment staging unrelated to employee-condition authority.

## Planned Implementation

1. Trace the current battle entry, death reconciliation, deployment, result application, and stats
   paths; classify every target-file dirty hunk.
2. Add focused failing tests for fixed-60% redeploy and duplicate per-instance result behavior.
3. Introduce one world-owned attempt employee-condition ledger seeded from battle-entry roster and
   one idempotent processed-incapacitation set keyed by runtime unit identity.
4. Reconcile each tactical death into working condition before projection/deployment, and drive
   redeploy preview/commit from that employee state.
5. Commit working conditions once on non-abandonment resolution and aggregate non-penalty outputs by
   employee without erasing instance replay history.
6. Remove fixed-60% and delayed duplicate incapacitation paths, update canonical documents, and run
   focused/broad validation.
7. Perform the completion review and unlock G15 only for Complete / High with no immediate
   correction.

## User-Visible Completion Conditions

- Two deaths of the same employee apply two and only two penalty bundles, even across redeployments.
- The first redeployment reflects the first death's current Run HP/trauma; no 60% overwrite occurs.
- Persistent death or equivalent non-deployability in working state blocks later redeployment.
- Victory and defeat commit accumulated condition once; no result-time duplicate penalty occurs.
- Survival XP, employee changes/result rows, and MVP contain one coherent employee-level outcome.
- Instance-level events/results remain usable for replay and diagnostics.

## Planned Validation

1. Focused G26 unit/world tests for repeated death, idempotence, redeploy, commit, and aggregation.
2. Existing withdrawal/redeploy and post-battle employee resolution tests.
3. Related battle result statistics tests.
4. `cargo check -q -p game_core`.
5. Relevant Core integration suites, then `cargo test -q -p game_core --all-targets -- --test-threads=1`.
6. `cargo fmt --all -- --check`, scoped `git diff --check`, and fixed-60%/legacy searches.
7. Server/Unity validation only if the traced implementation affects their contract; no mock-only
   proof is accepted.

## Stop Conditions

Stop if G26 needs a new public DTO/RON/save schema, new penalty/balance/failure/rollback meaning, an
unrelated unclassifiable dirty overlap, or a change to Core/server/Unity authority.
