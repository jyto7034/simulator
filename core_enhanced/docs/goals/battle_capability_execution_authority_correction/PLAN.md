# Battle Capability Execution Authority Correction

Status: complete

Implementation complete: true

Created: 2026-07-14

Child ID: G30

Primary findings: MSTR-05, MSTR-06

## Required Resume Read Set

1. `../cross_layer_contract_hardening_master/STATUS.md`
2. `../cross_layer_contract_hardening_master/phases/phase_01_authority/PLAN.md`
3. `../cross_layer_contract_hardening_master/phases/phase_01_authority/EXPERIMENT_NOTES.md`
4. this file
5. `EXPERIMENTS.md`
6. `EXPERIMENT_NOTES.md`
7. `../cross_layer_phase_01_authority_completion_review/COMPLETION_REVIEW.md` sections
   `R01-H-01`, `R01-H-02`, and `Final Verdict`

## Objective

Make the revisioned Core battle capability a real execution preflight and make missing/stale battle
authority share one ordered, non-mutating server resync recovery seam, without changing the public
DTO or gameplay policy.

## User-Visible Problems

- Honest Unity hides ordinary battle actions during a blocking battle VN, but a direct current-
  authority client can bypass that presentation gate because Core dispatch does not compose the
  battle context evaluator before handlers.
- A stale mutation request receives typed error plus current resync, while a mutation request with
  missing authority receives only an error and cannot deterministically recover from the response.

## Accepted Source-Of-Truth Boundary

- Core owns current battle context relevance/enabled state and payload validation.
- game_server owns correlated ordered delivery of error and current resync.
- Unity strictly consumes the Core/server result and does not recreate battle authority.
- PD-01 and PD-02 are already accepted; this Goal does not reopen them.

## Repository Baseline

| Repository | Branch/HEAD | Dirty fingerprint | Initial classification |
| --- | --- | --- | --- |
| Core workspace | `main` / `c2c5dc5` | `89d722e7...` | heavily dirty; target overlap audit required |
| game_server | `main` / `c2c5dc5` | `4c8314f7...` | same workspace plus server target overlap audit required |
| Unity | `master` / `7cd8df1` | `2b690122...` | heavily dirty; likely prior G01/G06/G07 plus unrelated UI changes |

Do not edit target files until their current scoped diffs are classified as completed predecessor
work, current G30 continuation, unrelated user change, or unclassifiable overlap.

## In Scope

### Core

- One pure battle-context evaluator with relevant/enabled/typed-reason output.
- Capability projection and command preflight share that evaluator.
- Preflight runs before battle handler mutation and preserves downstream payload revalidation.
- Direct-command negative coverage for deploy, withdraw, activate skill, pause/resume, speed,
  `RetreatBattle`, and deployment preview during blocking VN.
- Rejection proves unchanged gameplay state, event log, resource cost, RNG-relevant state, and
  battle revision.
- Projection/preflight reason parity and mutually exclusive VN progression.

### game_server

- One recovery builder/path for `MissingBattleStateAuthority` and `StaleBattleStateAuthority`.
- Preserve distinct typed error codes and request correlation.
- Emit error then current `battle_resync` without mutation or revision change.
- Add paired ordering/no-mutation coverage for missing authority.

### Unity

- Verify consumption of the server's missing-error then proactive resync sequence.
- Add no local missing-authority fallback unless current runtime evidence proves server recovery is
  insufficient.
- Use the already-open Unity MCP for focused and broad validation.

## Explicit Non-Scope

- New Unity-facing DTO, RON schema, save migration, UX, balance, failure, reward, or rollback policy.
- Handler-specific blocking-VN conditionals or a second battle action table.
- Automatic retry of rejected mutation intent.
- Deep internal deploy staging after world validation.
- Phase 2 features or unrelated dirty-worktree cleanup.

## Implementation Steps

1. Classify scoped dirty diffs and trace the current Core projection→gate→handler path, server
   error mapping/order, and Unity error/resync application.
2. Add failing focused Core tests that submit every omitted action during blocking VN and capture
   the exact unchanged authority state.
3. Implement the shared Core evaluator/preflight and make projection consume the same result.
4. Add failing server missing-authority recovery coverage, then implement one missing/stale builder.
5. Add/adjust Unity consumption coverage only at the actual changed recovery seam.
6. Run focused checks after each coherent change, then the applicable broad ladder.
7. Apply the completion-review guide and write `COMPLETION_REVIEW.md`.
8. Hand the result to R01 for the Phase rerun.

## Completion Conditions

- Every battle action omitted or disabled by current context rejects before mutation when submitted
  directly with current authority.
- Projected and execution-preflight status/reason come from one evaluator.
- Query/setup-loss recovery and the valid VN progression action remain reachable.
- Missing and stale requests both produce correlated typed error then current resync, with distinct
  codes and no mutation.
- Core/server exact behavior tests and required Unity MCP tests pass.
- No compatibility alias, fallback, second action authority, or test-only production branch remains.
- Child completion review returns Complete / High with no immediate correction.

## Verification Ladder

1. Focused Core capability/execution parity and blocking-VN direct-command tests.
2. Focused server missing/stale error→resync ordering tests.
3. Related Core/server integration tests and `cargo check`.
4. Exact Rust fixture comparison if serialized output changes.
5. Unity MCP focused EditMode test, full EditMode, and applicable PlayMode.
6. Core and game_server all-targets.
7. rustfmt, scoped diff checks, legacy/fallback/ignored-test searches.

## Stop Conditions

Stop and report if G30 requires a public DTO/RON/save/UX/gameplay-policy change, conflicts with an
unrelated or unclassifiable dirty overlap, or cannot run mandatory Unity validation.
