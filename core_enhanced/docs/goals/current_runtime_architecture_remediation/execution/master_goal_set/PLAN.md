# Core Runtime Architecture Remediation Master Goal Execution Plan

Status: `GEX-30 Attempt 3 complete / Complete / High / terminal current`

## Execution Identity

- Handoff: `HA-CRAR-P2G60-002` / current.
- Goal Set / member: `GS-CRAR-P2G50-002` /
  `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-MASTER)`.
- Run mode / root target: `goal-run` / full `GS-CRAR-P2G50-002` terminal result.
- Current pass: fresh read-only `GEX-30` complete; implementation mutation authorization: none.
- Goal Quality Contract: `2`.
- Repository: `/mnt/f/work/simulator/core_enhanced`, revision
  `f318414fb0156a67e04c5479000c2a2683fc0c09`, branch
  `cleanup/simulator-dirty-worktree-20260803`.

## Canonical Execution Document Set

- Directory: `docs/goals/current_runtime_architecture_remediation/execution/master_goal_set`.
- Plan: `docs/goals/current_runtime_architecture_remediation/execution/master_goal_set/PLAN.md`.
- Experiments: `docs/goals/current_runtime_architecture_remediation/execution/master_goal_set/EXPERIMENTS.md`.
- Notes: `docs/goals/current_runtime_architecture_remediation/execution/master_goal_set/EXPERIMENT_NOTES.md`.
- Completion audit: `docs/goals/current_runtime_architecture_remediation/execution/master_goal_set/COMPLETION_AUDIT.md`.
- No other canonical directory is selected for this exact Master member. The earlier root-level GEX-30 output is
  retained as a non-terminal historical attempt because it did not establish this directory and a complete Master
  Audit Target Record before its audit.

## Objective and Boundary

- Objective: independently audit the final current Core tree after all eight ordered Phase Goals, and issue the
  terminal Master verdict only if every Master completion condition is directly proven.
- Contract-visible result: one coherent Core-only runtime/data/public contract with all Freeze coverage, execution
  nodes and integration seams current; no completion-blocking unknown, hidden fallback or legacy authority.
- In scope: current Core source/data/docs/tests, P01-P08 completion/currentness records, final performance and
  integration evidence, manifest/API/static/release/full-suite validation, and scoped dirty attribution.
- Out of scope: `game_server`, Unity, sibling resources, external consumers, deployment/release rollout, product SLA,
  and deferred feature policy implementation.
- Implementation boundary: GEX-30 may write audit documents/evidence only. It may not change production source/data,
  tests, fixtures, tools, manifests or the audit target.

## Prerequisites and Current Baseline

- Direct prerequisites: P01 `_009`, P02 `_007`, P03 `_007`, P04 `_005`, P05 `_005`, P06 `_003`, P07 `_002`, and
  P08 `_001`, each with a latest `Complete / High` completion/currentness result.
- Attempt 2 found a runtime `cfg!(test …)` public-projection switch and correctly stopped. A separated P07 correction
  GEX-10/GEX-20 replaced it with compile-time capability constants, then P08 GEX-20 freshly revalidated the corrected
  tree: release/build/docs, manifest, 11 API-probe outcomes, five performance/public tools, structural review and
  1,159 Core tests passing.
- Master Audit Target Record: `ATR-CRAR-C2-MASTER-GEX30-002` /
  `evidence/P08_MASTER_AUDIT_TARGET_002.json`, SHA-256
  `8d858ace7809537a51de5d82c3999ebdc5a61f68f32a416becec818b12f34fe0`. It composes P08's exact 95-member original
  implementation submanifest, the exact 16-member correction/currentness submanifest, and 32 exact
  contract/upstream/Phase/currentness/build bindings.
- Existing unrelated Core dirty baseline: 92 modified, 23 deleted, 675 untracked, total 790, fingerprint
  `d9e815c705367387b6a610aa699e2eaa0968172c4c287cb567b822d6be7fd622`, after cumulative task-owned exclusions.
  Existing user work must remain unchanged and must not be attributed to this goal-run.

## Validation and Completion Conditions

- Revalidate the complete Master target identity before and after the audit.
- Confirm eight ordered Phase verdicts/currentness, Final Freeze selectors `10/10`, Final Execution Nodes `9/9`,
  integration seams `12/12`, policy acceptance `10/10`, validation dimensions `10/10`, feature inventory `57/57`,
  full Core test result `1,159/1,159`, expected API-probe outcomes and unresolved structural count `0`.
- Directly inspect current source classifications for single application authority, private finite kernel, validated
  data boundary, bounded history/result independence, V4-import/V5-write boundary, public projection isolation,
  test/benchmark production-branch absence and legacy/fallback removal.
- Reconfirm all required negative/edge dimensions; a broad happy-path suite alone is insufficient.
- Completion requires a complete Master Audit Target Record that remains current through audit exit, no implementation
  mutation during GEX-30, no hidden deferral and no completion-blocking unknown.

## Stop Conditions

Stop without expanding authority if policy meaning, Goal/Phase/dependency/mapping/execution boundary, external required
evidence, exact upstream basis, dirty isolation or target identity is stale, contradictory, unsafe or unknown. An
in-scope technical defect returns to the owning Phase's separated correction GEX-10/GEX-20 before another GEX-30.
