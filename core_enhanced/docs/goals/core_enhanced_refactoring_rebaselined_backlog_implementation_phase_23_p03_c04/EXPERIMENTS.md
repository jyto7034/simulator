# Phase 23 CRA-P03-C04 Experiments

Status: complete
Implementation Complete: true
Candidate: CRA-P03-C04
Master Phase: 23 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Experiment Protocol

- Do not inspect production until Phase 23 is the sole active Master row and all activation reads are complete.
- Treat historical caller names and topology as hypotheses until rediscovered in the current tree.
- Preserve the public `GrantExecutor` API and every effect/result/schema/consumer contract.
- Record expanded dirty status and prospective source/test/data/canonical hashes before production detail.
- Characterize exact five-root state and every caller's outer transaction before refactoring.
- Do not prebuild Phase 24 Event/Shop commit or Phase 25 HQ transaction work.
- Only an independent `Complete / High` audit may close this Child and release Phase 24.

## 2026-08-10 — E01 Historical Evidence And Initialization

### Evidence Read

- Phase 22's final 299-line audit is `Complete / High`; Master makes Phase 23 ready only after that exit.
- Original P03 evidence identifies five repeated mutable roots: inventory, skill fragments, roster, UUID manager and
  Enkephalin. `GrantExecutor` owns ordered effects; callers own clone/commit atomicity.
- Re-audit keeps CRA-P03-C04 `still_valid`, axes `3/1/2/3/2/3/unknown = 14`, Tier B, caller parity first. Boss result
  handling reuses the existing combat/grant path and adds no new ledger.
- Exact order places this candidate at 23, before P04-C03 and the dependent F-C02. The test catalog requires exact
  five-ledger state, diff order, UUID/seed and late failure across every caller.
- Historical public/consumer gate forbids removing or narrowing the public executor solely because workspace callers
  migrate. Private aggregation itself needs no policy decision.

### Locked Scope

- Consolidate only the common five-root capture/pass/commit family.
- Preserve effect/error/diff order, per-effect seed, UUID continuation, checked arithmetic, context/target behavior
  and exact caller no-commit boundaries.
- Retain shop/event/combat/admin/reward/starter/maintenance outer owners and public compatibility.
- Exclude independent public-executor rollback, session merging, transaction widening, Phase 24/25 and content work.

### Next Experiment

After sole activation and mandatory full reads, capture the dirty/source evidence before rediscovering the current
executor, all callers, root types, stage/commit topology, tests and canonical consumers.

## 2026-08-10 — E02 Mandatory Reads And Prospective Fingerprints

### Activation Gates

- After Master sole activation, PLAN 210/210, experiments 44/44 and notes 52/52 were read completely.
- Phase 22's final 299-line audit was reread completely after activation and remains `Complete / High`.
- No Phase 23 production source, test body, live data or canonical content was inspected before the fingerprint.

### Dirty And Prospective Baseline

- HEAD is `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded status is 485 entries at normalized SHA-256
  `99744aa80f44a0eec969ac44a077d4a7562ad7fff4600412c55664d94228eaca`.
- The increase from Phase 22's audit-inclusive 482 is exactly the three newly initialized Phase 23 Child documents;
  Master activation edits changed content of already dirty documents and add no row.
- Prospective production hashes: executor `5e51a825...`, world reward `cc5db810...`, shop `aa66cb4f...`, event
  `cbcab9aa...`, combat `ef896bfa...`, admin grants `5f7851b1...`, helpers `e48b02d9...`, world state `1e5f9177...`,
  UUID manager `f33bcbe2...`, inventory `2b32a864...`, resource state `1a6efb5b...`, economy `3b59a3ce...`, fragment
  inventory `5a0659e0...` and employee `8d3940dc...`.
- Prospective test hashes: node sessions `1edc4175...`, map flow `1fb7e507...`, combat `933df538...`, snapshots/start
  `2144e86e...`, facility `3e13a4c0...` and admin `8a1078a2...`.
- Canonical hashes: runtime `6e20d6a6...`, rulebook `6c53ecd5...`, server bridge `4d734af0...`, external Unity
  `a3224e24...`.

### Next Experiment

Read the complete current executor/root/caller graph, classify each outer transaction and run nonzero focused
baselines before adding five-root characterization.

## 2026-08-10 — E03 Current Caller Graph, Baseline And Characterization

### Current Graph

- The public `GrantExecutor` signature is unchanged and accepts five mutable roots plus database, fragment policy,
  context, effects/reward and seed. It derives `effect_seed = seed ^ (effect_index << 32)`, applies in input order
  and appends each diff family in that order.
- Eight production direct call expressions exist outside the executor: world reward has reward-option apply, generic apply
  and preview; shop has purchase; event has planned grant; combat has per-survivor XP and result/research rewards;
  admin has generic grant. No server or other workspace Rust caller was found.
- Reward and admin clone/commit five local variables. Shop clones the five, subtracts price in staged Enkephalin and
  commits them beside offer/session removal. Event owns a five-field `PlannedEventGrant`. Combat owns the same five
  fields in `StagedCombatResultState` and combines them with condition diff and broader result/node state.
- Starter and maintenance reach the generic world apply/preview helpers; they do not add direct executor callers.
  Public executor rollback, outer sessions and Phase 24/25 remain distinct.

### Baseline

- Pre-characterization filters all passed: node sessions 18, shop 6, map flow 40, combat late-failure 1, admin 16,
  executor serde tests 2, snapshots/start 23 and facility 11; total 117/117, zero failed/ignored.

### Test-First Characterization

- Added one generic preview/apply matrix covering all five roots, two ordered equipment UUID allocations,
  Enkephalin, fragment ownership/research and AliveRoster experience. Preview and apply results must serialize
  identically; preview must leave all five live roots unchanged.
- Strengthened reward and combat late failures to compare the complete five-root serialized state. Strengthened
  event success and combat survival-XP success to prove unrelated roots and UUID streams stay exact.
- Strengthened admin failure to compare all roots and admin equipment success to freeze the exact 51-UUID order and
  continuation. Strengthened shop success and added a corrupt-offer late grant failure preserving all roots and the
  exact shop session.
- The first compile attempt used `ActiveNodeContent` serialization, which that internal enum intentionally does not
  implement. The test fixture was corrected to serialize its public `ShopSessionState`; no production change was
  made. The corrected old implementation passes node sessions 19, shop 7, event 1, combat failure/success 1+1 and
  admin 16 with zero failures/ignored.

### Next Experiment

Introduce one private owned five-root state with capture/apply/commit mapping, migrate only the eight production
call sites and retain every outer transaction owner.

## 2026-08-10 — E04 Private Five-Root Boundary And Caller Transfer

### Implementation

- Added `world::grant_state::EconomyGrantState`, a `pub(super)` owned aggregate containing exactly inventory,
  skill-fragment inventory, employee roster, UUID manager and Enkephalin. `capture` clones them in the former order,
  `commit_into` installs them in the former order, and its two apply methods delegate to the unchanged public
  `GrantExecutor` entry points.
- Reward option apply, generic apply and preview now share the aggregate. Preview captures and applies but never
  commits. Reward option iteration, reward seed derivation, diff merging and result order remain in the reward owner.
- Shop purchase still stages the price subtraction before grant, removes the offer from a cloned session only after
  grant succeeds, commits the five roots, then commits the shop session. Event still plans before committing its
  session. Admin still owns command validation/output.
- Combat embeds the aggregate inside `StagedCombatResultState`; condition recovery, survivor XP, result rewards and
  research effects operate on that same staged aggregate. The condition diff, encounter discovery, run outcome and
  node-completion transactions remain outside it.
- Starter and maintenance retain their existing indirect generic-reward handoff. Phase 24 persisted-active and
  Phase 25 headquarters transaction work were not introduced.

### Removal And Compatibility

- Workspace search now finds `grant_reward_with_state`/`grant_effects_with_state` only in the unchanged public
  executor and the two private aggregate delegates. All eight former world/admin direct call expressions are gone.
- The exact repeated five-field capture/commit sequence occurs only in the aggregate. Remaining per-root clones are
  checkpoint/snapshot, headquarters Phase 25, maintenance/equipment, shop sale or unrelated UUID staging and are not
  competing grant callers.
- `src/game/reward.rs` remains byte-identical to activation at `5e51a825...`. Runtime/rulebook/server/Unity canonical
  hashes remain `6e20d6a6...`, `6c53ecd5...`, `4d734af0...`, `a3224e24...`; no public, serde, RON, V3, DTO or wire
  shape changed.
- Activation contained 485 expanded status rows. The current pre-broad tree contains 492 at normalized SHA-256
  `67920417...`; the changed target rows and new private module are preserved in the already-dirty repository and no
  reset, restore, clean, stash or unrelated cleanup was performed.

### Immediate Validation

- `cargo check --all-targets` passes after migration.
- Focused current-tree total is 120/120: node sessions 19, shop 7, map flow 40, two exact combat-result cases, admin
  16, reward serde 2, snapshots/start 23 and facility 11; zero failed/ignored.
- The full-root late failures, preview/apply equality, exact equipment UUID order/continuation, fragment research,
  XP target, event at-most-once and shop-session assertions all remain green through the new boundary.

### Next Experiment

Run complete Core and server debug/release tests and checks, static compatibility/format/diff gates and diagnostic
Clippy, then hand P23-C01–P23-C09 to an independent phase-exit audit.

## 2026-08-10 — E05 Broad, Release And Compatibility Validation

### Broad Results

- Complete Core debug and release each pass 1001 unit tests plus 51 official integration tests, with zero failures
  or ignored tests.
- Complete server debug and release each pass 59 tests, with zero failures or ignored tests.
- Core and server `cargo check --all-targets` pass in both debug and release profiles. `cargo fmt --all -- --check`
  and repository-wide `git diff --check` pass.
- Non-strict all-target Clippy completes with current repository debt: Core library 82 and library-test 135 warnings;
  server library 3 and library-test 7 warnings. No diagnostic points at `grant_state.rs` or the aggregate delegate/
  commit edits. Strict global `-D warnings` is therefore not claimed.

### Static And Dirty-Scope Results

- Direct executor search returns only the three unchanged public executor definition/self-delegation lines and the
  two private `EconomyGrantState` delegation lines. `PlannedEventGrant` and old direct staged-combat root fields have
  no matches.
- The new module mentions no shop/event persisted-session type and no headquarters transaction. Remaining five-root
  checkpoint/HQ clones are classified as persistence and Phase 25 authority, not grant fallbacks.
- Public executor and canonical hashes remain activation-identical. Final Phase source hashes include aggregate
  `246bb542...`, reward caller `7bef93f8...`, shop `dfeb6030...`, event `b107be74...`, combat `29b56171...`, admin
  `d89359a6...` and world module `c533e0ff...`.
- Pre-audit expanded status is 492 rows at normalized root-relative SHA-256 `679204175dd753b03b05c83c4536d59a979b4ebf4b9aca6e9ce40608a304c081`.
  The dirty tree was preserved without cleanup or destructive operations.

### Review Handoff

P23-C01–P23-C09 claim complete. P23-C10 remains `review_pending` until the 567-line completion guide, complete
current Child, current source/tests/consumers and static/broad evidence are independently reread and the phase-exit
audit returns `Complete / High`.

## 2026-08-10 — E06 Audit-Found Characterization Correction

### Finding And Correction

- The first independent audit pass found no production defect but rejected the initial test matrix as insufficient:
  event covered successful at-most-once grant but not a late grant failure, and admin's missing-equipment failure
  stopped before entering the aggregate.
- Added an event case whose staged Enkephalin effect succeeds before missing equipment fails; it proves all five live
  roots plus persisted and active event-session copies remain exact.
- Added an admin duplicate-UUID case that deliberately makes the first of two staged equipment grants succeed and the
  second collide; it proves none of the five staged roots installs on the late executor failure.
- The first compile attempt compared `GameError` with `assert_eq!`, but that enum intentionally has no `PartialEq`.
  The assertion was corrected to variant matching; this was test code only and no production behavior changed.

### Replacement Evidence

- Both exact new tests pass. The audit-focused matrix is now 122/122: node sessions 19, shop 7, map flow 41, combat
  2, admin 17, reward serde 2, snapshots/start 23 and facility 11.
- The changed test set invalidated the earlier broad count, so complete Core debug/release were rerun and each now
  passes 1003 unit + 51 integrations. Server debug/release 59/59 and its checks remain current because no production,
  library, server or dependency input changed in this correction.
- Core debug/release checks, formatter, repository diff and non-strict Clippy were rerun. Clippy remains 82/135 with
  no diagnostic on the new aggregate or correction tests.
- Static direct-call/future-Phase searches remain clean. Status remains 492 rows at `67920417...`; source hashes are
  unchanged. Updated test hashes are map flow `c823a5c9...` and admin `f21e7d58...`.

### Audit Restart

The initial audit pass is not the exit verdict. Reread the complete updated Child and current evidence, then perform
the phase-exit verdict from the corrected tree.

## 2026-08-10 — E07 Independent Completion Audit

### Replacement Audit Result

- The complete 567-line review guide and corrected Child were reread from the beginning after the characterization
  correction. Current source, all caller families, corrected tests, canonical consumers, static evidence and dirty
  scope were independently checked rather than inheriting the first pass.
- The corrected audit-focused matrix passes 122/122. Complete Core debug/release each pass 1003 unit plus 51
  integrations; current server debug/release each pass 59. Required checks, formatter and diff gate pass.
- All P23-A01–P23-A10 rows pass. Public executor semantics and all public/persistence/wire surfaces remain exact; no
  Phase 24/25 work was prebuilt.

### Exit Verdict

- `COMPLETION_AUDIT.md` is 264 lines and records `Complete / High`, improvement class `none`.
- Audit-inclusive expanded status is 493 rows at normalized root-relative SHA-256
  `d63d5b1640d6f85fa645b3b3ded7588388d061a573852927bf86341764911137`.
- P23-C10 is complete. Phase 24 may become ready only through the Master transition; it was not inspected or mutated
  during this audit.
