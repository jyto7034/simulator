# Phase 01 Authority And Strict-Data Foundation

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Establish one coherent Core/server/Unity authority foundation for battle revision ordering,
context capability and payload eligibility, reconnect/resync recovery, and exact generated live-data
publication before Phase 2 changes combat transactions.

## Required Resume Read Set

- `../../STATUS.md`
- this file
- `EXPERIMENT_NOTES.md`
- the active Child Goal's three documents
- the direct predecessor or correction review named by that Child

Phase experiments and completed Child histories are read only for Phase review or a concrete
cross-child conflict.

## Child Order And Status

| Child/review | Status | Review | Role |
| --- | --- | --- | --- |
| G01 `battle_projection_revision_contract` | complete | Complete / High | battle revision and cursor/order authority |
| G06 `contextual_action_capability_projection` | complete | Complete / High | common capability/preflight authority |
| G07 `battle_capability_and_reconnect_authority` | correction_required | Partially complete in R01 | battle eligibility and recovery authority |
| G12 `gameplay_ron_strictness_and_bundle_validation` | complete | Complete / High | exact live-data publication |
| R01 `cross_layer_phase_01_authority_completion_review` | complete | Complete / High after correction rerun | Phase integration review |
| G30 `battle_capability_execution_authority_correction` | complete | Complete / High | MSTR-05 and MSTR-06 correction |

## Accepted Authority

- PD-01: `battle_uuid + battle_state_revision` is the mutation authority and revisioned projection
  order. Event sequence and battle time keep their delivery/simulation meanings.
- PD-02: Core owns typed context capability and domain payload eligibility; projection and execution
  share validators and execution revalidates atomically.
- Missing and stale mutation authority remain distinct typed errors but both receive the current
  non-mutating resync recovery batch.

No new DTO, RON, save, UX, balance, failure, reward, or rollback decision is authorized by G30.

## Phase Invariants

- Every accepted battle projection mutation commits exactly one checked revision.
- No omitted or disabled battle capability is executable by a direct client at that revision.
- Query and setup-loss recovery remain available with their typed scopes during blocking battle VN.
- VN advance and choice progression are mutually exclusive.
- Payload handlers revalidate concrete employee/tile/skill/target state after context preflight.
- Missing and stale authority reject before mutation and emit correlated error then current resync.
- live gameplay RON is exact-version, strictly validated, and published all-or-nothing.
- Unity consumes Core authority and does not recreate a static/local action whitelist.

## Completed Correction Loop

1. G30 implemented and reviewed the shared Core battle-context evaluator/preflight.
2. G30 implemented and reviewed the server missing/stale recovery builder.
3. R01 is rerun without overwriting the first failed review evidence.
4. Phase 1 completes only if the rerun is Complete with no immediate correction.

## User-Visible Exit Conditions

- A handcrafted client cannot bypass a blocking battle VN or any disabled battle context action.
- Rejected commands change no gameplay state, event, cost, RNG, or battle revision.
- Missing and stale authority recover deterministically without automatic mutation retry.
- Core, game_server, and Unity expose and consume the same current projection.
- Focused, broad, exact-fixture, and required Unity MCP validation pass.

## Explicit Non-Scope

- Phase 2 employee/result/abandonment behavior.
- New public DTO or live RON fields.
- Deep staging of `BattleCore::deploy_player_unit` after already validated world inputs.
- G08 player-facing recovery UX.
- Unrelated dirty-worktree cleanup.

## Stop Conditions

Stop only if implementation requires a new public DTO/RON/save/UX/gameplay policy, an ownership
conflict with another active Goal, or modification of an unrelated/unclassifiable dirty overlap.
