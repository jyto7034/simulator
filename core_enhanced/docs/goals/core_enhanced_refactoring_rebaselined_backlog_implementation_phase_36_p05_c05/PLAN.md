# Phase 36 CRA-P05-C05 No-Op Mission Policy And Redundant Shape Test Cleanup Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — the user's 2026-08-09 instruction to execute the Master campaign
Parent Master: `core_enhanced_refactoring_rebaselined_backlog_implementation_master`
Phase: 36 of 44
Candidate / Work Type: CRA-P05-C05 / `cleanup`
Direct Linear Dependency: Phase 35 CRA-P10-C07 `Complete / High`
Semantic Dependency: Phase 27 CRA-P05-C03 `Complete / High`
Direct Policy: CRA-POL-006
Repository / Start Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Initialization Date: 2026-08-11 (`Asia/Seoul`)

## Objective

After current caller/visibility and assertion inventories, remove only mission-policy branches/helpers and test
bodies that express no distinct production contract. Preserve the complete category/SplitRoom/live mission matrix,
Boss mapping, default Defense scenario, reward empty/featured/forbidden behavior and every public result. Do not
narrow a public Rust helper under CRA-POL-006 without the required consumer evidence.

## User/Contract-Visible Problem

Original and post-Boss P05 audits found an elite-versus-ordinary branch whose arms return the same mission policy, an
archetype wrapper that ignored its argument and was historically test-only, and four redundant test bodies whose
assertions were either shape-only or strict subsets. These add false policy branches and maintenance synchronization
without protecting unique gameplay. Cleanup is safe only after current evidence proves the same facts and every
unique assertion is present in named surviving tests.

## Audit Finding ID

CRA-P05-C05 with test-cleanup rows CRA-TC-02, CRA-TC-03 and CRA-TC-04. The rebaselined catalog classifies it
`still_valid`, `cleanup`, Tier D, known subtotal `7/18`, and `assertion transfer/caller inventory` ready. Exact linear
item 36 requires P05-C03 completion, CRA-POL-006 and survival of category/SplitRoom/mission/reward assertions.

## Current Runtime And Live Data Evidence

Historical evidence names prospective policy/test seams under `combat_setup/mission_policy.rs`,
`events/combat.rs` and `reward_policy.rs`:

- elite and ordinary Combat preference branches both returned `[Defense]`;
- `fallback_mission_variant_for_archetype` ignored archetype, returned node-type default and had only its own test;
- two mission helper tests protected no distinct branch;
- one fixed-defense scenario test was a strict subset of the stronger default black-box Defense test;
- two Defense currency-only reward failures duplicated the same rejection, with one uniquely covering empty reward
  acceptance.

Post-Boss evidence said those no-op branches, wrapper and shape-only tests remained. Current-tree proof now confirms
both helpers are public, the preference helper has one production workspace caller, and the wrapper has no
non-test workspace caller but an unknown external-consumer surface. The identical elite branch is simplified; the
wrapper is retained unchanged under CRA-POL-006; all unique assertions moved before exactly four redundant bodies
were removed. Current live pairs/data and every non-target prospective hash remain activation-exact. Historical
evidence was used only to locate candidates, never as deletion authority.

## Source Of Truth And Ownership

### Current prospective authority

- The explicit mission policy table owns supported node-type/variant behavior and intentional SplitRoom rejection.
- Combat setup/scenario builders consume that policy and own complete objective, route, unit and win-condition shape.
- `CombatRewardPolicy` owns permitted/required reward-kind validation.
- Public Rust visibility plus actual caller/consumer evidence owns compatibility disposition.

### Target authority

One direct category-based mission policy remains, with no elite branch that returns the same value and no private
argument-ignoring wrapper. Surviving category/SplitRoom/live-pair, full default-Defense and reward policy tests own
the complete durable assertions. Public helpers are retained unchanged unless current evidence satisfies
CRA-POL-006's consumer/deprecation gate; negative workspace search alone never authorizes narrowing.

## Invariants

- Supported mission node types, exact variants, intentional SplitRoom failure and Boss mapping remain exact.
- Current live Defense/Defense and Boss/Boss pairs remain valid; no new variant, balance or fallback behavior.
- The stronger default Defense test retains objective, player plan, unit reference, route, enemy movement,
  defense-object position and win-condition assertions.
- Reward validation retains empty acceptance, exact currency-only Defense failure text, featured/allowed/forbidden
  identity behavior and Boss-positive behavior.
- All public Rust signatures/visibility and external observable results remain exact unless a separately approved
  migration is obtained.
- No production data/RON, DTO/wire, save/checkpoint, server, Unity or canonical contract change.
- No Phase-37 validator cleanup or any later candidate is pulled forward.
- Existing user/campaign dirty work and the protected Phase-27 approval marker remain exact.

## Refactor Candidate And Behavior-Preservation Boundary

- Candidate ID: CRA-P05-C05.
- Work type: `cleanup`; `still_valid`; Tier D; assertion-transfer/caller-inventory ready.
- Priority axes: ownership `1`, correctness/atomicity `1`, lifecycle `1`, change amplification `1`, test fragility
  `2`, failure blast radius `1`, change frequency `unknown`; known subtotal `7/18`.
- Prospective removal scope: identical elite branch, nonpublic argument-ignoring wrapper if still caller-free, two
  no-op mission tests, one strict-subset scenario test and one of two overlapping reward failure tests after its
  unique empty/exact-error assertions are transferred.
- Retained authority: explicit mission and reward policy tables, all unique setup assertions, live pairs and public
  APIs.
- First safety net: exact current visibility/caller graph and per-test assertion matrix, followed by nonzero
  category/SplitRoom/mission/reward baseline commands before mutation.
- Behavior-preservation boundary: any changed policy output/error, lost unique assertion, public/API narrowing,
  data/serialization difference or unresolved external consumer stops cleanup.

## Approved Policy And Evidence

- Original P05 C05: remove the no-op branch/wrapper and only redundant tests after exact assertion transfer.
- Post-Boss P05 C05: still valid; preserve Boss category mapping, SplitRoom rejection and stronger mission/reward
  assertions.
- CRA-TC-02: remove two no-op mission helper tests after category/SplitRoom/live-pair coverage and caller inventory.
- CRA-TC-03: remove the strict-subset Defense scenario test only after exact assertion comparison to the stronger
  black-box case.
- CRA-TC-04: merge duplicate currency-only Defense failures while preserving empty acceptance and exact failure text.
- Phase 27 P05-C03: the current typed resolver/caller graph is complete and public Floor APIs remain intentionally
  compatible; it supplies this Phase's semantic predecessor without authorizing unrelated API removal.
- CRA-POL-006: public Rust symbols cannot be removed or narrowed from a negative workspace search alone.

## Open Policy Decisions

None at initialization. If the wrapper is currently public or has an unknown/non-workspace consumer and cleanup
would require narrowing it, retain it and stop any deletion-dependent claim unless an approved compatibility path
already exists. If current mission/reward behavior differs from the audited invariants, stop for scope/policy review
rather than rewriting policy during cleanup.

## In Scope

- current current-tree inventory of named mission/reward helpers, visibility, callers and exact test bodies;
- current mission category, SplitRoom, live-pair, default Defense and reward assertion matrix;
- baseline of all named surviving and prospective redundant tests on old/current production;
- test-first transfer of any unique assertion into the named survivor before removal;
- minimal removal/simplification proven safe by current evidence;
- caller/legacy/ignored-test searches, focused/broad debug/release validation and scoped dirty preservation;
- separate completion audit.

## Out Of Scope

- new mission variants, elite tuning, archetype-driven mission choice, data-driven policy or balance/content edits;
- changing explicit SplitRoom rejection, Boss mapping, Defense/Boss live pairs or fallback meaning;
- public Rust/API removal, deprecation or signature/visibility narrowing without CRA-POL-006 evidence;
- broader combat preview/setup resolver refactors already owned/completed by P05-C03/C04;
- live RON, DTO/wire, save/checkpoint, server, Unity or gameplay canonical migration;
- Phase 37 disconnected event-log validator cleanup or later Stage-D work;
- broad formatting or unrelated lint/test cleanup.

## Implementation Scope

After sole activation and current-tree proof:

1. fingerprint every prospective source/test/canonical input before content investigation;
2. inventory exact visibility and workspace/non-workspace consumer classification for each helper;
3. build a test-by-test assertion table for the two mission helpers, subset scenario, duplicate reward failures and
   their named survivors;
4. run nonzero focused baselines on current production and prove which assertions are unique;
5. transfer unique assertions first, rerun the survivor, then remove only the redundant body/branch/helper allowed
   by current compatibility evidence;
6. rerun the exact mission/reward/setup matrices and compare policy outputs/errors;
7. run proportionate Core debug/release/static/broad validation and prove no data/contract/external delta;
8. perform an independent completion audit before closing the Phase.

## DTO, Data, Save And Migration

- DTO/wire migration: none.
- Live RON/data migration: none; current authored mission pairs/reward references must remain valid.
- Save/checkpoint migration: none.
- Public Rust migration: none approved. A public wrapper is retained unless a separate CRA-POL-006 migration gate is
  already satisfied by current evidence.
- Unity/external migration: none.
- Compatibility: no new alias, duplicate wrapper, fallback or deprecated shim is introduced. Existing public surface
  may be deliberately retained as the approved compatibility result.

## Deployment And Compatibility

Core and its in-workspace consumers are validated from one repository state. No coordinated rollout or external
repository edit is permitted. Private cleanup may ship atomically with exact behavior parity. Unknown deployed Rust
consumers remain explicit; the Phase satisfies CRA-POL-006 by retaining any public surface that cannot be safely
depublished, not by assuming absence.

## Focused Validation

Required after discovery:

- exact visibility/caller/reference inventory for every named helper/test;
- assertion-level old/current comparison between each removal candidate and named survivor;
- mission-policy category, SplitRoom and live-pair tests with nonzero counts;
- stronger default Defense scenario test with all unique shape assertions;
- reward-policy empty, featured/required/forbidden and Boss-positive matrix with exact errors;
- scoped diff/legacy search proving only authorized branches/helpers/tests changed.

Exact nonzero names, filters, counts and their old/current results are recorded in E04 through E06; zero-selected
filters are excluded from evidence.

## Broad Validation

At minimum after implementation:

- affected Core library families in debug and representative release;
- `cargo test -p game_core --all-targets` and representative release all-target validation with actual counts;
- workspace all-target checks in debug/release;
- `cargo fmt --all -- --check`, candidate-scoped `git diff --check` and normal Core Clippy diagnostics;
- live-RON integration if current caller/data inventory shows authored mission/reward coupling;
- final expanded status/path hash, prospective hashes and protected-marker check.

## Omitted Validation

- game_server and Unity behavior are provisionally omitted because no DTO/wire/API result change is approved;
  reconsider if current inventory finds a direct boundary.
- Save/checkpoint, WebSocket and performance validation are omitted unless discovery finds a changed carrier/hot path.
- Canonical edits are omitted if and only if policy/runtime behavior is exact and cleanup changes no durable contract.
- Repository-wide final Stage-D validation remains a later Master boundary.

## Completion Conditions

- Current evidence identifies exact helper visibility/callers and every candidate/survivor assertion.
- Identical mission branches are simplified without changing category/SplitRoom/live-pair/Boss results.
- The wrapper is removed only if nonpublic and consumer-safe; otherwise public compatibility is explicitly retained
  without adding a new shim.
- Every unique assertion is transferred before a redundant test body is removed.
- Default Defense and reward empty/featured/required/forbidden/Boss behavior and exact errors remain green.
- No public API, data/RON, DTO/wire, save/checkpoint, external consumer or canonical contract changes.
- No ignored test, fallback, placeholder or adjacent cleanup is introduced.
- Focused/broad debug/release/static validation is green with actual nonzero counts.
- Dirty work and the protected marker are preserved.
- A separate completion audit returns `Complete / High` with no material finding.

## Completion Audit Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P36-C01 | current helper/test/caller graph is exact | visibility, reference, caller and path inventory | complete — audit pass |
| P36-C02 | dirty targets and concurrent work are preserved | pre/post status, scoped diffs, hashes and protected marker | complete — audit pass |
| P36-C03 | candidate and survivor assertions are fully characterized | per-test assertion matrix and nonzero old/current baseline | complete — audit pass |
| P36-C04 | no-op mission branch is removed with exact policy parity | before/after category/SplitRoom/live-pair output | complete — audit pass |
| P36-C05 | wrapper disposition satisfies CRA-POL-006 | visibility/consumer proof or exact public retention | complete — audit pass |
| P36-C06 | only redundant tests are removed after transfer | assertion diff and surviving named tests | complete — audit pass |
| P36-C07 | full default Defense scenario remains exact | objective/unit/route/movement/position/win assertions | complete — audit pass |
| P36-C08 | reward policy remains exact | empty/featured/required/forbidden/Boss cases and exact error | complete — audit pass |
| P36-C09 | DTO/data/save/public/external contracts are unchanged | prospective hashes, live pair and compatibility audit | complete — audit pass |
| P36-C10 | focused/broad/static validation is green | actual counts and failure classification | complete — audit pass |
| P36-C11 | independent completion audit is current | `Complete / High` review and next-Phase boundary | complete — audit pass |

## Activation Protocol

Before Phase-36-directed production content, caller, assertion or test inspection:

1. Master records Phase 35 `complete — Complete / High`, Phase 36 as the sole `ready` Child and Phases 37~44
   pending.
2. Change all three Child headers from `initialized` to `active`; `Implementation Complete` remains `false`, and
   change Master Phase 36 from `ready` to `active`.
3. Fully reread this PLAN, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, the Phase-35 and semantic Phase-27 completion
   audits, original/post-Boss P05-C05 evidence, Final catalog/order/test/gate/policy inputs and Master contract.
4. Capture HEAD, expanded status count/canonical C-sort hash and path-known prospective hashes/status for historical
   mission-policy, setup-test, reward-policy, data/canonical and dependency inputs without reading their content.
5. Record activation evidence before current-tree content/caller/assertion investigation.

Historical audit observations do not waive this gate.

## Stop Conditions

Stop and do not activate Phase 37 if a removed assertion is not present in a survivor; the mission/reward output or
exact failure changes; helper removal would narrow public/unknown-consumer API; current policy is not the audited
Defense/Boss/SplitRoom contract; dirty overlap cannot be separated; a manifest/data/DTO/save/server/Unity/canonical
migration is required; required tests fail or select zero; the protected marker changes; or the independent audit is
not `Complete / High`.

## Initialization Dirty Baseline

- Expanded porcelain status before this Child: 548 rows.
- Canonical C-sort SHA-256:
  `9c03cb1c82f8acdd41686da5291ac30222aa155aa810d28ed7c389b3f4690172`.
- Phase 36 and Phase 37 paths are absent; no implementation Phase is active.
- Protected marker: 57 bytes, SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`.
- These three new Child files are the only expected status-set additions before activation evidence.
- All existing rows, including completed Phase-35 deletion and documents, are user/campaign-owned and preserved.
