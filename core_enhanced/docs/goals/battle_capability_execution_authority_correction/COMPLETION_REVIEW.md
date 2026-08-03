# Battle Capability Execution Authority Correction Completion Review

Review date: 2026-07-14

Review procedure: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

## Item Audit Table

| Item | Evidence status | Verdict | Long-term fit | Immediate correction |
| --- | --- | --- | --- | --- |
| MSTR-05 battle context execution authority | One Core evaluation list drives projection and execution preflight; every blocking-VN-omitted command rejects before handler mutation | Complete | High | None |
| MSTR-06 missing/stale recovery symmetry | One server builder preserves distinct errors and emits correlated error then current resync for both paths | Complete | High | None |

## MSTR-05 — Battle Context Capability Is A Core Execution Gate

Master item:

An action omitted or disabled by the current revisioned battle context must not be executable by a
direct client merely because it supplies current battle UUID/revision.

Policy source:

- Accepted PD-01 battle revision authority.
- Accepted PD-02 two-level context capability plus concrete payload eligibility.
- R01-H-01 required correction.

Expected behavior:

One Core-owned battle-context evaluation seam supplies the revisioned projection and execution
preflight. Context-irrelevant actions reject before domain mutation. Context-relevant parameterized
families continue into their existing concrete payload preflight so cooldown, resource, and target
reasons remain precise. Authority validation remains first and handlers revalidate payloads.

Runtime evidence:

- `src/game/world/battle_capability.rs` builds one canonical action/evaluation list from active
  battle, playback, deployment, skill readiness, finished state, and current VN.
- The same file maps that list to `ActionCapabilityDto` and consumes it in
  `preflight_battle_capability_behavior`.
- `src/game/world.rs` preserves authority validation first, then composes common and battle
  preflight before domain dispatch.
- Blocking VN maps deploy, preview, withdraw, activation, playback, speed, and retreat to
  context-irrelevant. Query/recovery plus exactly one valid VN progression remain enabled.
- Context-disabled parameterized actions proceed only to the existing domain payload validator;
  the tests prove exact cooldown/resource/target reasons and zero mutation remain intact.

Data evidence:

- No live RON, schema, loader, content, or save path changed.
- The evaluator derives current runtime facts and does not add an authored action table.

External contract evidence:

- No serialized DTO changed. The actual typed Rust battle fixture remains exactly equal to Unity's
  checked-in fixture.
- Unity production capability/store/command code is unchanged and still consumes the current
  revisioned Core projection without a local whitelist.
- Canonical `action_capabilities` wording already required execution-time Core preflight and now
  matches runtime.

Test evidence:

- New Core test submits deploy, deployment preview, withdraw, skill, Pause, Resume, speed, and
  `RetreatBattle` during blocking VN. Every command returns typed context rejection and the complete
  serialized current projection remains byte-semantically equal before/after.
- Existing blocking-VN group passed 3/3.
- Existing playback, stale-authority, deploy-cooldown, missing-target, and insufficient-resource
  focused tests passed 5/5 and preserve two-level reason ownership.
- Core all-targets passed 711 nonzero tests, 0 failed/ignored.

Legacy/fallback audit:

- Production searches found no `allowed_actions`, `ActionScheduler`, `ActionValidator`, or Unity
  `battleRuntimeActions` authority.
- No handler-specific blocking-VN conditional, second cached action list, compatibility field,
  fallback, ignored test, or test-only production branch was added.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: projection and execution now share one domain-owned evaluator while preserving the
  separate concrete payload validator layer required by PD-02.

Verdict:

- Complete.

Remaining risk:

- None for battle context authority. Final targeting semantics remain with G09/G10 as before.

## MSTR-06 — Missing And Stale Authority Share Current Resync Recovery

Master item:

Both missing and stale mutation authority must reject before mutation with distinct correlated
typed errors and the same current authoritative resync recovery.

Expected behavior:

The actor maps either typed Core error through one recovery builder, projects current battle state
without mutation, and emits response error followed by one current `battle_resync`. Unity does not
invent authority or retry the mutation.

Runtime evidence:

- Core authority validation remains before capability/payload work and returns distinct
  `MissingBattleStateAuthority` and `StaleBattleStateAuthority` errors.
- `game_server/src/game/player_game_actor/handlers.rs` routes the two error variants through
  `battle_authority_rejection_result`.
- The helper uses the read-only active-battle reconnect projection and constructs one
  `CommandExecutionResult` with correlated error, one resync side message, and no state snapshot.
- Existing actor emission sends response before side messages, preserving exact error→resync order.

Data evidence:

- No live data or save behavior changed.

External contract evidence:

- `game_server/docs/unity_bridge_protocol.md` and the two external Unity canonical contracts now
  describe missing/stale symmetry and distinct codes.
- Unity production already applies the proactive same-revision authoritative resync. It retains a
  defensive stale request path but does not add a missing-authority fallback or automatic retry.

Test evidence:

- Paired server tests passed 2/2. Both assert distinct correlated error, exactly one current resync,
  unchanged revision/playback, and delivered `[Error, BattleResync]` ordering.
- New Unity MCP focused EditMode test passed 1/1. It proves missing error creates no local resync
  request, then applies the proactive same-revision server resync and repairs checkpoint state.
- game_server all-targets passed 28 nonzero tests, 0 failed/ignored.
- Unity MCP EditMode passed 189/189.
- Unity MCP PlayMode passed with 51 passed, 0 failed, and 2 explicit live-server skips.

Legacy/fallback audit:

- No stale-only server special case remains.
- Missing and stale codes remain distinct; no authority fabrication, automatic mutation retry,
  client fallback, dual message schema, or ignored test was added.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: one actor recovery seam now owns both protocol-equivalent rejection paths while Core and
  Unity retain their existing responsibilities.

Verdict:

- Complete.

Remaining risk:

- None inside G30.

## Cross-Component Findings

- Core authority validation, battle-context evaluation, domain payload validation, server recovery,
  and Unity application now form one ordered chain without a competing authority source.
- Blocking-VN direct commands cannot rely on honest Unity behavior for safety.
- Missing/stale recovery does not change DTO shape, revision, event cursor semantics, or gameplay.
- Strict live-data publication is unaffected.
- The dirty worktree remains broad, but every G30 hunk was scoped to classified predecessor seams;
  no unrelated or unclassifiable hunk was overwritten.

## Validation Results

- Core focused direct-command: 1/1 passed.
- Core blocking-VN group: 3/3 passed.
- Core focused playback/stale/payload reason tests: 5/5 passed.
- Server focused missing/stale recovery: 2/2 passed.
- `cargo check -q -p game_core -p game_server`: passed.
- Core all-targets: 711 passed, 0 failed, 0 ignored.
- game_server all-targets: 28 passed, 0 failed, 0 ignored.
- Typed Rust fixture exact equality: passed.
- Unity MCP focused EditMode: 1/1 passed.
- Unity MCP full EditMode: 189/189 passed.
- Unity MCP PlayMode: 51 passed, 0 failed, 2 explicit live-server skips.
- Final Unity Console error audit after clearing test-runner output: 0 entries.
- `cargo fmt --all -- --check`: passed.
- Core/server and Unity scoped `git diff --check`: passed.
- New untracked Core file whitespace check: no diagnostics; exit 1 was expected because it differs
  from `/dev/null`.
- Production legacy-authority and relevant ignored-test searches: no matches.

Failed experiments retained in `EXPERIMENTS.md`:

- One zero-test `--exact` filter was rejected as evidence.
- The pre-correction direct-command and missing-authority tests each reproduced their finding.
- The first evaluator revision masked `already_active` and concrete payload reasons; it was corrected
  and all affected focused tests were rerun.
- The first formatting check failed on rustfmt wrapping; formatting and the same check then passed.

## Immediate Correction Candidates

None.

## Follow-Up Refactor Candidates

- R01's separate deep deploy staging/rollback candidate remains out of scope. It is not required for
  the validated world-input path or either G30 finding.

## Policy Questions

None.

## Review Conclusion

G30 closes MSTR-05 and MSTR-06 with Complete / High long-term fit and no immediate correction. R01
must now rerun the Phase 1 integration review before Phase 2 starts.
