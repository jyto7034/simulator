# Cross-Layer Phase 01 Authority Completion Review

Status: complete

Implementation complete: true

Created: 2026-07-13

Master goal: `docs/goals/cross_layer_contract_hardening_master/PLAN.md`

Review ID: R01

Required completed children:

- G01 `battle_projection_revision_contract` — Complete / High
- G06 `contextual_action_capability_projection` — Complete / High
- G07 `battle_capability_and_reconnect_authority` — Complete / High
- G12 `gameplay_ron_strictness_and_bundle_validation` — Complete / High

Guide: `docs/goal_completion_review_guide.md`

## Objective

Independently verify the complete Phase 1 authority foundation across Core, live RON, game_server,
and Unity before Phase 2 begins. Confirm that revision ordering, common and battle capability,
payload eligibility, stale/reconnect recovery, and strict live-data publication compose without
competing authority, legacy fallback, transport gaps, or unreviewed partial paths.

## User-Visible Problem

Each child can pass in isolation while their combined seams still disagree—for example, a capability
may be projected under one revision but executed under another, reconnect may restore state without
the matching eligibility, or server/Unity may bypass Core authority. Likewise, a strict live-data
loader is only foundational if every runtime consumer receives the validated bundle and later layers
do not reconstruct content validity.

## Working Method

At every start/resume, read this `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md` completely
before code search, modification, or validation. Then apply `docs/goal_completion_review_guide.md`
using this evidence order:

1. Actual Core runtime creation/mutation/projection/preflight paths.
2. Production live RON loader, live data, and final publication boundary.
3. game_server mapping, ordering, request correlation, reconnect, and resync.
4. Actual Unity reader/store/command/presenter behavior and Rust-generated fixtures.
5. Canonical runtime/data/transport documents.
6. G01/G06/G07/G12 goal documents and completion reviews.

Do not treat child completion labels as proof. If runtime/data/Unity evidence disagrees, R01 fails
until a dedicated correction is implemented and reviewed.

## Phase 1 Expected Contract

- `battle_state_revision` is the battle-UUID-scoped authoritative commit order for complete battle
  state; event sequence and battle time retain their separate delivery/simulation meanings.
- Root/common action capability is Core-generated typed context capability. Concrete payload
  eligibility comes from the owning domain and shares pure preflight validators with execution.
- Every active battle mutation requires exact battle UUID/revision and revalidates atomically.
- game_server preserves ordering and correlation, rejects stale/missing authority before mutation,
  and returns typed error followed by current resync state.
- Unity consumes Core capability/eligibility/revision and does not substitute static whitelists,
  local gameplay inference, revisionless mocks, or fallback DTO fields.
- Blocking battle VN retains only typed query/setup-loss recovery plus valid progression.
- Production gameplay RON is exact-version, canonical, fully validated, and returned all-or-nothing;
  game_server does not bind on failure and Unity does not reproduce RON validity.
- No legacy `allowed_actions`, cached action table, static battle action list, revision fallback,
  lenient live-data loader, alias, partial bundle, or ignored compatibility test remains.

## In Scope

- Revalidate each child against actual runtime/data/server/Unity code and live RON.
- Trace one representative mutation from Unity command through server to Core preflight/commit and
  back through revisioned projection/resync.
- Trace reconnect/blocking-VN recovery and confirm capability/eligibility share the committed
  revision.
- Trace game_server startup to the exact live-data publication boundary.
- Search all four layers for old authority names, static/local substitutes, fallback paths, dual
  schemas, stale mocks, ignored tests, and competing loaders.
- Compare canonical documents with actual contracts.
- Run the smallest focused and broad validation set that proves the combined boundary, including the
  available Unity MCP instance.
- Produce `COMPLETION_REVIEW.md` using the guide's item and cross-component structure.

## Explicitly Out Of Scope

- Phase 2 employee attempt-state, abandonment, BossOmen, or recovery implementation.
- New Event/Shop/inventory/targeting/compendium policy or DTO work.
- Refactoring child internals merely for style when the long-term authority boundary is already
  coherent.
- Silently implementing broad corrections discovered by review. Record a precise immediate
  correction candidate and keep R01 incomplete unless the correction is separately authorized and
  reviewed.

## Review Items

1. G01 battle revision and event cursor/order authority.
2. G06 common context capability and current-state preflight authority.
3. G07 battle eligibility, expected revision, reconnect/resync, and Unity consumption.
4. G12 exact-version live-data validation and all-or-nothing startup publication.
5. Cross-component composition among all four items.
6. Canonical documentation and regression evidence.

## Completion Conditions

- Every review item has runtime/data/external/test/legacy evidence and an explicit verdict.
- Cross-component tracing proves no revision/capability/data authority gap or competing source.
- Old authority/fallback/alias/ignored-test searches are clean or every hit is classified.
- Focused integration, Core all-targets, game_server all-targets, and required Unity MCP validation
  pass with nonzero counts.
- Canonical documents describe the current combined contract.
- `COMPLETION_REVIEW.md` verdict is Complete, long-term fit High, improvement class none, and no
  immediate correction is recommended.
- Only then may Master Phase 1 become complete and Phase 2 become ready.

## Planned Validation

- Exact Rust-generated Unity fixture comparison/probe paths inherited from G01/G06/G07.
- Focused Core revision/capability/stale/reconnect/live-RON tests with nonzero counts.
- `cargo test -q -p game_core --all-targets -- --test-threads=1`.
- `cargo test -q -p game_server --all-targets -- --test-threads=1`.
- Unity MCP EditMode and applicable PlayMode tests.
- `cargo fmt --all -- --check` and Core/server all-target checks.
- Scoped legacy/fallback/alias/ignored-test and competing-loader searches.

Exact commands and counts will be recorded only after the R01 document gate and source review.

## Stop Conditions

Stop and keep R01 incomplete if review finds:

- a policy decision not settled by PD-01/PD-02/PD-09,
- a material authority gap or compatibility path requiring implementation,
- a required Unity/server runner unavailable without equivalent evidence,
- a child completion claim contradicted by runtime/data behavior,
- a Phase 2 change required to make Phase 1 coherent.

Report the exact current behavior, affected files/tests, user-visible risk, recommended correction
goal, and whether Phase 1 remains blocked.

## Review Outcome

- Initial 2026-07-13 review: Partially complete / Medium / immediate correction recommended.
- G30 corrected R01-H-01 and R01-H-02 and passed Complete / High review.
- 2026-07-14 correction rerun: Complete / High / no immediate correction.
- Phase 1 passes and Phase 2 may begin.
