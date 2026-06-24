# Core Policy Implementation Master

## Objective

Confirmed policies from `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md` must be implemented in code, live RON/data, Unity-facing DTO contracts, and tests.

This goal is the execution master. The earlier `core_component_refactor_master` remains an audit and policy-decision input, not the implementation tracker.

## Source of Truth Order

When any document disagrees with code or data, verify in this order:

1. Actual runtime code.
2. Live RON/data loaded by the runtime.
3. Unity-facing snapshot/command DTO contracts.
4. Latest policy documents, especially `docs/refactor_preparation_plan.md` and `docs/goals/core_component_refactor_master/POLICY_DECISIONS.md`.

Do not preserve legacy behavior through compatibility layers, fallback paths, or dual schemas unless the reason and removal condition are recorded here and confirmed by the user.

## Subgoals

Run these subgoals in order. Each subgoal owns its own `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`.

1. `docs/goals/core_policy_static_data_map_scenario`
   - Live RON policy, run/map node source of truth, battlefield/scenario data, authored route rules, starter/legacy data loading boundaries.
2. `docs/goals/core_policy_grant_economy_rewards`
   - Canonical grant executor, rewards, Enkephalin, reward tags removal, equipment pools, abnormality reward validation.
3. `docs/goals/core_policy_employee_item_growth`
   - Employee growth, trust deferral, skill fragments, bound equipment, starter loadouts, trauma/final HP policy.
4. `docs/goals/core_policy_buff_status_stats`
   - Hard CC source of truth, buff cleanup, max stack invariants, movement speed stat policy.
5. `docs/goals/core_policy_movement_battle_runtime`
   - Behavior gates, movement/blocking, battle runtime events, same-timestamp attack resolution, runtime numeric/timeline policy.
6. `docs/goals/core_policy_skill_targeting_projectile`
   - Skill targeting, tile range previews, projectile impact/miss semantics, long line/piercing representation.
7. `docs/goals/core_policy_unity_server_contract`
   - Shared gameplay command DTOs, BehaviorResult split, typed run snapshot, timeline attachment ownership, BattleResync cleanup.
8. `docs/goals/core_policy_validation_test_harness`
   - Timeline/log validation, live content audit manifest, final cross-checks, broad validation suite.

## Working Rules

- Prefer long-term architecture over temporary or test-only patches.
- Update the active subgoal docs continuously while implementing.
- Record failed approaches and why in the subgoal `EXPERIMENTS.md`.
- Record policy-sensitive findings as `사용자와 정책 논의 필요` in the relevant `EXPERIMENT_NOTES.md`; do not invent a policy during implementation.
- If runtime code, live RON/data, Unity-facing DTO contracts, or tests reveal that an additional policy decision is required, complete the active goal with a policy-decision report instead of implementing that area.
- A policy decision is required when the implementation would change DTO shape, live RON schema, saved-data migration, UX meaning, balance rules, content deletion/replacement, failure/reward/consume timing, or any other user-visible gameplay contract not already decided in `POLICY_DECISIONS.md`.
- Do not continue by making an arbitrary temporary decision, compatibility fallback, or "reasonable default" for that area. Record the issue as `사용자와 정책 논의 필요`, list the concrete options and code/data evidence, then report it to the user.
- Scope follow-up ideas into notes unless they are required to complete the current policy or reduce blast radius.
- Tests should pin user-visible behavior, Unity-facing DTO shape, live RON loading, validation behavior, and gameplay flow rather than private implementation details.
- Remove tests that preserve obsolete policy instead of marking them ignored.

## Policy Decision Completion Conditions

Complete the active implementation goal with a policy-decision report when any of these occur:

- A new policy decision is required to implement the next change correctly.
- Code and confirmed policy conflict in a way that cannot be resolved by source-of-truth inspection.
- The only available implementation path would preserve legacy behavior through fallback, compatibility, or dual-schema support that was not explicitly approved.
- A user-visible contract change is needed but not already covered by the confirmed policy list.
- A live content deletion/replacement, migration decision, or balance rule must be chosen.

The report must include the implementation area requiring a policy decision, concrete options, code/data evidence, and the recommended long-term choice.

When a policy-decision report is written, close the currently active goal as `complete` through the available goal-status tool. Do not leave it active, do not mark it blocked, and do not continue into the undecided implementation area. The next implementation run should start only after the user makes the policy decision.

## Master Completion Conditions

- Every subgoal is complete and its completion report is written.
- A subgoal may be completed either by implementation/verification or by a policy-decision report when new user policy is required.
- Policy-decision completion is a valid goal outcome, not a blocked, stopped, or paused state.
- Every confirmed policy in `POLICY_DECISIONS.md` is either implemented or explicitly linked to a deferred trust-only policy.
- No normal runtime path accepts the removed legacy schemas/events/fallbacks named in the policy list.
- Live RON/data validation fails for policy-invalid content.
- Unity-facing DTO and WebSocket command contracts are documented by tests or fixtures.
- A final pass compares generated implementation docs against code and live data for drift.
- Final validation commands and remaining risks are reported.

## Final Cross-Check

After all subgoals finish, compare:

- Each generated subgoal document against touched code.
- Each confirmed policy against runtime behavior.
- Live RON schemas and loaded data against validation expectations.
- Unity-facing DTO snapshots/commands against server transport behavior.
- Tests against the new policy wording.

Any mismatch must be fixed or recorded as `사용자와 정책 논의 필요`.
