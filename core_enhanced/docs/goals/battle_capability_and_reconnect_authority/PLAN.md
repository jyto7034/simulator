# Battle Capability And Reconnect Authority Goal

Status: complete

Implementation complete: true

Created: 2026-07-13

Master goal: `docs/goals/cross_layer_contract_hardening_master/PLAN.md`

Master child: G07

Primary findings:

- DCA-H-11
- DCA-H-12
- DCA-M-12

Accepted policies: PD-01 and PD-02

Predecessors:

- G01 `battle_projection_revision_contract` — Complete / High / no immediate correction
- G06 `contextual_action_capability_projection` — Complete / High / no immediate correction
- R00 baseline review — Complete / High

## Objective

Make Core's complete revisioned battle projection the only authority for battle context capability,
payload eligibility, mutation preconditions, and reconnect/resync recovery. Remove Unity's remaining
battle-only static action substitution and synchronize the server/helper contract without changing
battle gameplay policy owned by later goals.

## User-Visible Problem

Unity currently substitutes a static battle command set instead of rendering Core authority, so
Pause/Resume, speed, deployment, withdrawal, skill, VN, query, and recovery availability can be
wrong for the current revision. A blocking battle VN can also hide setup-loss/reconnect recovery.
Commands sent from an older view do not yet carry the accepted expected battle revision, so a valid
button from a stale snapshot may mutate newer authoritative state.

## Goal Working Method

At every start/resume, read this goal's `PLAN.md`, `EXPERIMENTS.md`, and
`EXPERIMENT_NOTES.md` completely before code exploration, modification, or validation. Also retain
the G01/G06 completion boundaries and the Phase 1/R00 review gates.

Use this evidence order after the document gate:

1. Core battle mutation/query/recovery entry points and revision commits.
2. Live battle RON/data only where eligibility depends on authored content.
3. Rust battle setup/update/resync/ack/root DTOs and exact serialization.
4. game_server command mapping, expected-revision validation, ordering, attach, and resync.
5. Unity battle reader/store/command bus/presenter/reconnect behavior through the open MCP Editor.
6. Current canonical battle transport and Unity contracts.
7. Master/child history.

## Accepted Contract

- Battle capability and payload eligibility are part of the complete projection committed under one
  `battle_state_revision` for one `battle_uuid`.
- Core owns relevance, enabled/disabled state, typed reason, and domain eligibility. Projection and
  execution share pure validators; execution revalidates atomically.
- Every battle mutation command carries its expected battle UUID/revision. Stale authority returns a
  typed stale result plus the current authority/resync path and performs no mutation.
- Read-only battle queries and resync do not consume a revision. Recovery stays recovery-scoped and
  remains reachable during blocking battle VN.
- Pause and Resume are mutually exclusive. Selecting the active speed is disabled or a proven
  non-mutating result and never increments revision.
- Battle VN exposes only valid progression for the current scene: Advance or concrete choices.
- Remove Unity's static battle capability set and all local battle eligibility inference. No alias,
  fallback, dual schema, or revision fabrication is authorized.
- Preserve G01 ordering: Core owns checked commit revision; server connection owns ordered event
  delivery cursor/outbox; Unity orders by battle UUID/revision and uses event seq only for continuity.

## Source Of Truth And Ownership

- Core battle session plus pure battle validators own capability and eligibility at a committed
  revision.
- Rust typed battle projection owns exact serialized shape.
- game_server transports expected revision and typed stale/recovery results without policy.
- Unity renders current battle capability/eligibility and submits the selected typed payload with
  the current expected authority; it does not authorize locally.
- G08 owns final player-facing recovery/roster UX after G14/G15 policy is stable.
- G03 owns Event employee checks; G15/G26 own result/abandonment/death semantics; G09/G10 own final
  skill targeting/range policy. G07 may expose current executable eligibility without changing
  those policies.

## Invariants

- One committed battle revision describes checkpoint, playback, deployment, skill readiness, battle
  VN, context capabilities, and payload eligibility coherently.
- An enabled projected payload executes against unchanged battle UUID/revision; disabled projection
  and execution share the same typed reason.
- A stale expected UUID/revision cannot mutate state, events, resources, or revision.
- Query/resync/setup-loss recovery remain reachable according to their typed scopes even when
  gameplay is blocked by VN or setup loss.
- Pause/Resume and VN Advance/Choice never appear simultaneously when mutually exclusive.
- Repeating current speed consumes no revision.
- Unity has no static/local substitute for Core battle authority after cutover.
- G01 reconnect ordering, same-revision resync, gap handling, and cursor ownership remain unchanged.

## In Scope

- Trace all battle action kinds, current validators, mutation commits, payload identities, root/setup/
  update/resync/ack production, server mapping, and Unity battle presentation/submission.
- Define typed battle context capabilities and bounded payload eligibility/preflight projections
  using G06 types/reasons, extending typed reason vocabulary only when current behavior justifies it.
- Include capability/eligibility in the revisioned complete battle projection.
- Add expected battle UUID/revision to battle mutation commands and typed stale authority response/
  resync path.
- Integrate reconnect, setup-loss recovery, blocking battle VN, Pause/Resume, speed, deployment,
  withdrawal, skill activation, and relevant battle queries.
- Remove Unity `battleRuntimeActions` and local battle eligibility inference.
- Correct the current server bridge/helper documentation to match side-message ordering and the final
  typed projection.
- Produce an actual Rust battle capability fixture consumed by Unity.
- Add focused parity/stale/reconnect tests and broad Core/server/Unity validation.

## Explicitly Out Of Scope

- New targeting profiles, enemy range preview policy, global skill/range migration: G09/G10/G11.
- Event choice/employee/d20 flow: G03.
- Employee death/trauma/redeployment transaction: G26.
- Victory/defeat/abandonment and BossOmen recovery/gate policy: G15/G14.
- Final recovery/roster player UX: G08.
- New balance values, skill content, RON schema, save migration, or compatibility support.
- Generic untyped payload bags or projection of unbounded target Cartesian products.

## Current Historical Evidence To Revalidate

- G06 left one explicitly battle-scoped static Unity action set as the G07 boundary.
- G01 established required `battle_state_revision` on complete battle updates/resync/ack and same-
  revision recovery ordering, but expected-revision command preconditions remain G07.
- Blocking VN and battle setup-loss recovery may currently share a coarse gameplay gate.
- Server helper documentation may still describe state snapshot delivery in a way that contradicts
  battle side messages.
- Current deployment/skill/target checks likely occur after action-family selection and require an
  ownership matrix before selecting bounded projections versus selected-payload preflight.

These are hypotheses until current runtime, live data, DTO, server, and Unity paths are read.

The initial semantic Warp Grep attempt against Core, game_server, and Unity returned HTTP 429 for
all three repositories and produced no source evidence. Continue with the assigned read-only Warp
Grep subagent plus bounded `rg`/file reads; do not treat the failed search as validation.

## Current Implementation Checkpoint

Revalidated on 2026-07-13:

- `LiveBattleUpdateDto` commits revision, events, checkpoint, deployment, playback, and battle VN,
  but contains no battle `action_capabilities` or typed payload eligibility.
- Core mutation handlers revalidate concrete deploy/withdraw/skill payloads and commit revisions.
  Pause/Resume/current-speed requests are currently accepted as non-mutating same-revision results
  when already in the requested state; projection does not distinguish them.
- `LiveBattleDeploymentDto` already exposes finite employee costs, deployed/redeploying units, and
  per-deployed-unit skill readiness. Skill readiness uses booleans plus free-form string reasons,
  while deployment cell validity remains computed separately by Core handlers.
- No player battle behavior or server command envelope carries expected battle UUID/revision. Core's
  public execute seam currently carries behavior, source command ID, and server event cursor only.
- Server attach/order/cursor behavior matches completed G01. The client command envelope owns only
  `request_id`, `behavior`, and optional `battle_response`; it has no stale-authority precondition.
- Unity `GameSnapshotStore` discards root capabilities whenever a battle runtime exists and returns
  synthetic enabled entries from `battleRuntimeActions`. The set omits setup-loss recovery and
  deployment-range query while enabling both VN actions and every playback/speed family.
- Unity locally derives deployment employee/tile/affinity/occupancy/cost eligibility in
  `BattlefieldDeploymentModel`/`BattleActionHandler`. `BattleScreenComposer` chooses Pause/Resume
  from checkpoint state but enables every speed button, and skill buttons use coarse action plus
  local skill-ID presence rather than Core readiness eligibility.
- `docs/unity_bridge_protocol.md` still calls the root state an `allowed_actions` authority and must
  be corrected with the final side-message/revisioned capability contract.
- Core/server/Unity worktrees contain extensive pre-existing and G01/G06 changes in the same files.
  Preserve them and use scoped patches/diffs; unrelated deleted/generated files are not G07 scope.
- Focused baseline is green: Core `live_defense_` 9/9, game_server `live_battle` 5/5, and Unity
  MCP selected battle authority/reconnect tests 6/6. Two Unity tests explicitly protect the static
  runtime action substitution/current local presenter semantics and must be replaced by the new
  Core-authoritative contract rather than mechanically renamed.
- The first expected-authority trial added the shared typed authority and an optional Core transport
  context check only; it is not yet the mandatory cutover. Its first focused compile failed because
  two exhaustive snapshot error formatters lacked the new stale-authority variant. This is a bounded
  compile correction, not a policy issue.
- After that correction, the first rerun matched 0 tests because `--exact` was used with only the
  unqualified function name. It is not success evidence; rerun with a matching filter.
- The corrected non-exact focused rerun executes 1/1 and proves supplied stale authority rejects
  before playback/revision mutation while the exact current authority commits once. Mandatory
  missing-authority enforcement and server/Unity transport are still pending.
- Server production `cargo check` passes after threading optional envelope authority to Core. The
  first server test compile reports 10 test-only `ExecutePlayerBehavior` literals missing the new
  field; migrate those fixtures explicitly and rerun rather than adding a default/fallback struct.
- All 10 server test literals are now explicit, server library test compilation passes, and the
  live battle request transport test passes 1/1 with command-envelope authority preservation.
- The first revisioned capability slice adds the field to `LiveBattleUpdateDto` and a centralized
  final projection hook. Its first focused command reported rustfmt-only diffs and then the playback
  test failed during pre-run start with `InvalidAction`, before reaching the new battle projection.
  Root cause is not yet established; do not attribute success or change policy.
- Isolated rerun reproduced 0/1. Root cause is now established: the central attachment function
  computed battle capability before matching whether the result was a battle result, so non-battle
  RunStart incorrectly required an active battle. Move computation inside the battle-result arm.
- Lazy battle-only projection fixes the regression. Focused playback and blocking-VN tests pass 1/1
  each and now prove Pause/Resume exclusivity plus blocking-VN projection limited to Advance,
  battle query, and setup-loss recovery.
- Server library compile after adding the top-level field reports four explicit test migrations: one
  BattleUpdate pattern and three typed fixture constructors. Update them to assert/preserve the new
  field; do not ignore the public contract mechanically.
- The four server fixtures/patterns are migrated, library test compilation passes, and three battle
  update/setup/resync serialization tests pass. The top-level update asserts capability preservation;
  the real actor-flow pattern asserts an enabled Pause capability at revision 1.
- Unity production now parses battle capability atomically with each applied update/resync and the
  static compatibility set/helper is removed. MCP compilation is clean. The first four focused tests
  fail because their hand-built update helper lacks the now-required field, confirming strict
  no-fallback behavior; fixture migration is in progress.
- Migrated shared Unity battle update fixtures and replaced the static-whitelist expectation. The
  same four MCP tests now pass 4/4. Mutation commands now include the exact applied battle UUID/
  revision at the top-level envelope while RequestBattleState omits it; focused MCP tests pass 2/2.
- Stale authority now returns one ordered actor batch `error -> battle_resync` with unchanged current
  revision/capability, verified server 1/1. Unity converts that typed error into a current-seq resync
  request without changing applied revision, verified through MCP 1/1.
- Active battle mutation authority is now mandatory in Core; missing and stale authority reject
  before mutation. Focused Core missing/stale/playback/VN tests pass, and the server pause/resume
  actor test passes with exact revision handoff. Two accidental unused fixture bindings found by the
  compiler were removed.
- First broad Core library run after mandatory cutover executes 629: 611 pass and 18 fail. All 18
  are old direct mutation test calls missing authority (17 combat, 1 map-flow); no runtime behavior
  failure beyond the new precondition is indicated. Migrate them through the explicit test helper,
  with the overflow test supplying current max authority so it still reaches overflow validation.

## Implementation Plan

1. Record dirty-worktree overlap and trustworthy G01/G06 baselines.
2. Trace each battle command from Unity submission through server mapping, Core validation/mutation,
   revision commit, serialization, and Unity apply/render.
3. Build a battle action/payload ownership matrix with current pure validators and reason sources.
4. Run small read-only/focused trials for projection shape and expected-revision placement.
5. Implement Core revisioned battle capability/eligibility and shared pure preflight.
6. Add expected UUID/revision command preconditions and typed stale/resync behavior without changing
   G01 commit or server cursor ownership.
7. Migrate server transport and remove stale helper-contract wording.
8. Migrate Unity battle store/command/presenter and remove the static set/local inference atomically.
9. Generate an actual Rust fixture and consume it through Unity EditMode.
10. Run focused parity, stale, VN, playback, reconnect, resync, and setup-loss recovery tests.
11. Run broad Core/server/Unity tests plus formatting, diff, legacy, live-data, and canonical audits.
12. Apply `docs/goal_completion_review_guide.md` before marking G07 complete.

## User-Visible Completion Conditions

- Unity battle controls reflect only the current Core revisioned capability/eligibility projection.
- Pause/Resume, current-speed, and VN progression presentation match current authoritative state.
- Deployment, withdrawal, and skill actions expose typed current eligibility without Unity resource/
  target inference.
- A stale battle command performs no mutation and leads to current-authority resync.
- Query and setup-loss/reconnect recovery remain correctly scoped and reachable during gameplay
  blockers.
- Reconnect/resync preserves exact battle UUID/revision and capability/eligibility.
- Actual Rust serialization is consumed by Unity; mocks do not define the contract.
- No static battle whitelist, local capability substitute, dual schema, alias, or fallback remains.
- G01/G06 regressions and broad Core/server/Unity suites pass with nonzero counts.
- Completion review is Complete / High / no immediate correction.

## DTO, Data, And Migration

- Unity-facing DTO: expected revisioned battle capability/eligibility and expected-revision command
  fields are already accepted by PD-01/PD-02; exact placement must be derived from current typed
  transport and recorded before implementation.
- Live RON/data: no change expected. Stop if capability correctness requires schema/content policy.
- Save migration: none expected; active battle projection is not a cross-version saved-run payload.
- Compatibility: not authorized.

## Planned Verification Categories

- Pure battle projector/preflight parity for enabled and disabled contexts/payloads.
- Mutation-between-projection-and-command stale UUID/revision rejection with no side effects.
- Pause/Resume, current speed, and battle VN exclusivity/revision participation.
- Deployment/withdraw/skill representative enabled and disabled payloads.
- Query/recovery scope and blocking-VN setup-loss/reconnect flows.
- Exact Core/server serialized update/resync/ack and actual Rust Unity fixture.
- Unity no-static-substitution presentation and command payload tests.
- G01 ordering/gap/same-revision resync regressions.
- Full Core all-targets, game_server all-targets, Unity EditMode/PlayMode, formatting, diff, legacy,
  and canonical/probe audits.

Exact commands and counts will be recorded after current test entry points are inspected.

Mandatory-authority migration evidence: 611/629 -> 626/629 -> 628/629 -> 629/629. Production
missing/stale rejection remained strict throughout; only test transport call sites were migrated.

Payload implementation boundary now selected from current runtime: typed eligibility is embedded in
the revisioned owning domain DTO (deployment candidate/deployed unit/skill readiness/playback), while
large employee×tile×facing and skill×target combinations remain selected Core preview/preflight.
Stable reasons and actual handlers must share the same validator; free-form skill block strings and
Unity-side execution inference are removal targets.

Focused parity is now proven for insufficient stabilization, redeploy cooldown, targetless manual
skill, and already-active playback/speed. Server production and both server test executables compile
with explicit payload eligibility; Unity consumption and removal of local inference remain active.

Unity payload cutover is focused-green: editor compilation has 0 errors and six nonzero focused tests
pass. Revisioned updates are strict, current-speed/insufficient-cost/skill/withdraw UI uses Core
eligibility, and deploy Confirm requires selected Core preview eligibility. Full fixture migration,
actual Rust JSON consumption, and broad suites remain before completion.

Full EditMode migration baseline: 187 executed, 167 passed, 20 failed. All failures are classified as
stale mock/fixture expectations or strict-shape rejection; no accepted current payload semantic
regression is known.

Current EditMode migration state: 187 executed, 177 passed, 10 failed after shared mock and actual
Rust fixture updates. Remaining failures are bounded stale fixture/obsolete local-authority tests.

Unity EditMode migration is complete at 187/187 with 0 compile errors. Actual typed Rust JSON,
revisioned capability/eligibility, payload-aware UI, and selected Core deploy preview are covered.
PlayMode plus broad Core/server validation remain active.

Broad current results: Core library 629/629, server 27/27, Unity EditMode 187/187, Unity PlayMode
51 passed / 2 explicit live-server skips / 0 failed, editor compile 0 errors. All-targets and final
diff/canonical/review audits remain.

All-targets are now green: Core 670 nonzero tests, server 27 nonzero tests. Final diff, legacy,
canonical/probe, and completion-review audits remain.

Final completion checkpoint:

- Core owns complete revisioned battle capabilities, bounded domain payload eligibility, selected
  deployment preflight, and exact mutation UUID/revision validation.
- game_server transports the typed authority and emits ordered stale error then current resync
  without adding policy or a second freshness key.
- Unity consumes only the current revisioned battle projection, requires well-formed payload
  eligibility, submits exact mutation authority, and retains no static/local execution substitute.
- Actual Rust fixture equality, two live WebSocket probe runs, Core 670, server 27, Unity EditMode
  188/188, Unity PlayMode 51 pass/2 explicit live-server skips, formatting, scoped diff, legacy,
  ignored-test, canonical, and Console audits all passed.
- `COMPLETION_REVIEW.md` verdict: Complete / High / no immediate correction.
- G07 is closed. Phase 1 remains active; G12 is the next child before R01.

## Stop Conditions

Set `Status: policy_decision_required` and stop before implementing any newly discovered decision
about DTO placement beyond the accepted revision/capability contract, live RON, save migration,
targeting/range semantics, balance, result/retreat/reward meaning, employee death/redeployment,
recovery UX, content deletion, or compatibility. Record safe technical details in scope and defer
unrelated improvements to the owning child.
