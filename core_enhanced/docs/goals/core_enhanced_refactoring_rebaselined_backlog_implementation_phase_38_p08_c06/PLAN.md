# Phase 38 CRA-P08-C06 Disconnected DeathHandler Cleanup Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — the user's 2026-08-09 instruction to execute the Master campaign
Parent Master: `core_enhanced_refactoring_rebaselined_backlog_implementation_master`
Phase: 38 of 44
Candidate / Work Type: CRA-P08-C06 / `cleanup`
Direct Linear Dependency: Phase 37 CRA-P06-C04 `Complete / High`
Direct Technical Dependencies: Phase 11 CRA-P08-C01, Phase 12 CRA-P08-C02 and Phase 28 CRA-P08-C05, all
`Complete / High`
Direct Policy: CRA-POL-006
Repository / Start Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Initialization Date: 2026-08-11 (`Asia/Seoul`)

## Objective

Eliminate the disconnected public `battle::death` false authority only after its sole useful sorted/deduplicated
ally-occurrence invariant is expressed against the real production death lifecycle. First refresh the complete
current module/export/type/test/consumer graph, characterize the legacy assertion and the production
`finalize_unit_death` behavior, transfer equal or stronger observable coverage to production death tests, and then
remove or depublish the legacy module only when CRA-POL-006 consumer evidence authorizes the public narrowing.

## User/Contract-Visible Problem

The historical public `DeadUnit`, `DeathProcessResult` and `DeathHandler` model was not used by `BattleCore`. It
removed units from a separate map, queued deaths and handled only Modifier subsets, while the real death path
retained runtime units and owned lifecycle, movement/field-work interruption, buff cleanup, graveyard, event
lineage, causal kill credit and all supported OnDeath/OnKill/OnAllyDeath effects and activations.

Its lone green test therefore protected a partial alternate model. Immediate deletion would still be unsafe: the
test carried one useful deterministic ally ordering/deduplication intent, and the module was public Rust API. The
assertion must move to actual production behavior before its carrier disappears, and a negative workspace search
alone cannot settle deployed/standalone consumers.

## Audit Finding ID

CRA-P08-C06. The final catalog classifies it `still_valid`, `cleanup`, Tier D, priority axes
`3/1/2/1/2/1/unknown = 10`, ready after P08-C01, assertion transfer and external evidence. Exact linear item 38
requires P08-C01, P08-C02 and P08-C05, CRA-POL-006 and public-consumer inventory; its first proof is that sorted and
deduplicated ally occurrence behavior lives in production death tests before module deletion or depublication.

## Historical Evidence And Current-Proof Gate

Historical original/post-Boss evidence prospectively locates the candidate at
`core_enhanced/src/game/battle/death.rs`, its public module export, and one self-contained unit test. It records:

- `battle::death::{DeadUnit, DeathProcessResult, DeathHandler}` had no workspace production consumer;
- the legacy handler removed units and executed only a subset of modifier effects;
- production `core::death_runtime::finalize_unit_death` was the sole workspace lifecycle authority;
- the one legacy test asserted queued-death deduplication and sorted ally occurrence;
- runtime death tests protected graveyard, movement and buff cleanup but required complete occurrence
  ordering/deduplication coverage;
- external public Rust consumers were unverified.

Those observations are locating hypotheses, not current deletion authority. After sole activation this Phase must
refresh exact paths, visibility, reexports, constructors, callers, test names, assertion semantics, package/public
metadata and accessible consumer/deployment evidence against the activation dirty tree before any production edit.

## Source Of Truth And Ownership

### Current prospective authority

- `core::death_runtime::finalize_unit_death` owns real production death finalization.
- Phase 11's typed occurrence dispatcher owns deterministic OnDeath, OnKill and OnAllyDeath effect/activation
  planning; Phase 12 preserves causal modifier death and Phase 28 preserves the committed-damage handoff.
- The historical top-level `battle::death` module owns a disconnected public data/queue model and its lone test.
- Public exports, Cargo/package metadata, repository/deployment evidence and actual accessible consumers jointly
  determine whether the legacy module may be narrowed or removed.

### Target authority

Only `finalize_unit_death` plus the typed occurrence dispatcher remains as production death behavior. Durable
production death tests observe one real finalization, deterministic OnDeath then OnKill then UUID-sorted unique
OnAllyDeath delivery, dead-caster rules, effect-before-activation, causal lineage, full cleanup and graveyard state.
No parallel public death model remains only when CRA-POL-006 evidence authorizes its deletion or depublication.

## Invariants

- A unit enters the production death lifecycle at most once; duplicate/queued inputs cannot duplicate death,
  graveyard, kill credit or occurrences.
- Production occurrence order remains OnDeath, then causal OnKill when applicable, then unique active allies in
  deterministic UUID order.
- Effects remain before activations within each occurrence, with the established dead-caster exception only for
  OnDeath.
- Causal source, parent/event lineage, kill credit and same-time nested command order remain exact.
- Runtime units remain retained with Dead lifecycle while movement, action, field-work and owned/targeting buffs are
  cleaned by the existing authority.
- Ordinary committed damage and lethal MaxHealth clamping keep their Phase 12/28 handoff and do not gain a second
  death implementation.
- World incapacitation/result aggregation and Boss released-work behavior remain unchanged.
- Public Rust symbols or module exports are not narrowed from a negative workspace search alone.
- No shim, alias, deprecated duplicate, fallback, dual death path, ignored test or placeholder is introduced.
- No RON/content, DTO/wire, V3 save/checkpoint, server/Unity or durable canonical meaning changes.
- Phase 39 and later cleanup are not pulled forward.
- Existing user/campaign dirty work and the protected Phase-27 approval marker remain exact.

## Refactor Candidate And Behavior-Preservation Boundary

- Candidate ID: CRA-P08-C06.
- Work type: `cleanup`; `still_valid`; Tier D; after C01/assertion transfer/external evidence.
- Priority axes: ownership `3`, correctness/atomicity `1`, lifecycle `2`, change amplification `1`, test fragility
  `2`, failure blast radius `1`, change frequency `unknown`; known subtotal `10/18`.
- Prospective removal scope: current equivalents of the disconnected public death types/handler/module export and
  its sole false-authority unit test, after exact enumeration, assertion transfer and consumer authorization.
- Retained scope: production death finalization, occurrence dispatcher, committed damage/HP handoff, runtime event
  types and all real death/skill/item/world tests.
- First safety net: reproduce the legacy sorted/deduped ally intent and express it through real
  `finalize_unit_death` output/state before any legacy carrier is removed.
- Behavior-preservation boundary: any runtime lifecycle/order/event/data/public carrier change beyond the explicitly
  authorized legacy removal, or any Phase-39+ cleanup, stops implementation.

## Approved Policy And Evidence

- Original P08-C06: transfer the sole useful dedupe/sorted-ally assertion to production death occurrence tests,
  add complete effect/activation chaining evidence, then compile-search and run death/world families.
- Post-Boss P08: production `finalize_unit_death` is the authority; the public partial model remained disconnected
  and its cleanup remained gated by assertion transfer and external Rust consumer inventory.
- Final test catalog: `transfer then remove`; move the sole sorted/deduped ally occurrence invariant into production
  death tests and inventory public consumers.
- Exact linear item 38: P08-C01/C02/C05 precede cleanup and the production assertion must exist before deletion or
  depublication.
- Phase 11: one typed dispatcher owns all six item occurrences and deterministic target ordering.
- Phase 12: lethal runtime stat mutation enters the same causal production death authority.
- Phase 28: one committed-damage authority hands lethal HP state to the retained death authority without changing
  public carriers.
- Phase 37: the direct linear predecessor is independently `Complete / High` and created no Phase-38 placeholder.
- CRA-POL-006: public Rust/package surfaces require a consumer inventory plus deprecation/versioned migration or a
  new explicit breaking approval before narrowing.

## Resolved Policy Decision

Master PLAN row 235 now records the user's answer `승인` for the exact Phase-38 public-removal question. This is the
candidate-scoped CRA-POL-006 breaking approval for removing
`game_core::game::battle::death::{DeadUnit, DeathProcessResult, DeathHandler}`, the `death` module export and its
lone self-test on the authoritative statement that no external consumer exists.

The approval does not extend to production death/damage/event carriers, gameplay/data/DTO/save/wire meaning,
adjacent cleanup or Phase 39. Assertion transfer is already green, so Phase 38 may resume as the sole active Child
and apply only the approved cleanup before focused/broad validation and an independent exit audit.

## Current Runtime And Live Data Evidence

At initialization, no Phase-38 current production content, caller, test, consumer or data path has been inspected.
The historical paths and completed dependency audits above are the only locating evidence. Current facts remain
`discovery pending` until the Child is the sole active Phase, all three documents and direct audits are reread, and
prospective path-only fingerprints are recorded.

## In Scope

- exact current file/module/export/public type/method/test graph for the disconnected death surface;
- exact current production death, occurrence, HP/committed-damage and world consumer graph needed to prove scope;
- old legacy test behavior and its unique sorted/deduped invariant;
- real production death characterization covering duplicate input, occurrence order, unique sorted allies,
  effect-before-activation, dead-caster, cleanup, graveyard and lineage as applicable;
- public Rust/package/tool/repository/deployment consumer classification under CRA-POL-006;
- assertion transfer before any deletion/depublication;
- the smallest consumer-authorized removal, exact test/file/symbol delta and no replacement false authority;
- focused/broad debug/release/static validation, dirty preservation and a separate completion audit.

## Out Of Scope

- changing death gameplay, occurrence semantics, causal kill credit, event ordering or runtime state layout;
- refactoring `finalize_unit_death`, the typed occurrence dispatcher or committed-damage authority beyond test-only
  access strictly required for observable production proof;
- removing or narrowing public damage/event carriers retained by Phase 28;
- changing live RON/schema/content, DTO/wire, V3 save/checkpoint, server/Unity or Boss publication;
- adjacent movement, ghost-field, event-log I/O, range or package cleanup from Phases 39~44;
- unrelated formatting, lint or dirty-worktree cleanup.

## Implementation Scope

After sole activation and prospective fingerprints:

1. refresh the exact legacy death module/export/type/test graph and all accessible consumers;
2. refresh the production death/occurrence/HP/committed-damage ownership graph without changing it;
3. establish nonzero old/current baselines for the legacy test and affected production death families;
4. isolate the legacy test's unique sorted/deduped intent and add an equal-or-stronger real-production control;
5. prove the production assertion against old/current production before deleting any legacy test or API carrier;
6. classify each public symbol/module as retain, migrate/deprecate, explicitly approved break or externally blocked;
7. apply only the authorized cleanup with no runtime behavior/data/serialization change;
8. rerun exact discovery, focused/broad debug/release/static checks and dirty/marker verification;
9. perform an independent completion audit and transition linearly only on `Complete / High`.

## DTO, Data, Save And Migration

- DTO/wire migration: none.
- Live RON/data migration: none.
- Save/checkpoint migration: none.
- Runtime event serialization migration: none.
- Public Rust migration: no deprecation shim; Master row 235 records explicit candidate-scoped breaking removal
  approval for the disconnected `battle::death` surface.
- Unity/external migration: none.

## Deployment And Compatibility

Core and accessible in-workspace consumers are validated from one repository state. Package metadata, public docs,
fixtures/tools and accessible deployment records must be inventoried affirmatively; workspace symbol absence alone
does not prove external absence. No coordinated rollout or external repository edit is authorized. If deletion is
required but consumer authority remains unknown, stop the Master tail at this Phase rather than adding compatibility
code or silently weakening the goal.

## Focused Validation

Required after current discovery:

- exact module/file/export/public-symbol/test and caller/consumer inventories;
- nonzero legacy `DeathHandler` test baseline and assertion-by-assertion disposition;
- current production death baseline through real `BattleCore`/`finalize_unit_death` behavior;
- a test-first production control proving unique UUID-sorted ally occurrence under duplicate/competing inputs, plus
  effect/activation and lifecycle assertions needed to prevent the partial model from surviving as authority;
- exact pre/post test-name, file, module and public-symbol deltas for any authorized cleanup;
- zero legacy/fallback/placeholder/ignored replacement search.

Exact commands, selected cases, pass/fail/ignored counts and old/current results are recorded after activation.
Zero-selected filters, compile-only success and direct testing of a replacement private helper do not count.

## Broad Validation

At minimum after implementation:

- complete affected Core death/commands/item/skill/world families in debug and representative release;
- `cargo test -p game_core --all-targets` in debug and release with exact per-target counts;
- workspace all-target checks in debug/release;
- `cargo fmt --all -- --check`, full `git diff --check` and normal Core Clippy diagnostics;
- live-RON integration when the selected production proof consumes authored item/skill data;
- final status/path hashes, prospective non-target hashes, public/legacy searches and protected-marker verification.

## Omitted Validation

- game_server and Unity runtime/build are provisionally omitted because no DTO/wire/runtime result change is
  approved; reconsider if current consumer discovery finds compilation or fixture coupling.
- WebSocket/live server probes are omitted unless transport behavior becomes relevant.
- Save/checkpoint and performance validation are omitted because no persistent carrier or algorithm change is
  approved.
- Canonical edits are omitted if and only if gameplay/data/wire meaning remains exact and only a disconnected Rust
  surface is removed. Any migration or behavior change requires policy first.
- Repository-wide Stage-D boundary validation remains a Master requirement after Phase 44.

## Completion Conditions

- The current legacy death files, export, public symbols, methods, tests and consumers are exactly identified.
- The current production death/occurrence authority and all direct technical dependency seams remain singular.
- The sole useful sorted/deduped ally invariant has equal-or-stronger real-production coverage before old carrier
  removal.
- Production tests directly prove exactly-once death, deterministic unique ally order and relevant effect/
  activation/lifecycle behavior without depending on the legacy model.
- The disconnected module/test is removed or depublished only with affirmative CRA-POL-006 evidence; otherwise the
  Phase is explicitly blocked and not complete.
- Runtime death state, events/order/lineage, graveyard/cleanup, damage/HP handoff, world results and Boss behavior
  remain exact.
- No RON/data, DTO/wire, save/checkpoint, server/Unity, canonical or later-Phase cleanup change occurs.
- No shim, alias, fallback, dual path, ignored test or placeholder remains.
- Focused/broad debug/release/static validation is green with actual nonzero counts.
- Dirty work and the protected marker are preserved.
- A separate completion audit returns `Complete / High` before Phase 39 becomes ready.

## Completion Audit Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P38-C01 | current legacy graph is exact | file, export, symbol, visibility, constructor/caller and test inventory | complete — E03 |
| P38-C02 | production death authority graph is exact | finalize/occurrence/HP/damage/world caller and ownership inventory | complete — E03 |
| P38-C03 | dirty targets and concurrent work are preserved | pre/post status, scoped diffs, hashes and marker | complete — E02/E04/E05/E07 |
| P38-C04 | legacy assertion meaning is characterized | nonzero old test plus assertion-by-assertion disposition | complete — E03 |
| P38-C05 | real production death is characterized | nonzero current controls and observable state/event order | complete — E03 |
| P38-C06 | sorted/deduped assertion moves first | named production test passing before cleanup | complete — E04 |
| P38-C07 | public consumer/removal gate is exact | workspace/package/tool/deployment inventory and policy disposition | complete — candidate-scoped breaking approval in Master row 235; E03/E05/E06 |
| P38-C08 | only authorized false authority is removed/depublished | exact file/module/symbol/test delta and no parallel model | complete — E07 |
| P38-C09 | runtime lifecycle/order/data behavior remains exact | focused production death/occurrence/world evidence | complete — E03/E04/E07 |
| P38-C10 | DTO/RON/save/server/Unity compatibility is exact | prospective hashes, searches and affected package checks | complete — E02/E05/E07 |
| P38-C11 | focused/broad/static validation is green | actual test counts, profiles and failure classification | complete — E07 |
| P38-C12 | independent audit and next boundary are current | `Complete / High`; Phase 39 absent until exit | complete — 212-line `COMPLETION_AUDIT.md` at `b1063d8b...` |

## Activation Protocol

Before Phase-38-directed production content, caller, test or consumer inspection:

1. Master records Phase 37 `complete — Complete / High`, Phase 38 as `ready` and Phases 39~44 pending.
2. Change all three Child headers from `initialized` to `active`; `Implementation Complete` remains `false`, and
   change Master Phase 38 from `ready` to `active`.
3. Fully reread this PLAN, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, the Phase-37 completion audit, Phase 11/12/28
   dependency audits, historical P08-C06 evidence, Final catalog/order/test/gate/policy inputs and Master contract.
4. Capture HEAD, expanded status count/canonical C-sort hash and filename-only/path-known prospective status/hashes
   for the historical death module/export, production death/tests, manifests, data/canonicals and dependency inputs
   without reading their current content.
5. Record activation evidence before current-tree content, caller, test or external-consumer investigation.

Historical audit observations do not waive this gate.

## Stop Conditions

Stop and do not activate Phase 39 if the unique assertion lacks a real production replacement; the new control
tests only a helper/private shape; production lifecycle/order/data meaning changes; consumer evidence is
insufficient for required public narrowing; a migration/break requires unrecorded approval; dirty overlap cannot be
separated; a later-Phase, RON, DTO, save, server, Unity, canonical or package change is required; any mandatory test
fails or selects zero; the protected marker changes; or the independent audit is not `Complete / High`.

## Initialization Dirty Baseline

- Expanded porcelain status before this Child: 568 rows.
- Canonical C-sort SHA-256:
  `8e45c51357bed53aecdd7748743e298af9f53c107fcb52ce30a4bb936f99a5c9`.
- Phase 38 and Phase 39 paths were absent; no implementation Phase was active.
- Protected marker: 57 bytes, SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`.
- These three new Child files are the only expected status-set additions before activation evidence.
- All existing rows, including completed Phase-37 source/test/doc changes, are user/campaign-owned and preserved.
