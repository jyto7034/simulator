# Phase 06 Goal — CRA-P05-C01 Keep Serialized Preview Cache Out Of Setup Authority

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P05-C01
Master Phase: 06 of 44
Work Type: correctness correction
Production Mutation Authorized: yes — the user's 2026-08-09 Master execution request remains active; authorization
is limited to this Phase after Phase 05's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-09 (`Asia/Seoul`)

## Objective And Completion Conditions

Ensure a serialized combat-preview cache can never become authoritative battle setup after checkpoint/active-run
Continue. Battle start must recompute current setup inputs from authoritative run node identity, manifested
difficulty, live data and deterministic setup authority, even when a structurally valid persisted preview cache was
mutated before reload. Preserve the current V3 field and outward preview/wire shape under CRA-POL-006.

This Phase is complete only when current preview generation/cache, cancel, SavePoint/checkpoint export/import,
Continue, battle-start resolution, enemy stat/difficulty and active-session commit paths are identified; an exact
preview→cancel→SavePoint→durable Continue→start test reproduces setup corruption from a mutated serialized cache;
the smallest authority correction removes the cache from setup input without deleting/changing its serialized
surface; ordinary/Boss/Event/Monolith and deterministic setup contracts remain exact; focused/broad validation
passes; and a phase-exit audit returns `Complete / High`.

## User Or Contract-Visible Problem

Historical evidence found that V3 still serializes combat preview caches, skipped/defaulted preview fields can change
during deserialization, and the world cache fast path plus enemy spawn setup can trust that cache. A presentation
cache saved before a later battle can therefore survive durable Continue and influence authoritative setup instead
of being recomputed from the node and live policy.

## Dependency And Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 06 registry row.
- Direct predecessor: Phase 05 CRA-P04-C01, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_05_p04_c01/COMPLETION_AUDIT.md`.
- Historical evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_05_combat_preview_setup_and_boss_publication/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_05_combat_preview_setup_and_boss_publication/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_05_combat_preview_setup_and_boss_publication/BOUNDARY_AUDIT.md`
- Final execution evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/APPROVED_POLICY_DECISIONS.md`

Historical findings define the defect and required durable sequence. Current runtime/serialization paths determine
the exact seam after required-document initialization.

## Priority And Current Disposition

- Priority axes: `3 / 3 / 3 / 3 / 3 / 3 / unknown`; known subtotal `18/18`.
- Final disposition at activation: `still_valid`, correctness, exact durable-Continue proof first.
- Current live-tree disposition: discovery pending until Child initialization/read and scoped dirty inspection.

## Current And Target Authority

- Historically, manifested node difficulty/live policy owns internal scale, but a serialized preview cache is still
  reachable from the world resolver and enemy setup.
- Target authority: preview remains a presentation/cache carrier only. Battle setup uses the existing resolved setup
  authority from stable run node identity and live data, recomputing derived preview inputs after load.
- Exact V3/cache fields, mutation/default behavior, cache producer/consumer, setup resolver and server persistence
  seams must be rediscovered and recorded before mutation.

## In Scope

- discover current preview cache ownership and every battle-start read, including normal/Boss/Event/Monolith paths;
- capture expanded status, scoped diffs and hashes for every prospective source/test/canonical target;
- add an exact durable sequence that generates a preview, cancels it, reaches SavePoint, exports/persists, mutates
  only serialized cache-derived setup values, Continues and starts the target battle;
- compare authoritative setup/enemy values against an unmodified control and current live policy;
- remove cache authority at the narrowest existing world/setup resolver without changing public/serialized shape;
- retain C02 full publication feasibility and C04 `ResolvedCombatSetupPlan` ownership/parity.

## Out Of Scope

- removing, renaming or versioning V3 preview fields or public preview helpers;
- changing difficulty formulas, enemy balance, node generation, reward/scenario/Boss setup or battle lifecycle;
- redesigning combat preview generation/resolver families assigned to later CRA-P05-C03/C05;
- changing active-run envelope, SavePoint durability semantics, DTO/wire/nesting or Unity behavior;
- adding cache repair/migration/fallback or trusting a client/server preview as gameplay input;
- unrelated cleanup or dirty-worktree normalization.

## Preserved Invariants And Policy

1. Authoritative battle setup is derived from current run node/live data, never serialized presentation cache values.
2. Mutating only a structurally valid persisted preview cache cannot change setup, enemies, rewards or scenario.
3. Preview, cancel, SavePoint, export/Continue and start ordering/identity remain deterministic and atomic.
4. Ordinary, Boss, Event-started and Monolith setup continue through the existing typed resolved plan.
5. Current V3 checkpoint, RON, command/result, DTO/wire and public Rust shapes remain exact under CRA-POL-006.
6. Invalid checkpoint graphs continue to fail before mutation; no repair or old-version migration is added.
7. Pre-existing user changes are preserved without reset, restore, clean, stash or broad formatting.

Direct phase policy: CRA-POL-006. Open policy decisions: none at activation. If correctness requires deleting or
reinterpreting a serialized/public field, stop; the approved correction must preserve it while removing authority.

## Implementation And Dirty-Worktree Strategy

After the three Child documents exist and Master records sole activation, read them completely. Then capture the
expanded status and prospective target status/diff/hash before current source/test/canonical inspection. Treat all
existing changes as user-owned and edit only the preview-authority proof, resolver seam and nearest canonical.

Use test-first proof in this order:

1. Run current preview/setup/checkpoint tests with nonzero counts.
2. Trace one production preview/cancel/SavePoint/export/Continue/start sequence and identify a cache field currently
   accepted by V3 but capable of changing setup.
3. Add an unmodified control and mutated-cache case using real serialization/durable boundaries.
4. Demonstrate current battle setup differs only because it consumed the mutated cache.
5. Change the narrowest battle-start resolver to recompute from authoritative node/live inputs.
6. Re-run exact sequence, preview/setup families, V3/active-run boundaries and owning package.

## Migration, Deployment And Compatibility

- V3/checkpoint: field set and version remain unchanged; cache is retained but non-authoritative for setup.
- Live RON/data: no schema/content/balance change.
- DTO/wire/public Rust: preserve preview and battle setup/update/resync shapes and helper availability.
- Active-run/server: existing payload/envelope and SavePoint publish order remain unchanged.
- Deployment: no migration; current V3 payloads recompute setup at start.

## Validation Plan

- Baseline: current combat preview, resolved setup, checkpoint round-trip and active-run tests with nonzero counts.
- First proof: exact durable preview→cancel→SavePoint→Continue→start control/mutated-cache comparison.
- Focused final: same sequence plus ordinary/Boss/Event/Monolith resolver/setup parity and enemy stat assertions.
- Broad: combat preview/setup/world/checkpoint families, live RON integration, Core package, server active-run boundary
  where current topology makes the durable host consumer relevant.
- Static: cache field/read/write/caller inventory, V3 exact field audit, legacy/fallback search, scoped status/diff/
  hashes, canonical comparison, formatting and `git diff --check`.
- Zero-test filters do not count as proof.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P06-C01 | current preview/cache/checkpoint/setup authorities identified | full producer/consumer path inventory | complete |
| P06-C02 | dirty targets and concurrent work preserved | pre/post status, diff and hashes | complete |
| P06-C03 | durable mutated-cache setup defect reproduced | exact test-first control/mutation sequence | complete |
| P06-C04 | battle start recomputes authoritative setup after Continue | corrected setup/enemy equality proof | complete |
| P06-C05 | ordinary/Boss/Event/Monolith and deterministic setup remain exact | resolver/setup parity regressions | complete |
| P06-C06 | V3/RON/DTO/wire/public compatibility is unchanged | CRA-POL-006 and static field/consumer audit | complete |
| P06-C07 | focused and affected broad validation passes | nonzero runner counts and static checks | complete |
| P06-C08 | phase-exit review is Complete / High | `COMPLETION_AUDIT.md` | complete |

## Implemented Result

- `explicit_combat_preview_for_node` no longer returns `RunState::combat_previews` at battle start. It always
  regenerates the deterministic setup input from current node identity, encounter, manifested difficulty, live data
  and node seed before the existing `ResolvedCombatSetupPlan` path runs.
- Node-confirm preview publication, cache retention, V3 `combat_previews`, skipped internal scale, public Rust/DTO/
  wire fields and active-session commit shape are unchanged.
- The exact regression uses a real preview, cancellation, completed SavePoint, V3 export, valid serialized cache
  mutation, fresh-core Continue and actual battle start. Both unmodified and mutated payloads produce the same
  authoritative Hard setup/enemy after correction.
- Event-started retry independently corrupts its presentation cache and proves the authored encounter is regenerated;
  existing Boss, Monolith, preview, setup, checkpoint and live-data guards remain green.

## Stop Conditions

Stop and do not activate Phase 07 if current battle start already ignores serialized preview caches; the durable
sequence cannot isolate cache authority; correction requires V3/public/wire removal or balance policy; cache
mutation should instead be invalid under Phase 02's approved graph; dirty overlap is inseparable; a focused runner
selects zero tests; or completion review is not `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
