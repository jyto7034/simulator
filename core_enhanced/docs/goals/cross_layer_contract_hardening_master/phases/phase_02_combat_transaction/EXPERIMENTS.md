# Phase 02 Combat Transaction And Boss Recovery Experiments

## 2026-07-14 — Phase boundary initialized

Hypothesis:

Phase 1's Complete / High review unlocks G26 and provides stable revision/capability authority for
the Phase 2 combat transaction.

Action:

- Created the isolated Phase 2 PLAN, EXPERIMENTS, and EXPERIMENT_NOTES documents.
- Registered G26 as the only active Child and preserved `G26 -> G15 -> G14 -> G08 -> R02` order.
- Copied no Child implementation experiments into this Phase log.

Changed files:

- Phase 2 goal documents and Master status only.

Commands and actual test counts:

- Document and repository-state inspection only; 0 tests executed and no implementation success
  claimed.

Observed result:

- R01 is Complete / High and no dependency blocks G26.
- PD-16 and the employee-level aggregation contract are accepted; no new policy decision is needed.

Decision and next step:

Run G26's focused baseline, implement and review G26, then unlock G15 only after a Complete review.

## 2026-07-14 — G26 passes completion review

- G26 review verdict: Complete.
- Long-term fit: High.
- Improvement class: none.
- Core all-targets: 712 passed.
- game_server all-targets: 28 passed.
- Unity MCP EditMode: 189 passed.
- Fixed invariant handed to G15: employee attempt working state is the sole death/redeploy condition
  authority; committed result consumes it once and abandonment discards it.

Decision:

Unlock G15. G15 must extend the employee discard seam into exact complete pre-entry restoration and
must not recreate employee penalty logic.

## 2026-07-14 — G15 passes completion review

- G15 review verdict: Complete.
- Long-term fit: High.
- Improvement class: none.
- Core all-targets: 719 passed.
- game_server all-targets: 28 passed.
- Unity MCP EditMode: 189 passed.
- Unity explicit live PlayMode retreat contract: 1 passed against local game_server.
- Fixed invariant handed to G14: BossOmen defeat first commits the G15 result and enters RunFailed;
  any checkpoint recovery is an explicit action from that terminal context. BossOmen victory must
  use the committed result and open a dedicated Gate without bypassing Floor transition policy.

Decision:

Unlock G14 `boss_omen_gate_and_rollback_policy`.

## 2026-07-14 — G14 passes completion review

- G14 review verdict: Complete.
- Long-term fit: High.
- Improvement class: none.
- Core all-targets: 720 passed.
- game_server all-targets: 28 passed.
- Unity MCP EditMode: 189 passed.
- Fixed invariant handed to G08: Core alone identifies forced BossOmen recovery, projects the
  dedicated Gate/checkpoint capability, and executes the existing commands. Unity must render and
  dispatch from those projections without local boss/load rules.

Decision:

Unlock G08 `unity_recovery_and_roster_command_flows`.

## 2026-07-14 — G08 passes completion review

- G08 review verdict: Complete.
- Long-term fit: High.
- Improvement class: none.
- Core all-targets: 722 passed.
- game_server check and all-targets: 28 passed.
- Unity MCP EditMode: 193 passed.
- Unity ordinary PlayMode: 51 passed, 2 explicit skips.
- Unity explicit live Rust-server flow: 1 passed with actual roster move and setup recovery.
- Fixed invariant handed to R02: Unity consumes root/battle capability without local recovery or
  roster authority; incomplete battle runtime retains query/recovery scopes only.

Decision:

Activate R02 `cross_layer_phase_02_combat_completion_review` for the independent combined Phase
matrix and legacy/fallback audit.

## 2026-07-14 — R02 passes and Phase 2 closes

- R02 current-code trace found one coherent transaction/recovery boundary across all four Children.
- Focused and broad validation passed: Core 722 all-targets, server 28, Unity EditMode 193,
  ordinary PlayMode 51/2 explicit skips, and explicit live-server 1.
- Legacy/canonical/hygiene audit found no immediate correction.
- Verdict: Complete / High. Phase 3 G02 is unlocked.
