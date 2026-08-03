# Battle Capability And Reconnect Authority Completion Review

Review date: 2026-07-13

Review procedure: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

## Item Audit Table

| Item | Evidence status | Verdict | Long-term fit | Immediate correction |
| --- | --- | --- | --- | --- |
| DCA-H-11 Unity static battle whitelist | Revisioned Core capabilities and domain payload eligibility are consumed strictly by Unity; static/local execution authority is removed | Complete | High | None |
| DCA-H-12 blocking battle VN recovery | Blocking VN projects only valid VN progression while query and setup-loss recovery remain reachable and revision coherent | Complete | High | None |
| DCA-M-12 server bridge helper contradiction | Canonical bridge documentation now matches typed battle side messages and ordered stale recovery | Complete | High | None |
| PD-01 expected revision and reconnect integration | Every battle mutation requires exact battle UUID/revision; stale commands do not mutate and receive ordered current resync | Complete | High | None |
| PD-02 battle payload eligibility | Playback, deployment, withdrawal, and skill payloads use typed Core eligibility and strict Unity consumption | Complete | High | None |

## DCA-H-11 — Core-Owned Battle Capability And Payload Authority

Master item:

Remove Unity's static battle action substitution and make the complete revisioned Core projection
the sole battle capability and payload-eligibility authority.

Subgoal:

`docs/goals/battle_capability_and_reconnect_authority/`

Policy source:

- Master accepted PD-02 two-level capability contract.
- G06 completed the common capability types and root boundary.
- G07 PLAN accepted battle-domain projection and selected-payload preflight ownership.

Expected behavior:

Every complete battle update contains context capabilities for its exact battle revision. Bounded
payload options live in their owning playback/deployment/withdraw/skill DTOs. Unity renders those
typed values and does not authorize through a static action set, local resource calculation, local
tile/affinity rule, or hidden fallback. Large employee/tile/facing and skill/target combinations use
selected Core preview/preflight instead of an unbounded Cartesian projection.

Runtime evidence:

- `src/game/world/battle_capability.rs` projects battle context capabilities from current battle,
  playback, deployment, skill, and blocking-VN state.
- `src/game/world/state.rs` builds typed playback, deployment candidate, withdrawal, and required
  per-deployed-unit skill readiness eligibility from current authoritative state.
- `src/game/world/combat.rs` returns selected deployment preview `deploy_eligibility` from the same
  Core validation boundary used before mutation.
- `src/game/behavior.rs` owns the required revisioned `action_capabilities`, playback eligibility,
  payload eligibility, selected preview eligibility, and required deployed-unit skill readiness
  serialization shape.
- Projection and execution retain atomic Core revalidation; an enabled snapshot is not treated as
  an authorization token.

Data evidence:

- G07 changes no live RON schema or authored balance/content.
- Broad Core all-target validation includes current production live-RON loading and passed. No
  derived capability or eligibility list was moved into authored data.
- Active battle projection is not a saved-run migration boundary; no save migration is required.

External contract evidence:

- `../game_server/src/game/player_game_actor/messages.rs` transports the typed complete battle
  projection without rebuilding capability policy.
- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/BattleRuntimeState.cs` requires and
  atomically validates battle capabilities plus playback/deploy/withdraw/skill eligibility before
  applying a revision. Missing or inconsistent shapes request resync instead of locally enabling or
  hiding an action.
- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Battlefield/BattlefieldPendingDeploymentRangePreview.cs`
  requires selected Core `deploy_eligibility` before deployment confirmation.
- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameSnapshotStore.cs` keeps root
  non-battle capabilities separate from the active revisioned battle projection.
- `/mnt/f/unity projects/ark/Assets/Scripts/ArkClient/Runtime/Core/GameCommandBus.cs` submits the
  selected payload but does not recompute Core execution eligibility.
- `../game_server/examples/generate_battle_revision_fixture.rs` exactly reproduces the checked-in
  Rust fixture consumed by Unity.

Test evidence:

- Core focused tests cover Pause/Resume/current-speed exclusivity, blocking-VN capability, deploy
  cost/redeploy lock, withdrawal, selected deployment preview, and targetless manual-skill
  `no_valid_target` eligibility.
- Unity MCP EditMode: 188/188 passed, including actual Rust fixture consumption, current-speed and
  payload-aware presentation, selected Core deployment preview, and strict malformed projection
  rejection.
- The final exact Unity MCP malformed-payload test passed 1/1 after all edits. It checks six invalid
  cases: missing playback eligibility, duplicate speed, playback-state disagreement, missing deploy
  candidate eligibility, missing required skill readiness, and skill boolean/eligibility mismatch.
- Unity MCP PlayMode: 51 passed, 0 failed, 2 explicit live-server skips.
- Core all-targets: 670 nonzero tests passed, 0 failed. game_server all-targets: 27 nonzero tests
  passed, 0 failed.

Legacy/fallback audit:

- No active `battleRuntimeActions`, local deployment affordability/execution validator, optional
  deployed skill readiness, free-form skill block reason, root battle-capability fallback, serde
  alias, dual schema, or revision fabrication remains.
- Old local occupied-tile and affinity execution-authority tests were replaced with selected Core
  preview contract tests rather than preserved or ignored.
- Relevant ignored-test search across Core, server, and Unity returned no matches.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: the change removes the battle-specific client authority split, composes the completed G01
  revision and G06 capability seams, keeps bounded eligibility with its domain DTO, and uses selected
  Core preflight for combinatorial payloads. It introduces no parallel cache or compatibility path.

Verdict:

- Complete.

Remaining risk:

- Final skill targeting and range semantics remain intentionally owned by G09/G10. G07 exposes only
  current executable readiness and does not pre-decide those later policies.

## DCA-H-12 — Blocking Battle VN Query And Recovery Reachability

Master item:

Prevent blocking battle VN from hiding reconnect and setup-loss recovery while ordinary battle
gameplay remains blocked.

Expected behavior:

A blocking battle VN pauses ordinary simulation/gameplay and exposes only the currently valid VN
progression family, but current battle query and setup-loss recovery remain query/recovery scoped.
Recovery returns the immutable setup and complete current revisioned projection without consuming a
revision or inventing capability state.

Runtime evidence:

- `src/game/world/battle_capability.rs` separates gameplay, query, and recovery scopes. Blocking VN
  suppresses ordinary gameplay while retaining `RequestBattleState` and
  `RecoverBattleSetupLoss` and exactly one valid VN progression form.
- `src/game/world/combat.rs` preserves read-only state/resync and setup-loss recovery semantics.
- G01 revision/cursor ownership remains unchanged: Core owns projection commits and immutable event
  history; server connection state owns delivery ordering/cursor.

External contract evidence:

- game_server emits setup/current battle side messages without replacing them with a root snapshot
  capability list.
- Unity applies the returned complete update using battle UUID/revision and keeps recovery/query
  separate from mutation authority.
- Canonical Core/server/Unity documents describe blocking-VN progression, setup-loss recovery, and
  same-revision repair consistently.

Test evidence:

- Core `blocking_visual_novel_pauses_live_battle_tick_and_uses_battle_update_transport` proves the
  blocker, valid progression, query/recovery scope, and revisioned transport.
- Core `live_defense_setup_loss_recovery_preserves_battle_revision_and_state` proves recovery is
  non-mutating and coherent.
- Unity focused reconnect/resync/capability tests and the complete EditMode suite pass.
- The actual live WebSocket probe validates entry, explicit resync, complete capability/eligibility,
  and continued battle command flow against a running Core/server.

Legacy/fallback audit:

- No coarse all-gameplay battle whitelist remains to accidentally hide recovery or enable both VN
  progression forms.
- Query and recovery do not fabricate or require mutation authority.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: scope is part of the typed Core projection and reuses the revisioned recovery path instead
  of adding a VN exception or a Unity bypass.

Verdict:

- Complete.

Remaining risk:

- G08 still owns the final player-facing recovery/roster UX. That does not create another execution
  authority and is not required to close the G07 transport contract.

## PD-01 — Expected Battle Revision And Stale Recovery

Expected behavior:

Every active battle mutation carries the battle UUID and the exact revision from which the player
acted. Missing or stale authority rejects before any mutation. A stale server request produces the
correlated typed error followed by the complete current `battle_resync` in one ordered actor
boundary. Queries and recovery omit mutation authority and do not consume revision.

Runtime evidence:

- `src/game/behavior.rs` owns `BattleStateAuthorityDto` and typed missing/stale errors.
- `src/game/world.rs` classifies battle mutations and checks exact current UUID/revision before
  common/domain validation or mutation.
- `../game_server/src/game/player_game_actor/messages.rs` places optional typed authority only on the
  command envelope.
- `../game_server/src/game/player_game_actor/handlers.rs` maps stale rejection to ordered
  `error -> battle_resync` without mutation.
- Unity `GameCommandBus.cs` adds current runtime authority only for mutation action families;
  `RequestBattleState` remains authority-free.

Test evidence:

- Core `supplied_stale_battle_authority_rejects_before_playback_mutation` checks both missing and
  stale rejection plus unchanged state/revision before a current-authority success.
- Server `stale_battle_authority_emits_error_then_current_resync_without_mutation` checks exact
  message order and unchanged current projection.
- Unity `StaleBattleAuthorityErrorRequestsCurrentRevisionResync` checks resync request behavior
  without local revision mutation.
- The live smoke probe naturally observed an automatic-tick race, received typed stale error plus
  current resync, retried the same intent under a new request ID/current revision, and exited 0 on
  two fresh server runs.

Legacy/fallback audit:

- No event sequence, timestamp, default revision, server-side current-revision substitution, or
  automatic mutation retry is used as freshness authority.
- Missing authority is a distinct typed error; it is not silently repaired.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: one envelope precondition protects every mutation at the Core boundary and composes with
  the existing G01 commit revision and server actor delivery ordering. No second revision or
  per-behavior duplicate authority was introduced.

Verdict:

- Complete.

Remaining risk:

- A rendered revision can become stale during automatic ticks by design. The tested recovery path
  preserves intent safety; UI may choose how to present a retry later without weakening authority.

## DCA-M-12 — Canonical Server Bridge And Side-Message Contract

Expected behavior:

The helper contract distinguishes non-battle root snapshot behavior from revisioned battle side
messages, documents mutation authority, and states exact stale error/resync ordering.

Evidence:

- `../game_server/docs/unity_bridge_protocol.md` now documents the top-level `battle_authority`,
  revisioned battle capability/payload projection, setup/update/resync side messages, and ordered
  `stale_battle_state_authority -> battle_resync` path.
- `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`,
  `/mnt/f/unity projects/ark/docs/unity_core_contract.md`, workflow/implementation guides, and the
  runnable probe contract agree with the actual typed transport.
- Runtime/server/Unity searches found no active contradictory `allowed_actions` battle authority;
  the remaining Unity occurrence is a negative removal assertion.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: canonical documentation follows the proven runtime and live probe rather than creating a
  compatibility interpretation.

Verdict:

- Complete.

Remaining risk:

- None inside G07.

## Cross-Component Findings

- Core, game_server, Unity, canonical documents, generated fixture, and live probe use the same
  `battle_uuid + battle_state_revision` mutation authority.
- Complete battle capability and payload eligibility are committed under that same revision; no
  component caches a competing executable action list.
- Root capabilities remain the G06 non-battle authority and are never used as an active-battle
  fallback.
- Event sequence remains delivery continuity only; battle time remains simulation/presentation only.
- Blocking VN, selected deployment preview, setup-loss recovery, stale resync, and ordinary mutation
  paths exercise the same typed boundary across components.
- Live RON is unaffected and broad loading remains green. No old authored schema is being silently
  interpreted as capability authority.
- No conflict with G01 or G06 completion boundaries was found.

## Validation Classification

- Core broad: `cargo test -q -p game_core --all-targets -- --test-threads=1` — 670 nonzero passed.
- Server broad: `cargo test -q -p game_server --all-targets -- --test-threads=1` — 27 nonzero passed.
- Unity MCP EditMode: 188 passed, 0 failed, 0 skipped.
- Unity MCP PlayMode: 51 passed, 0 failed, 2 explicit live-server skips.
- Exact Rust fixture equality: passed.
- Actual live Core/server WebSocket probe: two fresh runs exited 0, including a real stale-tick race.
- `cargo fmt --all -- --check`: passed.
- Scoped Core, server, and Unity `git diff --check`: passed.
- Unity MCP Editor compilation state: idle/not compiling; Console error audit: 0.
- Relevant ignored-test and active legacy-name searches: no material matches.
- Zero-test groups and the failed workspace-root server launch are explicitly excluded from success
  evidence.

## Follow-Up Refactor Candidates

- None inside G07. Continue with the already planned domain owners instead of expanding this closed
  authority boundary.

## Policy Questions

- None.

## Review Conclusion

G07 satisfies DCA-H-11, DCA-H-12, DCA-M-12 and the battle portions of PD-01/PD-02. Runtime,
transport, Unity, fixture, live probe, tests, and canonical contracts agree; no material legacy or
fallback path remains. G07 may be marked complete with High long-term fit and no immediate
correction. Phase 1 remains active and the next implementation child is G12, followed by R01.
