# Phase 26 CRA-P04-C04 Experiment Notes

Status: complete
Implementation Complete: true
Candidate: CRA-P04-C04
Master Phase: 26 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Locked Decisions

- This is a behavior-preserving private assembly/placement refactor, not a VN or Monolith behavior correction.
- Definition-to-runtime construction has one private owner; placement remains an explicit caller responsibility.
- Trigger uses `Tail`; Monolith uses `FrontOfQueue`. The factory cannot infer placement from source.
- Existing active VN is never interrupted or replaced by either producer.
- UUID allocation sequence/timing, entry scene, presentation, source, blocking and uncommitted-choice state remain
  exact.
- Trigger eligibility/priority/consumption and Monolith accept/decline/payment/reward remain exact.
- Public/session/serde/V3/RON/DTO/wire/checkpoint shapes remain exact.
- Test-only direct constructors are not production ownership unless discovery proves otherwise.
- Phase 27 is prohibited.

## Open Evidence Slots

1. Activation status, HEAD and normalized dirty hash.
2. Prospective source/test/data/canonical hashes before content inspection.
3. Every production VN session producer and exact duplicated field/conversion list.
4. Definition lookup ownership and missing-definition/error ordering.
5. UUID allocation owner, persistence and ordering relative to validation/mutation.
6. Active/pending runtime queue owners and raw Tail/front operations.
7. Trigger priority/consumption and active-noninterruption behavior.
8. Monolith accept/decline, exchange and node lifecycle behavior.
9. Checkpoint/public/serde/server/Unity projections and consumers.
10. Baseline nonzero test filters and counts.
11. Test-first matrix and smallest implementation evidence.
12. Focused/broad debug/release/static evidence and independent verdict.

## Historical Risk Register

- Moving UUID allocation earlier can consume an ID on validation or lookup failures that previously consumed none.
- A generic enqueue helper can accidentally append Monolith, prepend Trigger or replace the active session.
- Front insertion with an existing active VN must reorder only pending work, not interrupt the active runtime.
- Rebuilding the presentation/session shape can change entry scene, blocking, source or choice initialization.
- Combining assembly with trigger matching can alter trigger priority or consumption.
- Combining assembly with Monolith exchange can alter payment/reward/decline or error order.
- Save/checkpoint projection may depend on exact active/pending order and complete session identity.
- Test fixtures with direct constructors can create misleading static counts; production authority is the refactor
  target, while behavior-observing tests should remain decoupled from private helper names.

## Historical Evidence Summary

- Original P04-C04 found duplicate VN definition lookup, presentation conversion, UUID allocation, entry scene,
  blocking/source and choice initialization in trigger evaluation and Monolith exchange.
- Trigger appends at Tail. Monolith inserts at index zero/FrontOfQueue. That difference is intentional and canonical.
- Reaudit found Boss-era trigger consumers but no centralized session assembly, so the candidate remains valid.
- Exact linear row 26 depends on Phase 25 completion and requires shared assembly while preserving UUID/order,
  noninterruption and blocking.
- Test rebaseline requires strengthened Monolith queue-front and Trigger tail coverage, UUID order, existing-active
  noninterruption and accept/decline behavior.
- Priority is `2 / 1 / 2 / 2 / 2 / 2 / unknown`, known subtotal 11, Tier C characterization first.

## Activation Boundary

No Phase 26 production source, test implementation, live data or canonical content was inspected during Child
initialization. After Master sole activation, reread this entire Child and Phase 25's audit, capture prospective
fingerprints, and only then begin production investigation.

## Dirty Worktree Note

The shared repository contains extensive user-owned modifications. Pre-initialization expanded state is 503 paths at
`aadd6b33...`. Phase 26 must use path fingerprints and scoped diffs, preserve all unrelated changes and never use
reset/restore/clean/stash to manufacture a clean baseline.

## 2026-08-10 — N01 Activation Baseline

- Mandatory post-activation reads completed: Child 196/61/71 and predecessor audit 371/371.
- HEAD `f318414f...`; expanded dirty state 506 entries at normalized SHA-256 `690ad9be...`.
- Exactly three new Phase 26 Goal paths explain the 503→506 increase; production/test/data/canonical status did not
  change during initialization or activation.
- Prospective paths and hashes are recorded in E02. They were discovered by filename only before content inspection.
- Disposition: activation gate complete; current source/caller/test investigation may begin, while production
  mutation still waits for test-first characterization.

## 2026-08-10 — N02 Discovered Contract And Test Boundary

- The candidate's “Monolith producer” is `start_monolith_exchange_if_available` in `map_content.rs`, not the pure
  Monolith ledger module. It runs after persisted/active Shop commit during Shop-node entry.
- Placement means “activate if idle; otherwise place within pending”: Trigger appends after all pending work;
  Monolith prepends before all pending work. Neither placement may replace the active session.
- The shared seam must accept caller-owned missing-definition error context so exact Trigger/Monolith diagnostics and
  lookup-before-UUID order remain unchanged. It must accept source and blocking as inputs rather than infer them.
- Trigger consumption occurs after enqueue; Monolith `Started` occurs before UUID allocation/enqueue. These local
  mutation orders are outside the shared factory and remain exact.
- Runtime session is top-level transaction state and active-only public projection. Run checkpoints restore trigger
  history and preserve the UUID stream, but do not serialize/rewind the active/pending overlay itself.
- Test-first work can live in the existing world facility suite, which has private state/fixture access while testing
  public Shop entry and VN choice behavior. Tests will assert behavior, not the future helper or enum name.

## 2026-08-10 — N03 Test-First Boundary Decision

- Characterization is placed in `node_sessions.rs`, the existing Shop/node integration suite, rather than the
  historical candidate's looser “facility tests” label. This reaches the actual public Shop entry and choice routes.
- A single factory call cannot both perform lookup and allocation without changing Monolith order: current Monolith
  resolves the definition, commits `Available→Started`, then allocates the UUID. The implementation therefore needs
  a shared resolved-definition seed followed by one allocator/constructor.
- Trigger can resolve and assemble immediately. Monolith can resolve, retain its state-machine commit point, then
  assemble. Both can call the same placement method with an explicit policy.
- Missing-definition errors remain caller-authored closures/values. This removes duplicate lookup mechanics without
  normalizing externally observable diagnostic wording.

## 2026-08-10 — N04 Implementation And Compatibility Decision

- The common owner lives with VN progression because it already owns active/pending promotion and has direct access
  to runtime projection semantics. No public module/type was added.
- The resolved seed is intentionally not a queue request: it freezes only authored ID/entry/presentation before any
  UUID allocation. Source, blocking and placement remain explicit caller inputs.
- The placement method returns immediately after idle activation. `FrontOfQueue` and `Tail` differ only when an
  active session exists, exactly matching both historical producers.
- Monolith's local state-machine mutation remains outside the helper, avoiding a second transaction/rollback owner or
  source-specific branching inside the common assembly path.
- Existing direct test fixtures remain. Removing them would couple behavior tests to the private factory and is
  outside the production-duplication target.
- No DTO, save version, RON, checkpoint, trigger data, Monolith policy, server or Unity change is needed. Broad gates
  and independent review remain before completion.

## 2026-08-10 — N05 Broad-Gate Disposition

- All focused and broad Core/server debug/release tests and checks are green with nonzero counts; both crate
  formatters and repository diff validation are green.
- Non-strict Clippy is diagnostic only and succeeds at the established warning baselines. There is no Phase 26-owned
  warning, so unrelated debt remains out of scope.
- The final source graph has one production constructor/UUID authority and one placement authority with an exhaustive
  two-variant policy. Caller-owned Trigger/Monolith behavior remains visible at each call site.
- Public session/state/snapshot files, live data and all four canonicals are byte-identical to activation. Their
  existing wording remains correct because this Phase moves only private construction ownership.
- The current 507-path expanded state and scoped hashes preserve the shared dirty tree. No reset, restore, clean,
  stash, broad formatter mutation or Phase 27 prebuild occurred.
- Disposition: implementation evidence supports all conditions except the independent verdict. The Child is
  `review_pending`, `Implementation Complete` remains false and P26-C10 alone awaits audit.

## 2026-08-10 — N06 Final Disposition

- Independent review found no missing runtime/data/DTO/persistence/consumer/legacy evidence and no cross-component
  conflict. Verdict is `Complete / High`; long-term improvement class is `none`.
- Phase 26 is complete. Its durable seam is one private assembler plus explicit caller placement; public and
  serialized behavior remains owned by the unchanged runtime/canonical contracts.
- Phase 27 may become ready in Master coordination, but no Phase 27 source inspection or mutation is authorized by
  this Child.
