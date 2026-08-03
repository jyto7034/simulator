# Battle Capability Execution Authority Correction Experiments

Status: complete

Final review: Complete / High / no immediate correction.

## 2026-07-14 — Goal initialization and dirty baseline

Hypothesis:

R01-H-01 and R01-H-02 can be corrected inside one bounded G07 authority seam without new policy or
public schema changes.

Action:

- Created the mandatory G30 documents after the continuation prompt explicitly authorized it.
- Created the new Master `STATUS.md` and bounded Phase 1 documents first.
- Captured Core, game_server, and Unity branch/HEAD/status fingerprints before G30 code search.

Observed result:

- All three repository views are heavily dirty.
- Known G30 target areas are already modified by completed G01/G06/G07 work, while Unity also has
  extensive unrelated UI changes.
- A scoped diff ownership audit is required before editing.

Commands and actual test counts:

- `git rev-parse`, `git status --short`, and scoped whole-tree fingerprint commands: completed.
- Executable tests: 0. No implementation success is claimed.

Decision and next step:

Read these three G30 documents, classify target-file overlap, then trace the current authority path
and establish focused failing tests before implementation.

## 2026-07-14 — Scoped overlap and authority-flow audit

Hypothesis:

The G30 target hunks can be isolated from the dirty worktree and corrected without overwriting
unrelated changes or changing a public contract.

Action:

- Inspected current Core projection/gate/domain dispatch and relevant scoped diffs.
- Inspected game_server authority mapping, emission ordering, and stale recovery test seam.
- Inspected Unity error handling, authoritative resync application, and adjacent EditMode tests.
- Ran an independent read-only Warp Grep audit against the same current worktree.

Observed result:

- `world.rs` is a mixed historical diff, but G30 only needs the existing gate near
  `gate_behavior_action`; the surrounding authority-first order is completed G01/G06/G07 work.
- Untracked `battle_capability.rs` is the completed G07 projector and is the correct G30 Core owner.
- No production battle handler change is required; the existing blocking-VN test is the focused
  extension seam.
- server `handlers.rs` is mixed, but the stale-only match/helper region and adjacent stale test are
  completed G07 work and can receive the bounded correction without touching RunStart/VN changes.
- Unity production already applies proactive `battle_resync`; no production change is required.
  The large mixed test file can receive one adjacent missing-error/resync consumption test.
- No DTO, RON, save, gameplay, UX, or error-code policy blocker was found.

Commands and actual test counts:

- Read-only `rg`, `sed`, and scoped `git diff` inspections: completed.
- Warp Grep read-only audit: completed.
- Executable tests: 0. No implementation success is claimed.

Decision and next step:

The scoped overlap is classifiable and safe to continue with `apply_patch`. Add a focused failing
Core direct-command test before implementing the shared evaluator.

## 2026-07-14 — Initial focused filter matched zero tests

Hypothesis:

The new blocking-VN direct-command regression can be run by its short name with `--exact`.

Action:

- Added the focused Core test covering deploy, deployment preview, withdraw, skill, pause, resume,
  speed, and `RetreatBattle` rejection plus unchanged serialized battle projection.
- Ran `cargo test -q -p game_core blocking_visual_novel_execution_preflight_rejects_every_omitted_battle_action_without_mutation --lib -- --exact --nocapture`.

Observed result:

- 0 tests executed; 670 filtered out.
- Rust's exact name includes the module path, so this command is invalid as evidence.

Failure cause:

The short test name was combined with `--exact` instead of using the fully qualified test path.

Correction and revalidation:

Rerun the same short filter without `--exact` and require a nonzero failing test before production
implementation.

## 2026-07-14 — Core direct-command regression reproduces MSTR-05

Hypothesis:

Without a shared battle-context preflight, a current-authority command omitted by a blocking VN can
reach its handler and mutate the battle.

Action:

- Reran the new regression without `--exact`.

Commands and actual test counts:

- `cargo test -q -p game_core blocking_visual_novel_execution_preflight_rejects_every_omitted_battle_action_without_mutation --lib -- --nocapture`
- 1 executed, 0 passed, 1 failed, 669 filtered out.

Observed result:

- The first hidden `DeployUnit` succeeded and committed battle revision 3 instead of returning the
  expected context rejection.
- The failure directly proves that the current projection is not the Core execution gate.

Failure cause:

`gate_behavior_action` composes only the common preflight; battle actions fall through to handlers.

Correction and revalidation:

Implement one canonical battle-context evaluation list in `battle_capability.rs`, reuse it for DTO
projection and a new execution preflight, then rerun the same nonzero test.

## 2026-07-14 — First shared evaluator preserved actions but weakened playback reason

Hypothesis:

Every action omitted from context projection can return `context_irrelevant` during execution
preflight.

Action:

- Implemented one shared Core battle-context evaluation list and composed it after authority and
  common preflight.
- Reran the new direct-command test, blocking-VN group, playback, and stale-authority tests.

Commands and actual test counts:

- New direct-command test: 1 passed.
- `cargo test -q -p game_core blocking_visual_novel --lib -- --nocapture`: 3 passed.
- stale-authority focused test: 1 passed.
- playback focused test: 1 executed, 1 failed.

Observed result:

- The authority gap was closed for blocking VN.
- Repeated Pause after pausing returned `context_irrelevant`, while the existing playback payload
  eligibility and handler contract require `already_active`.

Failure cause:

The first evaluator represented every omitted action with one reason. Context omission and
payload-state mutual exclusion need the same evaluator to preserve distinct typed reasons even
though both are omitted from context projection.

Correction and revalidation:

Carry a reason on the evaluator's irrelevant state. Projection still omits it; execution returns
`already_active` for the opposite Pause/Resume command and `context_irrelevant` for blocking-VN
suppression. Rerun all four focused groups.

## 2026-07-14 — Context-disabled preflight initially masked concrete payload reasons

Hypothesis:

A context capability disabled with `no_eligible_payload` should reject before every concrete domain
payload preflight.

Action:

- Ran existing deploy cooldown, missing skill target, and insufficient stabilization tests after
  composing the first shared battle evaluator.

Commands and actual test counts:

- deploy/withdraw lifecycle: 1 executed, 1 failed.
- missing manual-skill target: 1 executed, 1 failed.
- insufficient stabilization: 1 executed, 1 failed.

Observed result:

- Each handler previously returned its projected concrete payload reason: `redeploy_cooldown`,
  `no_valid_target`, or `insufficient_resources`.
- The first context preflight replaced all three with coarse `no_eligible_payload`.

Failure cause:

PD-02 is explicitly two-level. A disabled context family means no payload is currently enabled, but
the submitted concrete payload still belongs to its domain preflight, which owns its precise reason.
Only an irrelevant context such as blocking-VN suppression must stop before payload handling.

Correction and revalidation:

Keep disabled context projection unchanged, but allow parameterized disabled families to reach the
existing pure payload preflight. Retain hard rejection for irrelevant context and nonparameterized
mutual-exclusion reasons. Rerun all affected focused tests.

## 2026-07-14 — Focused Core/server/Unity correction passes; rustfmt check requires cleanup

Observed result:

- Core direct-command: 1/1 passed.
- Core blocking-VN group: 3/3 passed.
- Core playback, stale authority, deploy cooldown, missing target, and insufficient resources:
  5/5 individually passed.
- Server missing and stale authority recovery: 2/2 individually passed.
- Unity MCP focused missing-error/proactive-resync consumption: 1/1 passed.
- `cargo fmt --all -- --check` failed only on rustfmt line wrapping in the new Core evaluator and
  server authority match.

Failure cause:

The functional edits had not yet been passed through rustfmt.

Correction and revalidation:

Run workspace rustfmt, rerun formatting check, then rerun the smallest affected focused groups.

## 2026-07-14 — Server regression reproduces MSTR-06

Hypothesis:

Missing battle authority falls through the ordinary actor error path and therefore cannot produce
the stale-equivalent ordered recovery batch.

Action:

- Added a paired server test that sends Pause without battle authority after entering a live battle.

Commands and actual test counts:

- `cargo test -q -p game_server missing_battle_authority_emits_error_then_current_resync_without_mutation --lib -- --nocapture`
- 1 executed, 0 passed, 1 failed, 27 filtered out.

Observed result:

- The actor returned `PlayerGameActorError { code: "missing_battle_state_authority" }` directly.
- No transportable `CommandExecutionResult` and no current `battle_resync` were produced.

Failure cause:

The handler special-cases only `StaleBattleStateAuthority`; missing authority uses the ordinary error
branch.

Correction and revalidation:

Extract one actor recovery-result builder and route both typed authority errors through it while
preserving their distinct mappings. Rerun missing and stale focused tests.

## Experiment Template

### YYYY-MM-DD — Title

Hypothesis:

Action:

Changed files:

Commands and actual test counts:

Observed result:

Failure cause:

Correction and revalidation:

Decision and next step:
