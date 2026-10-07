# Phase 38 CRA-P08-C06 Experiment Notes

Status: complete
Implementation Complete: true

## Scope Decision

- This Child owns exactly CRA-P08-C06: assertion transfer and consumer-gated cleanup of the disconnected public
  death model.
- It does not refactor the production death, occurrence, HP or committed-damage authorities completed by Phases
  11, 12 and 28.
- It does not pull forward Phase-39 movement backend cleanup or any later Stage-D item.

## Authority Boundary

- Prospective real authority: `core::death_runtime::finalize_unit_death` plus the typed occurrence dispatcher.
- Prospective false authority: top-level public `battle::death::{DeadUnit, DeathProcessResult, DeathHandler}` and
  its lone partial-model test.
- Target: production tests own the useful sorted/deduped ally invariant; the false model remains only if required by
  a still-supported external compatibility contract.

## Direct Dependency Decisions

- Phase 11 establishes one typed deterministic occurrence dispatcher for OnDeath, OnKill and OnAllyDeath.
- Phase 12 routes lethal runtime stat clamps through the same causal death finalization.
- Phase 28 routes instant/projectile/general committed damage through one HP/death handoff while retaining public
  damage carriers.
- Phase 37 is the direct linear predecessor and is `Complete / High`; it introduces no Phase-38 placeholder.

## Compatibility Decision

- CRA-POL-006 is binding. A workspace-only negative search cannot authorize removal of a public Rust surface.
- Master row 234's explicit breaking approval was scoped to Phase 37's validator surface. It is evidence that a
  candidate-specific decision can close such a gate, not blanket approval for this death module.
- If affirmative package/tool/deployment evidence cannot settle the consumer class, record the exact external block
  or policy question. Do not add a shim, infer absence or mark the candidate complete.
- Master row 235 now records the user's explicit `승인` for the exact Phase-38 breaking-removal question. This closes
  CRA-POL-006 for the disconnected `battle::death` types/export/self-test only; no broader compatibility decision is
  inferred.

## Dirty Baseline

- Pre-Child expanded status: 568 rows.
- Canonical C-sort SHA-256:
  `8e45c51357bed53aecdd7748743e298af9f53c107fcb52ce30a4bb936f99a5c9`.
- Phase 38/39 paths absent; active implementation Child count 0.
- Protected marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, with exact recorded timestamps.
- Existing status rows and concurrent changes are user/campaign-owned.

## Current Evidence Boundary

At initialization, only historical/final documents and completed dependency audits have been read. No current
Phase-38 production file content, symbol graph, test body, consumer, data or canonical has been inspected. Those
facts remain `discovery pending` until sole activation, full Child/dependency reread and prospective fingerprints.

## Sole Activation — 2026-08-11

- Phase 38 is now the sole active Child; Master Phase 38 is `active`, and Phase 39 remains pending and absent.
- The three initialized files are the only status-set additions: 571 rows at canonical C-sort SHA-256
  `68785da9ec0b529b278f26f052882f771309fe4443f87e909de3312283c799a4`.
- No current production content was inspected during activation. Full active-document/dependency reread and
  filename-only prospective fingerprints remain mandatory before current discovery.

## Required First Proof

The cleanup cannot begin with deletion. A real production death test must first prove exactly-once finalization and
the unique deterministic ally occurrence property carried by the legacy test, together with enough event/state
evidence to show it is testing the real authority rather than a replacement model.

## Prospective Fingerprint Gate — 2026-08-11

- The active Child and all direct dependency/final records were reread after activation.
- The legacy target is currently clean: `battle/death.rs`, 244 lines, SHA-256 `f36b55cb...`.
- The production death authority/test carrier is already dirty from completed earlier work:
  `core/death_runtime.rs` at `096a6826...` and `core/mod.rs` at `822bf01e...`.
- The export file is already dirty from Phase 37 at `2ced98c3...`. Any removal must be exactly one attributable
  Phase-38 export delta after the public gate closes.
- Dependency authorities, world tests, live data and canonicals have recorded activation hashes and are preserved
  unless direct evidence proves a candidate-required narrow test-only change.
- Current content discovery is now authorized; production mutation still waits for baseline, first-proof transfer
  and consumer classification.

## Current Discovery — 2026-08-11

- Current source confirms the exact false-authority shape: one clean public file/export, three public types and one
  self-test; no production caller.
- Real production already proves UUID-sorted unique ally effects/activations through the exact Phase-11 test. The
  remaining useful translation of the legacy duplicate-enqueue assertion is duplicate lethal input at the real
  committed-damage/death handoff, not a queue or unit-removal compatibility model.
- Current baseline is legacy 1/1, death-filter 16/16, exact production occurrence 1/1 and commands 43/43.
- `game_core` remains publicly reachable and not marked `publish = false`; only game_server is a workspace dependent,
  while deployed/standalone consumers remain unverified. This is insufficient for deletion under CRA-POL-006.
- Proceed with policy-independent production-test strengthening only. Public cleanup remains stopped pending an
  explicit candidate-scoped breaking approval or approved migration authority.

## Assertion Transfer Complete — 2026-08-11

- The existing real production death control now sends two lethal inputs and proves one damage/kill summary, one
  `UnitDied`, one graveyard entry and unchanged exact OnDeath→OnKill→unique sorted OnAllyDeath effect/activation
  vectors.
- Exact test 1/1, commands 43/43 and death-filter 16/16 are green. No runtime behavior changed.
- This is equal-or-stronger replacement for the only legacy duplicate/sorted assertion. Deleting the public carrier
  is now technically safe inside the repository but remains contractually unauthorized until the external gate is
  resolved.

## Policy Gate — 2026-08-11

- Local consumer inventory is complete: only the legacy self-test uses the public surface; `game_server`, tools and
  accessible `/mnt/f/work` contain no consumer.
- External/deployed consumer absence is not provable from that inventory. The public crate/path and Git remote keep
  the class `unknown_deployment` under CRA-POL-006.
- Phase 38 is `policy_decision_required`. Required authority is either exact breaking-removal approval based on no
  external consumer, or a named deprecation/versioned migration and lifetime.
- The recommended exact break is limited to `battle::death::{DeadUnit, DeathProcessResult, DeathHandler}`, its
  module export and self-test. Runtime death, public damage/event carriers and all other contracts remain excluded.
- Phase 39 remains pending; no cleanup or broad completion audit begins before resolution.

## Policy Resolution — 2026-08-11

- Master row 235 records the required candidate-scoped approval. Phase 38 resumes as the sole active Child.
- The approved action is deletion, without a shim or deprecation duplicate, of `DeadUnit`, `DeathProcessResult`,
  `DeathHandler`, the public `battle::death` export and their sole self-test.
- Assertion transfer remains the mandatory already-green safety net. Production death/damage/event behavior and all
  later-Phase work remain unchanged and excluded.

## Authorized Cleanup And Validation — 2026-08-11

- The approved false authority is gone: one 244-line file, one public export, three public types and one self-test.
  No production runtime implementation changed.
- The exact production occurrence test and focused death/item/skill/world families remain green. Core all-target
  debug/release each pass 1,067 tests, and reconstructing the prior discovery requires exactly the one removed test.
- Workspace compile, format, diff and normal Clippy gates pass. Existing warnings are unchanged in classification
  and outside this cleanup.
- Source search, prospective hashes and the protected marker prove the API/data/DTO/save/wire/Phase-39 boundaries.
- Phase 38 remains active only for its independent completion audit; Phase 39 remains pending and absent.

## Completion — 2026-08-11

- The independent 212-line audit returns `Complete / High`; every completion condition passes and no correction,
  policy question or external blocker remains.
- Phase 38 is complete. Phase 39 is `ready` only and absent; its initialization, dependency/history reread,
  activation fingerprints and sole-active transition remain mandatory.

## Out-Of-Scope Handling

Independent defects or improvements discovered during current inspection are recorded here with evidence and an
owner, but are not implemented, reordered or turned into new Goals automatically.
