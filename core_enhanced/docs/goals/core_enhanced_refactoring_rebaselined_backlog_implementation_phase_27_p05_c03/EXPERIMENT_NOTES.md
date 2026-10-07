# Phase 27 CRA-P05-C03 Experiment Notes

Status: complete
Implementation Complete: true
Candidate: CRA-P05-C03
Master Phase: 27 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Locked Decisions

- This is a behavior-preserving private authority consolidation, not a preview/setup policy change.
- Ordinary manifested, Boss, Event-authored and Monolith inputs/results remain exact.
- Manifested difficulty remains the scaling authority; Floor must not gain gameplay or cache authority.
- P05-C01 battle setup regenerates from authoritative current inputs and cannot read the serialized presentation cache.
- One private resolved input/conversion/resolver owns shared mechanics; context-specific ownership remains explicit.
- Current public Rust symbols, request fields/signatures and serde/V3/RON/wire shapes remain compatible under
  CRA-POL-006 unless a separately authorized migration gate is satisfied.
- No silent normalization, fallback, alias, dual behavior or broad setup/mission cleanup is allowed.
- Phase 28 is prohibited.

## Open Evidence Slots

1. Activation status, HEAD and normalized dirty hash.
2. Prospective source/test/data/canonical hashes before content inspection.
3. Every public/internal preview-generation request and caller.
4. Every Floor field/parameter read, write and test/external reference.
5. Ordinary/Boss/Event/Monolith resolved input and error order.
6. Both historical instance-to-preview mappings and exact field/order parity.
7. Preview cache keys, hit/miss/publication/replacement/invalidation owners.
8. P05-C01 setup regeneration and P05-C04 setup-plan consumers.
9. DTO/serde/V3/RON/server/Unity consumers and deployment classification.
10. Baseline and test-first selected counts.
11. Minimal implementation and negative duplicate/public-narrowing evidence.
12. Focused/broad debug/release/static evidence and independent verdict.

## Historical Risk Register

- Combining resolvers can reorder validation and cache lookup, changing errors or accepting stale previews.
- A shared input that erases Event versus manifested context can select the wrong category, difficulty or seed.
- Incomplete field mapping can silently default/drop warning, mission, template, spawn, reward or Boss-facing data.
- Reusing a presentation cache at battle start would regress P05-C01's authoritative regeneration.
- Removing Floor-bearing public APIs after only workspace search would violate CRA-POL-006 for unknown deployments.
- Retaining Floor internally without a non-authority proof leaves a misleading future scaling seam.
- Monolith stage changes and Event retry can expose stale cache lifetime/invalidation differences.
- Broad conversion/setup cleanup could absorb P05-C05 or disturb the already-closed typed setup plan.

## Historical Evidence Summary

- Original P05-C03 found two twenty-plus-field mappings, two world resolvers with different cache/validation order and
  an unread `BattlefieldGenerationRequest.floor_index`.
- Target evidence requests one typed resolved input for manifested versus Event combat, one complete conversion and
  one resolver/cache authority.
- Reaudit after Boss runtime says all three duplication/ghost signals remain and the candidate is `still_valid`.
- P05-C01 is complete: serialized previews remain compatible presentation caches but battle setup regenerates from
  current run/live inputs for ordinary/Boss/Event/Monolith paths.
- Final order requires ordinary/Boss/Event/Monolith parity and caller inventory, and limits removal to duplicate
  private mapping under CRA-POL-006.
- Priority is `3 / 1 / 2 / 3 / 3 / 2 / unknown`, subtotal 14, Tier B after C01 and caller inventory.

## Activation Boundary

No Phase 27 production source, tests, live data or canonical content was inspected during Child initialization. After
Master sole activation, reread this entire Child plus final Phase 26 and P05-C01 audits, capture prospective hashes
using path discovery only, and only then begin current-source investigation.

## Dirty Worktree Note

The repository contains extensive user-owned modifications. Pre-initialization expanded state is 508 paths at
`3ecc8e70...`. Phase 27 must fingerprint prospective targets, preserve all unrelated work and never use reset,
restore, clean or stash to manufacture a clean baseline.

## 2026-08-10 — N01 Activation Baseline

- Mandatory post-activation reads completed: Child 212/66/70, predecessor audit 337/337 and P05-C01 dependency audit
  156/156.
- HEAD `f318414f...`; expanded dirty state 511 entries at normalized SHA-256 `31491d69...`.
- Exactly three new Phase 27 Goal paths explain the 508→511 increase. Production/test/data/canonical status was not
  changed by initialization or activation.
- Prospective path hashes are recorded in E02 and were captured before their content was read.
- Disposition: activation gate complete. Current source/caller/test investigation may begin; production mutation
  still waits for baseline and test-first characterization.

## 2026-08-10 — N02 Verified Seam And Compatibility Disposition

- The historical duplicate signal is current: two identical moves of every one of the 19 instance fields into a
  preview plus `node_id` and owned `encounter_id`. A single private conversion owner can replace both without a DTO
  change.
- The two world paths share node/seed/generation mechanics but deliberately differ in cache use, applicability,
  Monolith validation and publication. Their common authority must therefore preserve an explicit presentation
  versus battle-start mode rather than flattening those semantics.
- Cache-hit-before-category is an Event retry contract. Fresh generation at battle start is the P05-C01 contract.
  Monolith validation/invalidation remains presentation/reward-flow owned. These are testable order constraints, not
  incidental implementation details.
- The preview request's Floor field and Floor-bearing public wrappers are demonstrably unread by battlefield
  generation. They stay source-compatible and will receive a non-authority comment/test; no public removal,
  deprecation, serde change or replacement API is authorized.
- Workspace and canonical external searches cannot prove absence of deployed Rust callers. CRA-POL-006 resolves the
  policy question: retain all public functions, request fields and signatures, and remove duplicate private mapping
  only.
- Old-production focused baseline is green at 27 + 11 + 42 + 3 + 19 + 1 selected cases. Production editing remains
  blocked until new complete-value characterization tests are added and shown green against that old code.

## 2026-08-10 — N03 Test-First Gate Passed

- New old-production evidence adds complete value comparison, not selected-field comparison, for both duplicate
  conversion entrypoints and both world resolver roles.
- The world characterization intentionally uses the real public behavior sequence. This avoids changing private
  visibility merely for testing and covers cache publication, cache-hit precedence, battle-start regeneration and
  commit replacement together.
- Ordinary and Boss run through the same assertion body. Existing Event defeat/retry and Monolith stage/invalidation
  suites supply the two special-context branches; all remain green after characterization.
- The original implementation passes the new tests, so they are refactor characterization rather than a disguised
  policy change. Smallest production consolidation may begin.

## 2026-08-10 — N04 Implemented Authority Boundary

- Conversion ownership is now structurally complete: generation and validation happen before exactly one consuming
  conversion, so omitted/defaulted fields cannot differ by caller family.
- Resolver ownership is typed rather than boolean. `actual_category` retains live-node authority for required
  difficulty errors, while `generation_category` and `encounter_id` retain explicit Event-authored battle inputs.
- Presentation mode checks its cloned cache before applicability/difficulty/Monolith gates exactly as before.
  Battle-start mode never obtains a cache value and does not acquire presentation-only Monolith validation.
- Difficulty still comes from the actual manifested node; category/encounter/seed retain their original sources.
  Floor is passed through for compatibility but remains unread by generation.
- All focused families and persistence/publication checks are green. No migration, fallback, alias, deprecation,
  public visibility widening or P05-C05 cleanup was introduced.

## 2026-08-10 — N05 Broad Green; Preservation Stop

- All required Core/server debug and release tests/checks are green. Non-strict Clippy confirms only established
  cross-tree warning debt; no Phase 27 warning was introduced.
- I deleted a pre-existing 688,108-byte untracked server log after misclassifying it as a test artifact. Exact status
  reconstruction proves it belonged to the activation dirty set. It is not recoverable through Git, open handles,
  rotation copies, workspace/F:-drive copies or Recycle Bin.
- The implementation itself remains bounded to four source/test paths plus the three Child documents, and all
  canonical/data hashes remain unchanged. Nevertheless, preservation is a completion condition, so functional green
  evidence cannot silently override the incident.
- Stop state: keep Phase 27 active, `Implementation Complete: false`, do not write a passing completion audit and do
  not activate Phase 28 until the user explicitly decides whether this diagnostic-log loss is accepted as a waived
  dirty-boundary exception or requires external recovery action.

## 2026-08-10 — N06 Independent Audit Disposition

- The completion reviewer independently classifies the work as `Partially complete / High`: the implementation is
  complete and durable, but preservation is part of completion rather than an optional quality attribute.
- The audit therefore records P27-C02 and downstream gate P27-C10 as failures while leaving every functional,
  compatibility and validation condition passing. This avoids rewriting the incident as functional failure or
  hiding it behind green tests.
- Phase 27 stays the sole active Phase. Phase 28 has not been initialized or modified.
- Required user decision: restore the exact original log from an external source, or explicitly accept its loss and
  authorize a one-path dirty-boundary exception. Either choice is evidence for a new audit; it is not retroactively
  inserted into the current audit.

## 2026-08-10 — N07 Approved Exception Boundary

- The newly created 57-byte file at the deleted path is an explicit approval marker, not a recovered log. Preserve
  that distinction in every later audit and final report.
- Approved exception: accept the irrecoverable loss of the original 688,108-byte
  `game_server/logs/app.log.2026-08-09` only. No other dirty-work loss, replacement or cleanup is authorized.
- The marker's own bytes and path are new user-owned state and cannot be deleted after serving as approval evidence.
- Functional conditions do not inherit a waiver. Phase 27 still requires a new scoped dirty comparison and complete
  independent review; Phase 28 stays prohibited until that review is `Complete / High`.

## 2026-08-10 — N08 Preservation Reaudit Result

- Exact normalized status-set comparison proves every activation row is accounted for after excluding only the two
  intended Phase-added rows. The approved exception is therefore neither a general waiver nor a substitute for
  evidence about unrelated dirty work.
- The current marker at the old log path does not satisfy original-byte restoration and must never be described as
  such. It is retained solely as the durable explicit approval record and new user-owned state.
- Functional/source/data/canonical hashes are unchanged and fresh focused tests are green. No material finding now
  remains outside the already approved one-path loss; the independent reviewer may reassess P27-C02 and P27-C10.
