# Battle Failure, Abandonment, And Consumable Resolution Completion Review

Goal: G15

Review date: 2026-07-14

Guide: `docs/goal_completion_review_guide.md`

Overall verdict: Complete

Overall long-term fit: High

Immediate correction: none

## Audit Table

| Item | Verdict | Fit | Immediate correction |
| --- | --- | --- | --- |
| DCA-M-17 typed immediate recovery predicate | Complete | High | none |
| DCA-M-20 attempt-consuming/exhausted abandonment | Complete | High | none |
| DCA-M-31 consumable result timing | Complete | High | none |
| DCA-L-04 incapacitation terminology | Complete | High | none |
| CPR-H-02 fatal committed defeat priority | Complete | High | none |
| CPR-H-09 complete pre-entry abandonment restore | Complete | High | none |

## DCA-M-17 — Typed Immediate Recovery Predicate

Master item:

Recovery-node eligibility was undefined, could use weak future reachability, and allowed checkpoint
availability to stand in for a gameplay recovery capability.

Policy source:

PD-11 and the G15 accepted transition table allow nonboss `NoDeployableEmployees` deferral only for
currently enterable nodes with explicit Treatment or Recruitment capability. Current content has no
Treatment payload. Recruitment exists only where HeadquartersContact can produce live candidates.

Expected behavior:

A committed no-deployable result consumes its combat node, then either exposes only immediately
selectable typed recovery nodes or enters RunFailed. Rest, SavePoint, Shop, labels, future nodes, and
checkpoint presence do not qualify. Completing the one recovery node rechecks deployability.

Runtime evidence:

- `src/game/world/state.rs` owns an internal `CombatRecoveryCapability` and one run-local
  `CombatRecoveryRestriction`; it does not add a public map/RON/save field.
- `src/game/world/combat.rs` evaluates the live node category/payload, authored candidate count, and
  recruitment catalog, derives candidates only from the post-completion selectable set, and applies
  the same rule to victory and defeat.
- `selectable_map_node_ids` is shared by projection, capability, and command validation.
- `src/game/world/node_flow.rs` masks nonqualified available nodes in `MapViewDto`, blocks checkpoint
  load during restriction, and rechecks the roster after recovery-node completion.
- `src/game/world/helpers.rs`, `action_capability.rs`, and `snapshot.rs` all consume the shared
  restriction rather than implementing parallel category/label heuristics.

Data evidence:

- Live `SupportNodeType` exposes only SavePoint and Rest; neither is treated as Treatment.
- The evaluator requires both positive HeadquartersContact `candidate_count` and a nonempty live
  recruitment catalog. It invents no Treatment schema or authored fallback.

External contract evidence:

- Existing `map_navigation.selectable_node_ids`, map node state, action capabilities, and
  `SelectMapNode` are reused without a DTO shape change.
- External `unity_core_contract.md` now states the recovery restriction, `action_locked`
  checkpoint capability, and post-node re-evaluation. Unity remains a consumer of Core projection.

Test evidence:

- Focused tests prove recruitment-only projection/validation, successful recruitment, failed
  supplies re-evaluation, victory parity, future-HQ/Rest rejection, zero-candidate rejection, and
  checkpoint action locking: 5/5 passed.
- Core library 678/678 and final all-targets 719 passed.

Legacy/fallback audit:

- `can_recover_no_deployable_with_checkpoint` is removed. No Safe/Support label check, future graph
  scan, Shop inference, public Treatment placeholder, or secondary selection validator remains.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: one typed internal predicate feeds every projection and validation consumer, while future
  Treatment can be added at the capability seam only when an authored schema is actually accepted.

Verdict:

- Complete.

Remaining risk:

- Treatment content remains intentionally absent and requires a later schema decision; it is not a
  missing G15 implementation.

## DCA-M-20 — Exact Non-Consuming Abandonment

Master item:

`RetreatBattle` consumed attempts, and its exhausted branch fabricated a Draw, decremented
consumables, recorded combat, completed the node, or failed a boss run.

Expected behavior:

Every battle-stage abandonment restores the authoritative state from immediately before entry and
returns to NodeConfirm. It creates no result, record, node completion, reward, penalty, discovery,
or attempt-exhaustion branch.

Runtime evidence:

- `src/game/world/state.rs` defines `BattleEntrySnapshot` over every battle-mutable gameplay owner:
  game/node/run state, content/session, UUID/RNG continuation, resources, inventory, roster/order,
  starter/pending state, skill fragments, VN/trigger state, and run checkpoint.
- `src/game/world/node_flow.rs` captures the map-combat snapshot before progression/attempt/UUID
  mutation.
- `src/game/world/combat.rs` installs that snapshot on the active battle and has one retreat path:
  take active battle, restore snapshot, and emit a read-only NodePreview.
- Event-started combat builds a reentry-form snapshot after the Event choice/immediate effects have
  committed, preserving that irreversible choice while discarding only the battle attempt.

Data evidence:

- No balance, encounter, reward, or live RON change is needed; this is world transaction ownership.

External contract evidence:

- External `unity_core_contract.md` distinguishes the active-battle provisional attempt projection
  from restored NodeConfirm values and removes the old third-retreat `NodeCompleted` contract.
- The explicit live Unity flow now asserts `attempts_started == 0` and
  `remaining_attempts == 3` after first-entry retreat.

Test evidence:

- Exact serialized pre-entry/after-retreat equality and restored battle UUID continuation: 1/1.
- Four repeated normal retreats and final-boss retreat prove no exhaustion/result/record: covered by
  the combat module and 678-test library pass.
- Event choice retreat/reentry regression: 1/1.
- Unity explicit live PlayMode: 1/1 passed.

Legacy/fallback audit:

- Exhausted retreat Draw/result/node/boss branches and rumor-disproof mutation are removed.
- Attempt-consuming retreat tests were replaced by exact state and no-record assertions. No ignored
  compatibility test remains.
- Existing public result DTO fields are not used to fabricate an abandonment result; no runtime
  path sets a committed outcome to “retreated.”

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: one snapshot owner and one restore path replace branch-specific rollback. The explicit
  field list makes the transaction boundary reviewable, and root serialized equality protects the
  durable user-visible state when that boundary evolves.

Verdict:

- Complete.

Remaining risk:

- A future battle-mutable authoritative field must be added to `BattleEntrySnapshot`; the exact root
  projection regression catches user-visible omissions, while unprojected new state still requires
  normal transaction review.

## DCA-M-31 — Once-Only Consumable And Result Timing

Master item:

Consumable duration timing conflicted across result, retry, abandonment, and documents, while defeat
committed employee/Event/consumable state through separate mutations.

Expected behavior:

Every committed victory or defeat keeps the entered attempt and decrements combat duration exactly
once. Abandonment restores pre-entry direct/duration state. Employee condition, Event resolution
effects, and duration are validated in one staged result before mutation.

Runtime evidence:

- `stage_committed_combat_resolution` composes G26 employee conditions, survival XP,
  `OnAnyResolution` Event effects, and duration decrement in one `StagedCombatResultState`.
- Defeat plans fatal/retry/node/recovery transition before committing the staged state. Victory uses
  the same staged state and duration helper before reward/node completion.
- `RetreatBattle` no longer calls result or decrement helpers; snapshot restoration covers any
  battle-stage mutation.

Data evidence:

- Existing consumable duration metadata and Event reward policy are reused. No balance or RON shape
  changed.

External contract evidence:

- Core and Unity canonical documents now state one decrement per committed CombatResult, including
  a retryable defeat, and no decrement on abandonment.

Test evidence:

- The consumable regression proves repeated abandonment preserves one remaining combat use and the
  first committed defeat expires it.
- Existing victory/defeat G26 settlement and Event `OnAnyResolution` tests pass inside the 678-test
  library suite.

Legacy/fallback audit:

- The two-step defeat calls that independently committed post-battle state and duration are removed.
- Canonical “duration only when the node finally resolves” and “retreat consumes an attempt” text is
  removed. Historical goal records remain evidence only.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: one staged asset/employee state composes existing reward executors and one node-completion
  plan rather than introducing a second result schema or compensating rollback.

Verdict:

- Complete.

Remaining risk:

- None inside G15.

## DCA-L-04 — Incapacitation Terminology

Master item:

Earlier wording generalized tactical incapacitation into persistent employee death.

Expected behavior:

A dead RuntimeUnit instance receives the authored incapacitation bundle, but persistent
`EmployeeLifeState::Dead` occurs only when that bundle and death-prevention rules produce it. A
surviving employee can redeploy from the updated attempt-local condition.

Runtime evidence:

- G26's `apply_incapacitation` and G15's commit/discard transaction remain the sole life-state path;
  G15 adds no unconditional Dead assignment.
- Recovery failure distinguishes no living from alive-but-not-deployable rosters.

Data evidence:

- Existing post-battle policy and death-prevention effects remain authoritative.

External contract evidence:

- `game_rulebook.md` and `core_runtime_contract.md` now explicitly separate tactical RuntimeUnit
  death from persistent employee death.

Test evidence:

- G26 repeated death/redeploy, persistent-death lockout, and trauma-aware redeploy tests remain green
  in the 678-test library run.

Legacy/fallback audit:

- Canonical documents contain no rule that every tactical incapacitation sets the employee Dead.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: terminology now matches the already-centralized life-state runtime instead of adding a
  second “death” interpretation.

Verdict:

- Complete.

Remaining risk:

- None.

## CPR-H-02 — Fatal Committed Defeat Priority

Master item:

A defeat with attempts remaining returned retry NodeConfirm before evaluating an all-dead committed
roster, exposing retry/cancel despite terminal failure.

Expected behavior:

After the result state is staged, `NoLivingEmployees` and boss defeat preempt retry and cancel.
Nonfatal viable rosters alone may retry while attempts remain.

Runtime evidence:

- `handle_complete_combat_result` computes failure from the staged post-result roster before the
  attempt branch.
- Final boss defeat and no-living failure commit result state once and call `fail_run_with_outcome`
  directly. The retry branch requires `failure_reason.is_none()`.
- `fail_run_with_outcome` clears node/content/battle and transitions to typed RunFailed.

Data evidence:

- No policy/balance data change is required.

External contract evidence:

- No DTO shape changed; existing typed RunFailed reason and action-capability-empty terminal state
  are used.

Test evidence:

- A committed all-dead defeat with two attempts still remaining returns
  `RunFailed(NoLivingEmployees)`: 1/1 passed.
- Boss and no-deployable matrices remain green in the full library suite.

Legacy/fallback audit:

- There is no attempt-first fatal defeat branch, checkpoint exception for no living, or cancel path
  after terminal commit.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: one priority order is derived from the same staged roster that is committed, eliminating
  the former source-of-truth timing split.

Verdict:

- Complete.

Remaining risk:

- G14 owns the separately accepted explicit checkpoint action from BossOmen RunFailed; it must not
  reintroduce retry/cancel before RunFailed.

## CPR-H-09 — Full Authoritative Pre-Entry Restore

Master item:

G26 discarded attempt-local employee conditions on abandonment, but the complete world rollback
still had to cover attempts, RNG/UUID, consumables, inventory/resources, node/session/VN, and every
other battle-stage mutation.

Expected behavior:

The complete authoritative projection after abandonment equals pre-entry state, not merely the
roster. Reentry follows the restored deterministic continuation.

Runtime evidence:

- `BattleEntrySnapshot` surrounds G26's employee transaction and owns all battle-mutable world
  fields listed in the accepted contract.
- Snapshot capture occurs before map combat entry mutation; active battle owns the only copy; restore
  consumes it and clears active battle.
- Event combat preserves the already-committed choice boundary while giving the battle attempt the
  same restore/reentry semantics.

Data evidence:

- No live-data authority participates in rollback; generated preview/battle data is reproduced from
  restored run/UUID/RNG state.

External contract evidence:

- Unity receives the existing NodePreview/root snapshot shapes. Canonical Unity docs and the live
  PlayMode assertion now consume the restored attempt fields rather than carrying local counters.

Test evidence:

- Exact full root JSON equality before entry and after repeated abandonment, plus identical reentry
  battle UUID: 1/1.
- G26 abandonment roster projection and repeated-death discard regressions pass in the broad suite.
- Explicit Unity live retreat: 1/1.

Legacy/fallback audit:

- No partial roster-only rollback, compensating attempt decrement, cached rumor mutation, record
  write, or exhausted abandonment fallback remains.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: world rollback composes around the G26 working roster instead of duplicating employee
  mutation or trying to reverse individual side effects.

Verdict:

- Complete.

Remaining risk:

- Same field-maintenance risk noted for DCA-M-20; no current omission was found.

## Cross-Component Findings

- G26 employee attempt state and G15 world snapshot are nested rather than competing: G26 owns
  in-attempt condition, and G15 owns commit versus whole-stage discard.
- Event-started combat required a distinct transaction boundary: committed choice/immediate effects
  survive, while attempt/UUID/battle mutations restore. The broad Event regression now pins it.
- Recovery restriction uses the already-public map/action surfaces, so server transport and Unity do
  not gain a parallel recovery schema.
- Unity's explicit live harness initially trusted a stale root `node_confirm` during battle setup.
  It now uses `HasActiveBattleRuntime`, matching the Phase 1 battle transport authority, and the
  retreat assertion verifies the restored root state after battle teardown.
- No live RON, save schema, server result mapper, or Unity runtime inference was added.

## Validation Results

- Focused G15 contract tests: 8/8 named behaviors passed
  (exact restore, fatal priority, five recovery cases, Event reentry).
- Core library: 678 passed, 0 failed/ignored.
- Core all-targets: 719 passed across nonzero targets (`678 + 3 + 3 + 18 + 13 + 4`), 0 failed/ignored.
- `cargo check` for game_server: passed.
- game_server all-targets: 28 passed, 0 failed/ignored.
- Unity script validation for `ArkClientLiveFlowTests.cs`: 0 diagnostics.
- Unity MCP EditMode job `0f3a751a9b474723a9a4259c827098b8`: 189 passed, 0 failed/skipped.
- Unity MCP explicit live PlayMode job `4ae11b1e0f9d475ba4434daf4a5f4167`: 1 passed, 0 failed/skipped.
- Unity post-run error console after clearing the TestRunner result-save label: 0 entries.
- Core formatting, Core diff check, scoped Unity diff check, canonical legacy searches, and relevant
  ignored-test searches passed. A repository-wide Unity diff check still reports pre-existing CRLF
  whitespace in the unrelated dirty `.gitignore`; the G15-scoped Unity paths are clean.

Failed experiments retained in `EXPERIMENTS.md`:

- The initial exact-restore and fatal-priority tests failed on the intended regressions.
- The first broad Core run exposed missing Event reentry snapshot semantics and was corrected.
- Three live PlayMode attempts exposed stale-root harness assumptions before the final pass.

## Immediate Correction Candidates

None.

## Follow-Up Refactor Candidates

None inside G15. G14 is a dependent policy implementation for BossOmen Gate/RunFailed checkpoint
recovery, not a G15 correction.

## Policy Questions

None.

## Review Conclusion

All six G15 items are Complete with High long-term fit and no immediate correction. G14 may begin on
the accepted BossOmen transition/recovery contract while preserving G15's commit, fatal-priority,
and exact-abandonment invariants.
