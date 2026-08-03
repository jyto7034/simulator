# Phase 01 Authority Completion Review

Reviewed: 2026-07-13

Guide: `docs/goal_completion_review_guide.md`

Initial verdict (2026-07-13): Partially complete

Initial long-term fit: Medium

Initial improvement class: immediate correction recommended

Initial Phase 1 exit gate: failed

Current verdict after correction rerun (2026-07-14): Complete

Current long-term fit: High

Current improvement class: none

Current Phase 1 exit gate: passed

## Item Audit Table

| Item | Child | Verdict | Fit | Immediate correction |
| --- | --- | --- | --- | --- |
| Battle revision and event cursor/order authority | G01 | Complete for normal/reconnect paths | High | none in reviewed normal path |
| Common context capability and current-state preflight | G06 | Complete | High | none |
| Battle capability/eligibility execution parity | G07 | Partially complete | Medium | required |
| Missing/stale authority recovery | G07 | Partially complete | Medium | required |
| Exact-version live-data publication | G12 | Complete | High | none |
| Cross-component Phase 1 composition | G01/G06/G07/G12 | Partially complete | Medium | required |

## G01 Battle Revision And Cursor Authority

Master item: battle-UUID-scoped authoritative revision, read-only projection, and server-owned event
delivery cursor.

Expected behavior: every accepted mutation validates exact UUID/revision first, commits one new
revision, and returns a complete projection; reconnect and resync do not mutate Core or fabricate
revision/event/time.

Runtime evidence:

- `src/game/world.rs` validates `BattleStateAuthorityDto` before common/domain preflight or mutation.
- `src/game/world/state.rs` owns checked projection revision increment and read-only
  `battle_update_dto_after(after_seq)`.
- Reviewed pause, resume, speed, deploy, withdraw, skill, tick, and battle-VN projection paths check
  commit capacity, mutate, commit once, then build the checkpoint.
- `game_server/src/game/player_game_actor` owns `BattleEventDeliveryCursor` and emits command response
  then side messages through one actor/socket path.
- Active-battle attach resets only the connection cursor and sends state snapshot, immutable setup,
  and current revisioned update without changing Core.
- Unity ordinary update applies only a higher revision; authoritative same-revision resync has a
  distinct strict path.

External evidence:

- `GameCommandBus` attaches the store's current battle UUID/revision to battle mutations.
- Actual Rust `battle_revision_contract.json` exactly matches the current typed generator and is
  consumed by Unity EditMode tests.

Test evidence:

- Focused playback revision, setup-loss/reconnect preservation, and stale-before-mutation tests each
  passed 1/1.
- Core all-targets and server all-targets passed.

Legacy/fallback audit:

- No Core delivery cursor, time/seq freshness authority, revisionless mock fallback, or destructive
  reconnect path remains in production.

Long-term direction review:

- Fit: high for the reviewed normal/reconnect paths.
- Improvement class: none for this item.

Verdict: Complete.

Remaining risk:

- `BattleCore::deploy_player_unit` appends a scenario group before all fallible spawn operations
  finish. Current validated caller inputs make the later error path exceptional, but staging or
  rollback would strengthen deep atomicity. This is a follow-up refactor candidate, not the reason
  Phase 1 fails.

## G06 Common Capability Authority

Master item: replace `allowed_actions`, cached validator state, and state-only scheduler authority
with typed context capability plus shared current-state preflight.

Expected behavior: Core projects context relevance/enabled state and reuses the same evaluator before
execution; server is transparent; Unity cannot recreate absent common actions.

Runtime evidence:

- `src/game/world/action_capability.rs` owns common evaluation, projection, and preflight.
- `src/game/world.rs` invokes common preflight before the owning domain handler.
- Root snapshot carries only typed `action_capabilities`.
- Unity `ActionCapability.Parse` strictly checks action/scope/enabled/reason and `GameCommandBus`
  gates on the store's authoritative capability.

Test evidence:

- Focused common map/checkpoint capability parity and state-transition revalidation each passed 1/1.
- Actual Rust `action_capability_contract.json` exactly matches the typed generator and is consumed
  by Unity EditMode tests.

Legacy/fallback audit:

- No production `allowed_actions`, `ActionValidator`, `ActionScheduler`, old admin dump, or common
  Unity fallback was found. The one Unity `allowed_actions` test occurrence is a negative assertion
  proving removal.

Long-term direction review:

- Fit: high.
- Improvement class: none.

Verdict: Complete.

Remaining risk: concrete inventory/Event/Shop payload DTOs remain with their registered later domain
goals and do not invalidate the common seam.

## R01-H-01 — Battle Capability Is Not A Core Execution Gate

Master item: G07 battle capability/eligibility authority, especially blocking battle VN.

Expected behavior: an action omitted or disabled by the current revisioned battle capability cannot
execute even when a direct client sends a syntactically valid command with current UUID/revision.
Projection and execution must share the same battle-context evaluator.

Runtime evidence:

- `src/game/world/battle_capability.rs` omits DeployUnit, WithdrawUnit, ActivateSkill, pause/resume,
  speed, retreat, and deployment preview during a blocking battle VN; it retains query, setup-loss
  recovery, and the valid VN progression action.
- `src/game/world/action_capability.rs` deliberately returns domain-owned `false` for battle actions.
- `src/game/world.rs` then dispatches those battle actions directly to handlers.
- The reviewed handlers revalidate payload/revision but do not reject because a blocking VN omitted
  their context capability.

External contract evidence:

- Honest Unity is safe: the store uses the battle update capability and `GameCommandBus.CanSend`
  prevents the hidden mutation.
- Unity cannot be the authority. A handcrafted client with current UUID/revision bypasses the UI gate
  and Core currently accepts at least pause/deploy-class handlers if their payload is otherwise valid.

Test evidence:

- Existing blocking-VN tests assert projected actions and paused ticking, but do not submit hidden
  battle mutations and assert Core rejection/no mutation/same revision.
- Broad green suites therefore do not cover this authority gap.

Legacy/fallback audit:

- This is not a remaining static whitelist or compatibility field. It is a missing Core preflight
  composition between the new projector and execution.

Long-term direction review:

- Fit: medium until corrected.
- Improvement class: immediate correction recommended.
- Reason: current behavior makes Unity's honest-client gate security-relevant and violates PD-02's
  Core authority contract.

Verdict: Partially complete.

Required correction:

- Add one pure battle context capability evaluator/preflight shared by projection and execution.
- Gate battle commands before handler mutation using relevant/enabled/typed-reason results; do not
  add handler-specific `if blocking_vn` patches or a second action table.
- Preserve query/setup-loss recovery and mutually exclusive VN progression.
- Revalidate payloads atomically in handlers after the context gate.
- Add Core tests proving every omitted blocking-VN mutation and preview query rejects with no state,
  event, cost, or revision change; enabled/disabled reason parity must match projection.
- Add server and Unity contract tests only where the corrected error/resync behavior crosses layers.

## R01-H-02 — Missing Authority Does Not Receive Current Resync

Master item: G07 missing/stale authority recovery.

Expected behavior: both missing and stale battle authority reject before mutation with a correlated
typed error and current authoritative resync, enabling deterministic recovery.

Runtime evidence:

- Core correctly returns distinct `MissingBattleStateAuthority` and `StaleBattleStateAuthority` before
  mutation.
- `game_server/src/game/player_game_actor/handlers.rs` special-cases only stale authority to emit
  error followed by current `battle_resync`.
- Missing authority falls through the ordinary error path and receives no current resync.

External contract evidence:

- Unity defensively requests resync only for `stale_battle_state_authority`; a missing-authority
  response is displayed as an error and relies on no automatic recovery.
- Server proactive resync is the preferred single recovery guarantee; a second client fallback is not
  required to repair the contract.

Test evidence:

- Focused stale error→current resync without mutation passed 1/1.
- No paired server test proves missing authority emits the same recovery batch.

Long-term direction review:

- Fit: medium until corrected.
- Improvement class: immediate correction recommended.
- Reason: error typing exists, but recovery composition is asymmetric and contradicts the accepted
  missing/stale contract.

Verdict: Partially complete.

Required correction:

- Route both missing and stale authority through one server recovery builder that emits correlated
  typed error then current `battle_resync` without mutation.
- Preserve distinct error codes; do not convert missing to stale or fabricate authority.
- Add server ordering/no-mutation/same-revision tests for missing authority and Unity application
  coverage for the resulting error→resync sequence.

## G12 Strict Live-Data Publication

Master item: exact-version canonical gameplay RON and all-or-nothing startup publication.

Runtime/data evidence:

- `src/game/data/live_ron.rs` owns schema version 1, manifest plus 23-source parsing, source-aware
  diagnostics, local/domain/cross/map/RunStart/Event/extra-wave validation.
- `GameDataBase::load_live_embedded()` assembles a private candidate, validates generated
  preview/briefing contracts, and returns only success.
- game_server main maps failure to startup `InvalidData` before bind and hands the same Arc to player
  actors.
- Unity does not interpret RON schema or content validity.

Test evidence:

- Live-RON focused 41/41 passed.
- Core/server broad suites passed.

Legacy/fallback audit:

- Production search found only main's live loader plus a use inside server test helpers. No alternate
  production builder, alias, ignored legacy test, partial bundle, or map-catalog reader remains.

Long-term direction review:

- Fit: high.
- Improvement class: none.

Verdict: Complete.

## Cross-Component Findings

- Normal battle mutation composition is coherent: Unity capability/payload → UUID/revision envelope
  → server cursor → Core authority/payload validation → one revision commit → ack then update →
  strict Unity apply.
- Reconnect, same-revision resync, root common capability, and strict live-data startup compose without
  a competing source.
- R01-H-01 breaks the Core-authority guarantee specifically when a battle context capability is
  omitted/disabled but a direct client submits it.
- R01-H-02 breaks deterministic recovery specifically for missing authority.
- Historical goal/audit documents retain superseded terms as evidence; current canonical Core/data/
  server/Unity contracts are clean.
- Master finding status drift for DCA-H-08, DCA-M-13, DCA-M-18, and DCA-M-19 shared scope was corrected
  during this review.

## Validation Results

- Typed Rust fixture exact comparisons: battle revision and common capability passed; 0 tests by
  design, provenance evidence only.
- Corrected Core focused filters: 7 nonzero tests passed (capability serialization, stale filter, and
  five explicit revision/capability flows); live-RON 41/41 passed.
- game_server focused: 5/5 passed.
- Core all-targets: 710/710 passed, 0 failed/ignored.
- game_server all-targets: 27/27 passed, 0 failed/ignored.
- Unity MCP EditMode: 188/188 passed, 0 failed/skipped.
- Unity MCP PlayMode: result Passed; 51 passed, 0 failed, 2 explicit live-server skips.
- Final Unity editor state: idle, not compiling/playing, ready for tools; error console 0 entries.
- rustfmt, Core/server all-target checks, Python probe syntax, repository-scoped diff checks, and
  production/canonical legacy searches passed.
- Two initial focused-filter attempts executed 0 tests and are explicitly recorded as failures, not
  evidence.

## Immediate Correction Candidates

Create one bounded correction goal for the G07 authority seam:

1. Shared Core battle context projector/preflight execution gate.
2. Missing/stale server error→current-resync symmetry.
3. Matching Core/server/Unity behavior tests and actual typed fixture regeneration only if serialized
   output changes.

Do not start Phase 2 before that correction passes its own completion review and R01 is rerun.

## Follow-Up Refactor Candidates

- Stage or rollback `BattleCore::deploy_player_unit`'s scenario/spawn mutation so exceptional internal
  errors cannot leave a partially appended scenario group. Current validated live deployment inputs
  make the risk remote; it does not replace the two immediate corrections.

## Initial Final Verdict — 2026-07-13

- Verdict: Partially complete.
- Long-term fit: Medium.
- Improvement class: immediate correction recommended.
- Phase 1: remains active.
- Phase 2: blocked.
- Policy questions: none; both immediate findings are implementation gaps under accepted PD-01/PD-02.

## Correction Rerun — 2026-07-14

Correction child:

- G30 `battle_capability_execution_authority_correction`
- Review: `docs/goals/battle_capability_execution_authority_correction/COMPLETION_REVIEW.md`
- Verdict: Complete / High / no immediate correction.

### Corrected Item Audit

| Item | Previous verdict | Current evidence | Current verdict | Fit |
| --- | --- | --- | --- | --- |
| R01-H-01 / MSTR-05 battle context execution gate | Partially complete | One Core battle evaluation list drives projection and execution preflight; direct blocking-VN command matrix rejects without changing complete projection | Complete | High |
| R01-H-02 / MSTR-06 missing authority recovery | Partially complete | One server recovery builder emits distinct missing/stale error then current resync; paired server and Unity tests pass | Complete | High |
| G01 revision/cursor/order foundation | Complete | Authority-first order, unchanged revision semantics, exact Rust fixture, broad tests remain green | Complete | High |
| G06 common capability foundation | Complete | Common fixture exact match and two focused current-state/preflight tests pass after battle-gate composition | Complete | High |
| G07 battle capability/reconnect foundation | Partially complete through R01 | G30 closes both integration gaps without changing DTO or adding fallback | Complete | High |
| G12 strict live-data publication | Complete | live-RON focused 42/42 and broad startup consumers pass unchanged | Complete | High |

### R01-H-01 Correction Evidence

- `src/game/world/battle_capability.rs` now owns one canonical battle-context evaluation list.
- Projection maps that list to typed capability; execution preflight consumes the same evaluation.
- `src/game/world.rs` preserves exact authority validation first, then common and battle context
  preflight, then domain payload validation/mutation.
- Context-irrelevant blocking-VN commands reject before handler access. Context-relevant
  parameterized families still use their concrete payload validator and preserve precise cooldown,
  resource, and target reasons.
- The new Core regression submits deploy, preview, withdraw, activation, Pause, Resume, speed, and
  `RetreatBattle` with current authority. Every command rejects and the complete serialized current
  projection is identical before/after.

Verdict: Complete / High / no immediate correction.

### R01-H-02 Correction Evidence

- `game_server/src/game/player_game_actor/handlers.rs` routes both typed authority errors through
  `battle_authority_rejection_result`.
- Missing and stale retain distinct error codes and correlation, then emit exactly one current
  `battle_resync` through the existing response-before-side-message actor order.
- Paired server tests prove no playback/revision mutation and exact `[Error, BattleResync]` delivery.
- Unity keeps server-driven recovery authoritative: missing error does not invent a local retry or
  resync request, and the following same-revision authoritative resync repairs current state.
- Server bridge and external Unity canonical contracts now describe the symmetric behavior.

Verdict: Complete / High / no immediate correction.

### Current Validation Results

- Core direct-command regression: 1/1 passed.
- Core blocking-VN group: 3/3 passed.
- Core playback/stale/concrete-payload focused tests: 5/5 passed.
- Common capability focused tests: 2/2 passed.
- strict live-RON focused: 42/42 passed.
- server missing/stale authority recovery: 2/2 passed.
- Core all-targets: 711 passed, 0 failed, 0 ignored.
- game_server all-targets: 28 passed, 0 failed, 0 ignored.
- Exact Rust battle revision and common capability fixture comparisons: passed.
- Unity MCP focused missing recovery: 1/1 passed.
- Unity MCP full EditMode: 189/189 passed.
- Unity MCP PlayMode: 51 passed, 0 failed, 2 explicit live-server skips.
- `cargo check -q -p game_core -p game_server`: passed.
- `cargo fmt --all -- --check`: passed.
- Simulator/Unity scoped diff checks, production legacy-authority search, and relevant ignored-test
  search: passed.
- Final Unity Console error audit: 0 entries.

### Correction Cross-Component Pass

- Core projection, execution context gate, concrete payload validation, server correlation/order,
  and Unity application now use one responsibility chain without a static/local substitute.
- Revision, event cursor, battle time, query/recovery scope, and strict live-data semantics did not
  change during correction.
- No DTO/RON/save migration, compatibility alias, fallback path, second action table, automatic
  mutation retry, or ignored test was introduced.
- The R01 non-blocking deep deploy staging candidate remains a separate follow-up; it is not a
  current validated-caller or Phase 1 authority failure.

### Current Final Verdict

- Verdict: Complete.
- Long-term fit: High.
- Improvement class: none.
- Immediate correction candidates: none.
- Policy questions: none.
- Phase 1 exit gate: passed.
- Phase 2: may begin under the Master dependency order.
