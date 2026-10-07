# Phase 25 CRA-F-C02 Experiments

Status: complete
Implementation Complete: true
Candidate: CRA-F-C02
Source Candidates: CRA-P03-C05, CRA-P04-C02
Master Phase: 25 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Experiment Rules

- No production source, test implementation, live data or canonical content is inspected before sole activation and
  the complete activation protocol.
- Every command records selected test count; zero selected cases are not evidence.
- Add complete-state success and late-failure characterization before transaction/context refactoring.
- Compare every non-battle root and separately prove encounter-compendium monotonic exclusion.
- Preserve active-battle rejection, Phase 24 Shop pairing and exact HQ validation/result/error order.
- Reuse the existing common transaction; no parallel snapshot aggregate or future Phase placeholder.
- Preserve unrelated dirty work; no reset, restore, clean, stash or broad formatting.
- Phase 26 work is forbidden.

## Historical Hypothesis To Verify

Headquarters has a private transaction that snapshots/restores ten roots already covered by the common non-battle
transaction. Its completion planner call temporarily removes active HQ content to pass an admission rule. The common
transaction already preserves the complete non-battle state on error while merging/preserving encounter-compendium
monotonic knowledge.

The refactor hypothesis is that a thin HQ delegate plus explicit resolved-HQ completion context can remove both
duplicates without changing any branch behavior. Current source is authoritative and may refine this hypothesis
after activation.

## Required Experiment Sequence

1. Activation/status/prospective-fingerprint capture.
2. Complete common/HQ transaction, state-root, exclusion, completion and caller inventory.
3. Baseline nonzero focused test discovery and runs.
4. Test-first full-state success/late-failure/exclusion matrix on the old implementation.
5. Smallest common-transaction delegation and explicit completion-context change.
6. Static removal/fallback/public/persistence/canonical comparison.
7. Focused then broad Core/server debug/release validation.
8. Independent 567-line completion-guide phase-exit audit.

## 2026-08-10 — E01 Initialization

- Phase 24's final 309-line audit was fully read and Master records it `complete — Complete / High`.
- Original P03-C05/P04-C02, merged F-C02, re-audited and final candidate/order/test/dependency evidence was read before
  this Child was written. Both source candidates describe one HQ transaction/context root cause.
- Pre-initialization expanded status is 499 entries at normalized SHA-256
  `1f576cd8f6dcb7108d3ba54a3b4e14cf3118f852249107bf8309788c86a4a331`.
- This initialization creates only the three required Phase 25 Goal documents. Production inspection/mutation is
  forbidden until Master sole activation and the full Child/predecessor reread.

## Pending Evidence

- activation HEAD/status and prospective hashes;
- exact `GameCoreState` roots and common transaction inclusion/exclusion behavior;
- exact HQ helper and branch/completion caller graph;
- exact active-content bypass and narrow context seam;
- baseline and test-first counts;
- implemented delegate/context and negative legacy search;
- focused/broad/static/canonical results;
- independent verdict.

## 2026-08-10 — E02 Activation Baseline And Prospective Fingerprints

### Activation Gate

- Master registers Phase 25 as the sole active Child and Phases 26–44 remain pending.
- The active 201-line PLAN, 63-line experiments, 72-line notes and final 309-line Phase 24 audit were read completely
  after activation.
- HEAD is `f318414fb0156a67e04c5479000c2a2683fc0c09`. Expanded activation status is 502 entries at normalized SHA-256
  `a8552b7563238f7e6a2f8a58cb1637e85e6fd35dad4773007ade9d6c3ede725c`; the three-row increase from readiness is
  exactly the three Phase 25 Goal artifacts.

### Prospective Source/Test Fingerprints

- Production: Headquarters `1bc7d090...`, world root `c533e0ff...`, node flow `a311f02b...`, state `1e5f9177...`,
  active run `bea18fa6...`, helpers `e48b02d9...`, reward `7bef93f8...`, facility `35ce3dfe...`, Shop
  `aa53d11a...`, map content `6d21eccc...`.
- Tests: common helper `f6c405d5...`, node sessions `dddcb61b...`, map flow `bfbf36a2...`, facility
  `3e13a4c0...`, checkpoint validation `6c269025...`, snapshots/start `2144e86e...`.
- Canonicals: runtime `6e20d6a6...`, rulebook `6c53ecd5...`, server bridge `4d734af0...`, external Unity
  `a3224e24...`.
- A prospective command included nonexistent `world/mod.rs`; `rg --files` established the actual module root as
  `world.rs`, which was fingerprinted before content inspection. This path correction changed no file.
- Most targets are already modified in the shared dirty tree. These hashes are ownership boundaries, not attribution
  of pre-existing work to Phase 25.

### Next Experiment

Production-detail inspection is now authorized. Inventory the common/HQ transaction and completion context, then run
nonzero baseline filters before adding characterization.

## 2026-08-10 — E03 Current Owner Graph And Test-First Characterization

### Transaction And Completion Graph

- `GameCoreState` currently owns 19 roots: player, game state, active content, node session, run, VN runtime, VN
  triggers, UUID manager, Enkephalin, inventory, roster, starter candidates, pending mode, skill fragments, fragment
  policy, encounter compendium, active battle, run checkpoint and run-power authority.
- `clone_non_battle_transaction_state` rejects any active battle before the operation runs, clones every non-battle
  root, and replaces only encounter compendium with default. On failure `with_state_transaction` restores that full
  snapshot and merges the live compendium, retaining both pre-transaction and in-flight monotonic profile knowledge.
- The removed HQ helper cloned/restored only ten roots: game state, active content, node session, run, UUID manager,
  Enkephalin, inventory, roster, skill fragments and checkpoint. Recruit, emergency and open-Shop were its only
  callers. The common node-completion planner owns research delivery, map/progression and lifecycle cleanup.
- The old HQ planner wrapper temporarily `take()`n active HQ content, called the ordinary planner, and restored it.
  Direct public `CompleteNode` is rejected earlier by capability preflight as exact `ActionUnavailable { CompleteNode,
  ContextIrrelevant }`.

### Baseline And Characterization

- Old-implementation baseline selected 88 cases: node sessions 21, map flow 42, facility 11, checkpoint validation 6,
  common compendium transaction 1 and Shop 7; all passed, zero failed/ignored.
- Tests were added before production mutation for complete non-battle rollback after recruit and emergency late
  completion failure, direct-HQ completion rejection without mutation, open-Shop outer late failure with full rollback
  plus profile union, and common-transaction active-battle rejection before operation execution.
- The complete-state fingerprint covers every current non-battle root and separately excludes/asserts encounter
  compendium. Success tests additionally compare every top-level fingerprint root outside each branch's explicit
  mutation set; branch assertions cover the allowed result, queue/candidates, roster/UUID, Enkephalin, persisted/active
  Shop, node/map transition and exact nested completion.
- Test-first selected counts became node sessions 25 and the exact active-battle test 1; both were green on the old
  structure after correcting the direct command's expected preflight error to its observed exact variant.

## 2026-08-10 — E04 Minimal Implementation And Focused Validation

- Added private typed `NodeCompletionContext::{CurrentNode, ResolvedHeadquartersContact}`. The resolved variant is
  accepted only while active content is actually HQ; the same variant on non-HQ content and ordinary context on HQ
  both return `InvalidAction`. The public ordinary wrapper remains `CurrentNode`.
- HQ's completion wrapper now calls that explicit seam directly. The hidden active-content `take()`/restore is gone;
  recruit/emergency still clear content only as their real lifecycle commit.
- Deleted the ten-root `with_headquarters_transaction`. All three HQ branches now use the existing
  `with_state_transaction`; open-Shop retains Phase 24's `commit_shop_session(...RequireRun)` paired commit.
- No branch validation, staging, queue/candidate, grant, result or completion order changed. No public/session/serde/
  V3/RON/DTO/wire/data type changed, and no compatibility fallback was added.
- Focused post-change evidence is 93/93: node sessions 25, map flow 42, facility 11, checkpoint validation 6, common
  compendium transaction 1, active-battle exclusion 1 and Shop 7. `cargo check --all-targets`, formatter and diff
  checks pass; zero focused failures/ignored.
- Runtime `6e20d6a6...`, rulebook `6c53ecd5...`, server bridge `4d734af0...` and external Unity `a3224e24...` remain
  byte-identical. Their current wording confirms planning-return delivery, typed recursive HQ completions, opaque
  server pass-through, one-action HQ consumption and the same `in_shop` handoff.

### Next Experiment

Run the complete Core/server debug and release suites/checks, final static/hash inventory and independent exit audit.

## 2026-08-10 — E05 Broad, Static And Compatibility Gates

### Complete Executable Gates

- Core debug and release `cargo test --tests` each passed 1,011 library tests plus official integrations
  `4 + 4 + 25 + 14 + 4 = 51`; zero failed/ignored. The five Phase 25 tests account for the 1,006→1,011 increase.
- Server debug and release `cargo test --tests` each passed 59 library tests; zero failed/ignored.
- Core and server debug/release `cargo check --all-targets` all passed. Both crate formatter checks and repository
  `git diff --check` passed.
- Diagnostic non-strict all-target Clippy completed with unchanged established debt: Core library 82/library-test
  135, server library 3/library-test 7. No warning points at the Phase 25 enum, HQ delegates or new assertions;
  strict global `-D warnings` remains outside this Phase's gate.

### Final Ownership And Fingerprints

- Negative search finds no `with_headquarters_transaction`, old rollback variable, or HQ active-content `take()`.
  Positive search finds three HQ common-transaction calls, the one resolved-HQ planner call, ordinary current-node
  wrapper, and Phase 24 `RequireRun` Shop commit. Phase 26 factory/placement symbols remain absent.
- Final production hashes: Headquarters
  `72306a6451e26ffda6068051e6616ce0576097e3cfdfce2596eeb9c0e0f23fd2`, node flow
  `ce97869fcaa07557e2b361220ba1d595f015542bb0947eb0cd845699beb5c13f`.
- Pre-audit Phase 25 test hashes: common helpers
  `95fb353bc886d6dcbed2f10f20ef1f92b84f99b794e7c5b3f36c56994fad6cee`, node sessions
  `44d1131839918e9625cc855ddc784bd608074fe6c92cef3db9b69244ae0f515f`, combat
  `0ef4e29316c67fcfb6b7299bedeadccde707ff626bf3014ea046eb93883a9aff`; E06 supersedes the first two after the
  audit-driven assertion strengthening.
- Unchanged adjacent authorities retain activation hashes: state `1e5f9177...`, Shop `aa53d11a...`, map content
  `6d21eccc...`, active run `bea18fa6...`, helpers `e48b02d9...`, facility `35ce3dfe...`, reward `5e51a825...`.
  Canonicals remain exact at runtime `6e20d6a6...`, rulebook `6c53ecd5...`, server `4d734af0...`, Unity
  `a3224e24...`.
- Expanded dirty status is still exactly 502 paths at normalized SHA-256
  `a8552b7563238f7e6a2f8a58cb1637e85e6fd35dad4773007ade9d6c3ede725c`. Source/test targets were already dirty at
  activation; Phase 25 added no status row beyond its three initialized Goal artifacts, and unrelated work remains
  preserved.

### Review Handoff

All implementation gates are green. Status is `review_pending`; P25-C10 alone remains pending until the complete
authoring/execution and completion-review guides are independently applied to current evidence.

## 2026-08-10 — E06 Audit-Driven Evidence Correction And Revalidation

- The independent review fully read the current 874-line authoring/execution guide and 567-line completion-review
  guide, then the 204/185/126-line Child and current runtime/data/transport/consumer evidence. It found no production
  defect, but found that success assertions allowed the entire serialized `run_payload` root and therefore did not
  independently prove every unaffected nested run field unchanged.
- Because the Phase explicitly requires complete-state success evidence, the test helper was strengthened to compare
  every serialized `RunCheckpointPayload` field. Recruit permits only map, map progression, recruitment, roster and
  UUID manager; emergency permits only map, map progression, recruitment and Enkephalin; open-Shop permits only Shop
  sessions and recruitment. Every other nested run field must remain byte/value equal. The three success tests pass
  with those exact allowlists.
- The current node-session filter passed 25/25. During the first audit focused batch,
  `cargo test --lib game::shop::tests` selected zero because the module namespace was incomplete; that command is
  explicitly discarded as evidence. The corrected `cargo test --lib game::world::shop::tests` passed 7/7. Together
  with map-flow 42, facility 11, checkpoint validation 6 and two exact transaction tests, current focused evidence is
  93/93 with zero failed/ignored.
- After the correction, Core debug and release complete suites were rerun: each passed 1,011 library tests plus
  official integrations `4 + 4 + 25 + 14 + 4 = 51`, zero failed/ignored. Debug/release all-target checks, formatter,
  diff check and current non-strict Clippy also pass; Clippy remains 82/135 established warnings. Server production
  and all Core production dependencies are unchanged by this test-only correction, so the already-current debug/
  release 59/59 and all-target server gates remain valid.
- Final test hashes are common helpers
  `c8d52c36fae590a4a49c2b0a3d62e088a2c898140729cbffe85e32cd4fcfde47`, node sessions
  `541eac53722ecb3434e6c1368eed90a8ecf7dba7232a3e1623e3c2e19c1aa65a`, combat
  `0ef4e29316c67fcfb6b7299bedeadccde707ff626bf3014ea046eb93883a9aff`. Production hashes and canonical hashes are
  unchanged from E05. Dirty status remains 502 rows at `a8552b75...`.

### Audit Restart

The evidence gap is closed. Reapply the current Child evidence matrix, source/consumer/static checks and verdict
criteria from the beginning; the earlier pre-correction review cannot itself authorize completion.

## 2026-08-10 — E07 Independent Completion Verdict

- The restarted current-state audit is recorded in the 371-line `COMPLETION_AUDIT.md` and returns
  `Complete / High`, improvement class `none`; all P25-C01..C10 pass and no material finding remains.
- The audit artifact raises expanded status from 502 to 503 paths at normalized SHA-256
  `aadd6b3327bbda94719be9660aad2e3a7028e7126f30fa79945f1b2069984e0c`. This one-row increase is exactly the new
  audit document; unrelated dirty work remains preserved.
- Phase 25 transitions to `complete`, `Implementation Complete: true`. Master Phase 25 becomes
  `complete — Complete / High`, and Phase 26 CRA-P04-C04 becomes `ready` only.
- Phase 26 production inspection remains forbidden until this final audit is fully reread, historical Phase 26
  evidence is reviewed, and its three Child documents are initialized, registered as the sole active Phase and fully
  read under the activation protocol.
