# Cross-Layer Phase 02 Combat Completion Review

Review date: 2026-07-14

Review procedure: `docs/goal_completion_review_guide.md`

Primary verdict: Complete

Long-term fit: High

Improvement class: none

## Item Audit Table

| Item | Source of truth | Verdict | Long-term fit | Immediate correction |
| --- | --- | --- | --- | --- |
| G26 tactical death/redeploy and employee aggregation | attempt-local employee state plus employee result projection | Complete | High | None |
| G15 committed result and whole-stage abandonment | staged settlement plus exact entry snapshot | Complete | High | None |
| G14 BossOmen Gate and qualified checkpoint recovery | Core node progression and common action capability | Complete | High | None |
| G08 Unity recovery and roster flow | Core capability/eligibility plus typed command transport | Complete | High | None |

## G26 — Tactical Death, Redeployment, And Employee Outcomes

Expected behavior:

Each dead runtime instance affects the matching attempt-local employee exactly once. A fresh
deployment uses that updated employee through the ordinary stat/HP pipeline, while XP, result rows,
outcome changes, and MVP remain one employee-level result across runtime instances.

Runtime and data evidence:

- `src/game/world/state.rs` owns the working roster and processed-instance ledger in
  `BattleEmployeeAttemptState`.
- `src/game/world/combat.rs` composes the existing authored trauma, Run HP, injury, trust, and
  consumption pipeline before deployment reconciliation. Death redeployment has no fixed HP
  override; persistent death prevents a new draft.
- `src/game/battle/result_stats.rs` preserves raw instance events/participants while aggregating one
  chronological terminal row per employee.
- No live RON, loader, balance value, or save shape changed; existing post-battle policy data remains
  the single authored penalty source.

External and test evidence:

- The revisioned battle update projects the same working employee state used by deployment.
- Combat 55/55 and result-stats 4/4 cover idempotence, repeated deployments, terminal identity,
  survival XP, defeat/victory settlement, and MVP.

Legacy/fallback audit:

- No production/canonical fixed-60% death override, duplicate participant penalty loop, UUID-order
  terminal overwrite, DTO alias, or relevant ignored test remains.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: runtime-instance facts and employee transaction state have separate explicit owners and
  meet only through one projection/commit seam.

Verdict: Complete.

Remaining risk: none inside Phase 2.

## G15 — Atomic Result And Exact Abandonment

Expected behavior:

Victory/defeat commits employee condition, consumable duration, rewards, node outcome, and failure
priority atomically. `RetreatBattle` abandons the whole stage and restores the exact authoritative
pre-entry state without consuming attempts or completing the node.

Runtime and data evidence:

- `StagedCombatResultState` and planned node completion validate all fallible work before one commit.
- `BattleEntrySnapshot` captures/restores the complete gameplay state, including run/node/session,
  resources, roster/order, inventory/fragments, UUID/RNG continuation, VN/triggers, and checkpoint.
- NoLiving and boss failure preempt retry; nonboss NoDeployable deferral uses one typed immediate
  recruitment restriction and rechecks after recovery.
- No live data/schema/save migration was introduced.

External and test evidence:

- game_server preserves the root-snapshot versus battle-side-message boundary after transitions.
- Combat 55/55 includes exact snapshot/RNG restoration, repeated/final-boss abandonment, atomic
  failure retry, once-only settlement, fatal priority, and typed recovery negative controls.

Legacy/fallback audit:

- No partial employee-only rollback, exhausted-abandonment node completion, checkpoint bypass of a
  nonboss recovery restriction, or old retry-before-fatal branch remains.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: one transaction plan and one complete entry snapshot replace ad hoc inverse mutations.

Verdict: Complete.

Remaining risk: none inside Phase 2.

## G14 — BossOmen Gate And Failure Recovery

Expected behavior:

A committed forced-BossOmen victory opens one same-Floor dedicated Gate; only confirming that Gate
advances the Floor and applies Gate recovery. A committed forced defeat enters RunFailed and exposes
the existing checkpoint load only when the current forced node and load capacity qualify.

Runtime and data evidence:

- `complete_forced_boss_with_exit_gate` completes the forced node, creates a deterministic
  `boss_omen_exit_gate`, makes it the sole selectable successor, and clears the chain without
  advancing the Floor.
- `handle_confirmed_gate_transition` is the sole recovery/Floor transition.
- `is_forced_boss_omen_run_failed`, common capability evaluation, and checkpoint execution share the
  qualified current-state predicate and still reject active nonboss recovery restrictions.
- Existing BossOmen/checkpoint RON and wire shapes are reused.

External and test evidence:

- Combat tests prove forced victory Gate timing, forced defeat/load, exhausted capacity, and
  ordinary final-boss negative controls. Map tests prove Gate preview is nonmutating and confirmation
  owns advance/recovery.

Legacy/fallback audit:

- No forced-victory direct Floor advance, pre-result retry/cancel, ordinary RunFailed checkpoint
  exception, or new recovery command/alias remains.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: BossOmen composes existing result, Gate, checkpoint, and capability owners instead of
  introducing a parallel transition protocol.

Verdict: Complete.

Remaining risk: none inside Phase 2.

## G08 — Unity Recovery And Roster Consumption

Expected behavior:

Unity exposes setup reconstruction, checkpoint load, and roster movement only from current Core
capabilities and typed identities. Missing setup keeps recovery/query reachable without exposing
gameplay or falling back to root authority.

Runtime and external evidence:

- Core projects/revalidates `MoveRosterUnit` only in ViewingMap/NodeConfirm and returns authoritative
  `roster_order` slots.
- game_server forwards exact command payloads; nonbattle success produces a root snapshot while
  battle setup recovery produces setup/update side messages without mutation authority.
- `GameSnapshotStore.TryGetActionCapability` uses revisioned battle capabilities and narrows an
  incomplete runtime to query/recovery scopes. Composers, handlers, and `GameCommandBus` consume the
  exact action names and snake-case payload fields.

Test evidence:

- Core all-targets 722; server all-targets 28; Unity EditMode 193/193; ordinary PlayMode 51 passed
  with 2 explicit environment skips; explicit current Rust-server combat flow 1/1 passed.
- The live flow exercised roster movement, setup-loss recovery, battle controls, and abandonment.

Legacy/fallback audit:

- No route/reason/count inference, root battle gameplay fallback, local roster reorder, command
  alias, or third presentation authority remains.

Long-term direction review:

- Fit: High.
- Improvement class: none.
- Reason: Unity is a typed consumer of the same Core capability and state owners used at execution.

Verdict: Complete.

Remaining risk: none inside Phase 2.

## Cross-Component Findings

- G26 working condition is committed/discarded only through G15; neither layer re-applies the other.
- G14 consumes G15's committed result before choosing Gate or RunFailed recovery, so checkpoint load
  cannot become an abandonment/retry branch.
- Phase 1 UUID/revision/capability authority surrounds every battle mutation and recovery response.
- G08 reads the same root/battle capabilities and roster identities that Core revalidates; no local
  rule source competes with Core.
- Canonical Core and Unity documents describe the current transaction and transport boundaries.

## Validation Results

- Core focused: combat 55/55, map flow 28/28, result stats 4/4.
- Core all-targets: 722 passed (`681 + 3 + 3 + 18 + 13 + 4`), 0 failed/ignored.
- game_server all-targets: 28 passed, 0 failed/ignored.
- Unity EditMode `86c5de1e5f074a8891302b1e06f7713b`: 193/193.
- Unity ordinary PlayMode `05a67e0ec5094167912bd53d645ac170`: 51 passed, 2 explicit skips.
- Unity explicit live-server `7acdff34607d40478499cd836ae99119`: 1/1.
- Unity post-clear error console: 0.
- Core/server formatting and diff checks passed; scoped Unity Phase diff check passed.

## Immediate Correction Candidates

None.

## Follow-Up Refactor Candidates

None required for the Phase 2 contract. Unrelated dirty Unity CRLF/IDE files remain user-owned.

## Policy Questions

None.

## Review Conclusion

R02 independently verifies Phase 2 as Complete / High with no immediate correction. Phase 2 may
close and Phase 3 G02 may begin against the fixed transaction, recovery, and authority boundary.
