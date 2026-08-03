# Cross-Layer Phase 02 Combat Completion Review Experiments

Review: R02

Updated: 2026-07-14

## 2026-07-14 — Review initialized

Question:

Do the four Complete / High Child reviews compose into one Phase-complete transaction and recovery
boundary on the current worktree?

Evidence available at initialization:

- G26, G15, G14, and G08 each have a `Complete / High / no immediate correction` review.
- Latest broad counts are Core 722 all-targets, game_server 28 all-targets, Unity EditMode 193,
  ordinary PlayMode 51 passed/2 explicit skips, and explicit live-server 1 passed.
- The repositories remain heavily dirty and all unrelated changes remain user-owned.

Claim status:

No Phase success is claimed yet. R02 will independently trace current code, select the combined
focused matrix, rerun relevant broad validation, and perform the Phase legacy/canonical audit.

## 2026-07-14 — Current transaction and recovery trace

Question:

Do the current runtime seams compose one owner from tactical death through result/abandonment,
BossOmen transition, transport, and Unity presentation?

Evidence:

- `ActiveBattleSession.employee_attempt` owns the battle-entry employee clone and processed runtime
  instance ledger. `reconcile_employee_attempt` applies the complete authored incapacitation bundle
  once per dead deployed instance before the revisioned projection is committed.
- Death locks carry no HP override. A fresh deployment rebuilds the employee draft from the current
  attempt roster; persistent death fails ordinary deployability. Withdraw locks retain only their
  distinct explicit HP carry rule.
- `CombatBattleState.employee_conditions` promotes the completed working roster. Staged result
  settlement replaces the persistent roster once and employee result stats aggregate runtime
  instances into one employee row before XP, reward context, outcome rows, and MVP consume them.
- `BattleEntrySnapshot` captures and restores world state, node/session/content, run state,
  VN/triggers, UUID/RNG continuation, resources, inventory, roster/order, fragments, starter flow,
  and checkpoint state. `RetreatBattle` consumes it and creates no result or node completion.
- Forced BossOmen victory calls `complete_forced_boss_with_exit_gate`, leaving the current Floor and
  one deterministic `boss_omen_exit_gate`; only confirmed ordinary Gate handling applies recovery
  and advances the Floor. Qualified forced-BossOmen `RunFailed(BossDefeated)` alone enables the
  existing checkpoint action when load capacity is valid.
- game_server maps nonbattle command payloads to a following root snapshot, maps battle mutations
  and setup recovery to ordered setup/update or resync side messages without a root battle
  snapshot, and returns current non-mutating resync for missing/stale authority.
- Unity `GameSnapshotStore` reads complete battle capabilities from the revisioned battle update.
  While setup is missing it retains query/recovery scopes only and never falls back to root
  gameplay. Composers and handlers dispatch checkpoint load, setup recovery, and roster movement
  only from those capabilities and authoritative `roster_order` identities.

Result:

The four Child boundaries compose without duplicate employee condition, partial settlement,
pre-failure BossOmen retry, root battle authority, or local Unity rule ownership.

Decision and next step:

Run the focused and broad combined matrix, then perform legacy/canonical hygiene checks.

## 2026-07-14 — Combined validation matrix

Question:

Does the current worktree protect the combined seams rather than only the individual Child paths?

Evidence:

- `cargo test game::world::tests::combat`: 55 passed, 0 failed/ignored. This includes repeated
  incapacitation/settlement, live redeployment, exact abandonment/RNG restoration, fatal priority,
  typed nonboss recovery, forced BossOmen defeat/load/Gate, revision authority, and setup recovery.
- `cargo test game::world::tests::map_flow`: 28 passed, 0 failed/ignored.
- `cargo test game::battle::result_stats::tests`: 4 passed, 0 failed/ignored, including chronological
  redeploy aggregation and incapacitated-employee MVP eligibility.
- Core `cargo test --all-targets`: 722 passed (`681 + 3 + 3 + 18 + 13 + 4`), 0 failed/ignored.
- game_server `cargo test --all-targets`: 28 passed, 0 failed/ignored.
- Unity EditMode job `86c5de1e5f074a8891302b1e06f7713b`: 193/193 passed.
- Unity ordinary PlayMode job `05a67e0ec5094167912bd53d645ac170`: 51 passed, 0 failed,
  2 explicit environment-dependent live tests skipped by design.
- A current local Rust server was started on port 8081. Explicit live job
  `7acdff34607d40478499cd836ae99119` passed 1/1 and exercised authoritative roster movement,
  battle setup recovery, mutation controls, and abandonment; the server then shut down cleanly.
- Post-run Unity error console after clearing TestRunner save output: 0 entries.

Result:

All nonzero required targets passed. No zero-test command was counted as evidence.

Decision and next step:

Audit legacy/fallback paths and formatting before the completion verdict.

## 2026-07-14 — Legacy, canonical, and hygiene audit

Question:

Does any old authority or partial transition remain despite green tests?

Evidence:

- Production and canonical searches found no fixed-60% death-redeploy override. Historical goal
  evidence retains the superseded policy, while current runtime/canonical text uses attempt-local
  employee condition and the ordinary Battle HP pipeline.
- The only forced-BossOmen completion helper creates the dedicated Gate. Floor advance remains in
  ordinary Gate confirmation; no forced-victory direct-advance branch was found.
- Result settlement is staged before commit and `RetreatBattle` restores the full entry snapshot;
  no employee-only undo or exhausted-abandonment completion branch remains.
- Unity searches found exact new action names/wire keys only, no compatibility alias, local roster
  swap/reorder mutation, static checkpoint eligibility, or root battle gameplay fallback.
- Relevant Rust focused files contain no ignored tests. Unity has exactly two explicit live tests;
  the applicable ordinary live combat flow was executed successfully in this review.
- `docs/core_runtime_contract.md`, `docs/game_rulebook.md`, and Unity's canonical contract describe
  the current employee transaction, BossOmen Gate/load, and capability-driven commands.
- Core and game_server `cargo fmt --all -- --check` and `git diff --check` passed. The scoped Unity
  Phase files passed `git diff --check`. Repository-wide Unity diff hygiene still reports unrelated
  pre-existing/user-owned CRLF additions such as `.gitignore` and Rider metadata; they were not
  modified or treated as Phase defects.

Result:

No material legacy path, compatibility layer, duplicated authority, or canonical contradiction was
found in Phase scope.

Decision and next step:

Apply the completion-review guide. R02 passes `Complete / High / no immediate correction`; close
Phase 2 and unlock Phase 3 G02.

## Experiment Template

### YYYY-MM-DD — Short title

Question:

Evidence:

Result:

Decision and next step:
