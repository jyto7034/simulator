# GrantExperience Per-Recipient Policy Implementation — Experiments

## 2026-08-13 — Goal Initialization

### Hypothesis

The approved XP rule is one centralized correctness correction and can be executed as a single Goal without a
Master, schema migration or new policy decision.

### Performed work

- Re-read `goal_authoring_and_execution_guide.md` completely before Goal creation.
- Confirmed the target Goal path did not exist.
- Confirmed the previous persistent Goal was complete and no unfinished Goal blocks activation.
- Recorded repository root, start commit and the broad dirty worktree.
- Created the three required Goal documents before new production investigation or mutation.

### Exact initialization checks

```text
read core_enhanced/docs/goal_authoring_and_execution_guide.md to EOF
test ! -e core_enhanced/docs/goals/grant_experience_per_recipient_policy_implementation
git rev-parse --show-toplevel
git rev-parse HEAD
git status --short
get current persistent Goal status
```

### Result

- Goal type: single.
- Required documents: 3/3 created.
- Path collision: none.
- Repository/start commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Previous persistent Goal: complete.
- Production changes during initialization: 0.
- Tests during initialization: 0; pass/fail/ignored 0/0/0.
- Worktree: broadly dirty, including all anticipated target files; hunk-level ownership is a required pre-mutation
  gate.

### Interpretation

The Goal may become active after document validation. Activation permits only the approved per-recipient XP
implementation and verification. Production mutation still waits for target hunk inventory and current baseline.

## 2026-08-13 — Goal Activation

### Performed work

- Validated all three required files, all 20 required PLAN sections, trailing whitespace and scoped diff.
- Registered the PLAN execution objective as the current persistent Goal.

### Result

- Persistent Goal status: `active`.
- PLAN status: `active`; `Implementation Complete: false`.
- Tests/probes in activation unit: 0; pass/fail/ignored 0/0/0.
- Production mutation in activation unit: 0.

### Interpretation

The Goal is active. The next execution step is target hunk ownership inventory and a fresh focused baseline before
any production code/data/test/canonical mutation.

## 2026-08-13 — Pre-Goal Policy And Characterization Input

These checks were performed during the user-approved implementation review immediately before Goal creation and are
recorded as activation input, not as the Goal's required pre-mutation baseline replacement.

```text
cargo test -p game_core reward_session_grant_experience_uses_explicit_alive_roster_target_and_reports_diffs -- --quiet
  1 passed; 0 failed; 0 ignored; proves current legacy [3,2,2]

cargo test -p game_core grant_preview_matches_apply_without_mutating_any_of_the_five_live_roots -- --quiet
  1 passed; 0 failed; 0 ignored; proves current legacy total diff sum 7 and preview/apply equality

cargo test -p game_core live_experience_rewards_declare_target_policy -- --quiet
  1 passed; 0 failed; 0 ignored; live amount 200 / AliveRoster

cargo test -p game_core combat_result_completion_failure_does_not_partially_apply_rewards_on_retry -- --quiet
  1 passed; 0 failed; 0 ignored; reward rollback characterization
```

The current central function, live description, target usages and affected tests were read-only reviewed. No
production code/data/test mutation occurred in that review.

## 2026-08-13 — Pre-Mutation Hunk Inventory And Fresh Baseline

### Target hunk ownership

- `src/game/reward.rs` is already modified by the user's skill-fragment-research removal. The XP resolver and its
  quotient/remainder block are unchanged from `HEAD`, so the Goal owns only the narrow arithmetic replacement there.
- `src/game/world/tests/node_sessions.rs` already contains broad user work, including the legacy AliveRoster and
  preview assertions. The Goal owns only the XP expectation changes and the adjacent target-policy matrix it adds;
  all other hunks remain untouched.
- `src/game/world/tests/combat.rs` contains broad unrelated user work. It is baseline/verification evidence only and
  is not an intended Goal edit target.
- `tests/ron_loading.rs` contains broad unrelated user work. The existing `live_experience_rewards_declare_target_policy`
  test is a merge-safe narrow seam; the Goal may extend only that test with the approved description contract.
- `game_resources/data/events/rewards/base.ron` contains unrelated reward catalog removals/pool changes. The
  `experience_reward` entry at lines 21-28 is unchanged from `HEAD`, so the Goal owns only its description line.
- `docs/core_runtime_contract.md` and `docs/game_rulebook.md` are broadly modified. The rulebook's approved policy is
  already final. The Goal owns only removal of the runtime canonical's explicit legacy-mismatch sentence after the
  implementation verifies.

### Fresh legacy baseline

```text
cargo test -p game_core reward_session_grant_experience_uses_explicit_alive_roster_target_and_reports_diffs -- --quiet
  1 passed; 0 failed; 0 ignored; legacy diff amounts [3, 2, 2]

cargo test -p game_core grant_preview_matches_apply_without_mutating_any_of_the_five_live_roots -- --quiet
  1 passed; 0 failed; 0 ignored; legacy preview diff sum 7 and exact preview/apply equality

cargo test -p game_core combat_result_completion_failure_does_not_partially_apply_rewards_on_retry -- --quiet
  1 passed; 0 failed; 0 ignored; two failed completion attempts preserve all serialized grant roots

cargo test -p game_core live_experience_rewards_declare_target_policy -- --quiet
  1 passed; 0 failed; 0 ignored; live amount 200 / AliveRoster loads through the RON integration runner
```

### Result

- Fresh focused baseline: 4 passed; 0 failed; 0 ignored across four independently filtered commands.
- The first two tests prove the current implementation still has the legacy total-pool behavior.
- No production/test/data/canonical mutation occurred before this inventory and baseline record.
- Exact Goal seams are merge-safe; no stop condition is met.

## 2026-08-13 — Target-First RED And Central Correction

### Performed work

- Replaced the two legacy total-pool expectations with per-recipient exact values.
- Added target-first coverage for all four target policies, Normal/Critical/Breakdown/Dead membership, participant
  order/dedupe/nonmember filtering, explicit-context failures, empty final targets and amount 250 multi-level growth.
- Extended the existing live RON test to require full-payment description wording.
- Ran the target filters before production mutation and observed only the intended policy failures.
- Removed quotient/remainder arithmetic from `GrantExecutor::grant_experience` and passed the same authored amount to
  every resolved employee.
- Updated only the live description and the runtime canonical's temporary legacy-mismatch sentence.

### RED evidence

```text
cargo test -p game_core grant_experience_ -- --quiet
  2 passed; 4 failed; 0 ignored
  failed: AliveRoster, CombatParticipants, AllRoster and legacy reward-session full-amount expectations
  passed: SelectedEmployee single-target multi-level and zero/context/empty semantics

cargo test -p game_core live_experience_rewards_declare_target_policy -- --quiet
  0 passed; 1 failed; 0 ignored; old description did not contain full-payment wording
```

### GREEN focused evidence

```text
cargo test -p game_core grant_experience_ -- --quiet
  7 passed; 0 failed; 0 ignored

grant_preview_matches_apply_without_mutating_any_of_the_five_live_roots
combat_result_completion_failure_does_not_partially_apply_rewards_on_retry
combat_result_commits_attempt_conditions_and_survival_xp_once_per_employee
mastered_elite_node_projects_direct_and_sweep_from_one_core_authority
sweep_stages_stable_battle_independent_result_then_commits_every_effect_once
live_experience_rewards_declare_target_policy
  each command: 1 passed; 0 failed; 0 ignored
```

### Interpretation

The former multi-recipient distribution failed before the central correction and all target/data contracts pass after
it. Existing single-target, failure, empty-target, growth, atomicity, survival and sweep semantics remain green.

## 2026-08-13 — Broad Validation And Completion Audit

The first audit pass found that PLAN C5 named saturation test evidence while the tree only had unchanged-source
evidence. A SelectedEmployee max-level/remainder regression was added, then the focused filter and both broad suites
were rerun before the final verdict. This was an evidence correction, not a production-policy change.

```text
cargo test -p game_core -- --quiet
  1080 passed; 0 failed; 0 ignored
cargo test -p game_server -- --quiet
  72 passed; 0 failed; 0 ignored
cargo fmt --all -- --check
  exit 0
git diff --check
  exit 0
scoped legacy/compatibility search
  0 legacy matches
```

- Broad total: 1152 passed; 0 failed; 0 ignored.
- Completion audit: `Complete / High`; material findings 0.
- Unity and amount-200 balance/playtest remain documented non-required omissions.
