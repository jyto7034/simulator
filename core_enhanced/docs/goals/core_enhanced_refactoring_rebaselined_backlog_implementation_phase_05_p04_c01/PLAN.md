# Phase 05 Goal — CRA-P04-C01 VN Repeat/Source Publication Matrix

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P04-C01
Master Phase: 05 of 44
Work Type: correctness correction
Production Mutation Authorized: yes — the user's 2026-08-09 Master execution request remains active; authorization
is limited to this Phase after Phase 04's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-09 (`Asia/Seoul`)

## Objective And Completion Conditions

Move every currently runtime-rejected Visual Novel trigger source/repeat-policy pairing into strict typed publication
validation. Programmatic metadata and the official embedded bundle path must reject the same incompatible pairs with
exact trigger ID, field path and cause before any database becomes usable, while all currently executable pairings
and the live bundle continue to publish unchanged.

This Phase is complete only when the current typed trigger schema, loader/publication validator, runtime match and
consumed-key derivation matrix are identified; every runtime-rejected pair is captured with structured test-first
negative evidence through both typed and official-bundle publication where feasible; the smallest data-owned matrix
is implemented without weakening runtime defense; accepted pairings and the live bundle remain valid; no RON,
checkpoint, DTO, wire or public API shape changes; focused/broad validation passes; and a phase-exit audit returns
`Complete / High`.

## User Or Contract-Visible Problem

Historical evidence found that trigger publication validates local fields and references but not whether the repeat
policy has the identity required by its source. At least `MapNodeEntered + OncePerBattle` publishes successfully,
then runtime `consumed_key_for_trigger` fails because no battle UUID exists. The surrounding command rolls back, but
production data that can never execute has already passed startup validation.

## Dependency And Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 05 registry row.
- Direct predecessor: Phase 04 CRA-P03-C02, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_04_p03_c02/COMPLETION_AUDIT.md`.
- Historical evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_04_interactive_node_and_visual_novel/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_04_interactive_node_and_visual_novel/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_04_interactive_node_and_visual_novel/BOUNDARY_AUDIT.md`
- Final execution evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/APPROVED_POLICY_DECISIONS.md`

Historical findings define the defect class and minimum example. Current source/data determine the exact matrix and
implementation seam after required-document initialization.

## Priority And Current Disposition

- Priority axes: `1 / 2 / 1 / 1 / 2 / 2 / unknown`; known subtotal `9/18`.
- Final disposition at activation: `still_valid`, correctness, structured negatives first.
- Current live-tree disposition: discovery pending until Child initialization/read and scoped dirty inspection.

## Current And Target Authority

- Historically, typed publication validates trigger-local values and references, while runtime key derivation owns
  the actual source/repeat identity requirements.
- Target authority: typed trigger publication owns the accepted compatibility matrix and runtime retains defensive
  consumed-key validation for impossible bypassed/programmatic state.
- Exact source/repeat variants, paths, error types, loader layers, runtime identity inputs and live usages must be
  rediscovered and recorded before mutation.

## In Scope

- discover current trigger types, publication/official-bundle validation, runtime matching and consumed-key paths;
- capture expanded status, scoped diffs and hashes for every prospective source/test/live-data/canonical target;
- enumerate the complete current source × repeat-policy matrix from runtime identity availability;
- add exact structured negatives for every runtime-rejected pair, including programmatic typed validation and the
  official embedded-bundle route where the current harness supports an isolated invalid fixture;
- preserve runtime defensive rejection, live accepted pairs, deterministic trigger ordering and consumed history;
- update the nearest canonical only after implemented publication behavior is real.

## Out Of Scope

- inventing new trigger sources, repeat policies, identity fallback, retry/idempotence or authored expansion;
- changing currently runtime-accepted battle `OncePerNode` meaning without a new policy decision;
- changing trigger matching, priority, queue/session assembly, persisted/active commits or checkpoint history;
- changing live content merely to avoid validation or adding permissive normalization/compatibility branches;
- changing RON/schema/checkpoint/DTO/wire/public Rust shape or external Unity behavior;
- implementing adjacent CRA-P04-C03, CRA-P04-C04 or CRA-F-C02 work.

## Preserved Invariants And Policy

1. Publication rejects exactly the pairings runtime cannot key; it does not narrow currently executable pairings.
2. Every rejection carries exact object/trigger ID, field path and stable cause through the existing error contract.
3. Live embedded data remains valid as one whole fail-fast bundle; no silent normalization or partial publication.
4. Runtime keeps defensive key derivation and at-most-once consumed-history behavior.
5. Trigger priority/order, blocking, queue/non-interruption and checkpoint rewind/persist semantics are unchanged.
6. CRA-POL-006 applies: current RON, checkpoint, DTO/wire and public API compatibility remain exact.
7. Pre-existing user changes are preserved without reset, restore, clean, stash or broad formatting.

Direct phase policy: CRA-POL-006. Open policy decisions: none at activation. If the current runtime accepts a pairing
whose future meaning is questionable, preserve it; expanding or narrowing that meaning requires explicit policy.

## Implementation And Dirty-Worktree Strategy

After the three Child documents exist and Master records sole activation, read them completely. Then capture the
expanded status and exact target status/diff/hash before current source/data/test inspection. Treat all existing
changes as user-owned and edit only the trigger compatibility proof, data-owned validation and canonical seam.

Use test-first proof in this order:

1. Run current VN trigger/data/live tests with nonzero counts.
2. Enumerate the runtime's actual identity requirements for every source/repeat pair.
3. Add one table-driven typed negative matrix with exact provenance and demonstrate current fail-open publication.
4. Exercise the official bundle loader/publication boundary with isolated invalid pairs without mutating live data.
5. Add the same compatibility predicate/matrix to the existing typed publication authority.
6. Re-run all accepted/live behavior, runtime defense, loader and owning-package validation.

## Migration, Deployment And Compatibility

- Live RON/data: no schema or intended content change; all current authored rows must remain valid.
- Save/checkpoint: consumed history and runtime identity fields remain unchanged.
- DTO/wire/public Rust API: preserve trigger/session internals and existing Unity-facing VN projection.
- Deployment: invalid previously accepted authoring fails startup publication instead of failing only when triggered.
- Temporary compatibility: none; strict whole-bundle publication is the approved default.

## Validation Plan

- Baseline: current visual-novel typed validation and live RON loader tests with actual nonzero counts.
- First proof: every runtime-rejected pair succeeds publication before correction, with runtime failure evidence where
  an existing public path can exercise it without unrelated setup ambiguity.
- Focused final: table-driven typed/programmatic negatives with exact object/path/cause, official-bundle negatives,
  accepted matrix positives and runtime defensive tests.
- Broad: full VN/data module, live RON integration, owning Core package and affected server compile boundary.
- Static: source/repeat caller inventory, live trigger search, legacy/fallback search, scoped status/diff/hashes,
  canonical comparison, formatting and `git diff --check`.
- Zero-test filters do not count as proof.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P05-C01 | current publication/runtime/loader authorities and exact pair matrix identified | call/data/identity inventory | complete |
| P05-C02 | dirty targets and concurrent work preserved | pre/post status, diff and hashes | complete |
| P05-C03 | every runtime-rejected pair reproduced as fail-open publication | structured test-first matrix | complete |
| P05-C04 | typed and official publication reject with exact provenance | negative matrix across both boundaries | complete |
| P05-C05 | accepted/live/runtime defensive semantics remain exact | positive matrix, live bundle and runtime tests | complete |
| P05-C06 | RON/checkpoint/DTO/wire/public compatibility is unchanged | CRA-POL-006 and static contract audit | complete |
| P05-C07 | focused and affected broad validation passes | nonzero runner counts and static checks | complete |
| P05-C08 | phase-exit review is Complete / High | `COMPLETION_AUDIT.md` | complete |

## Stop Conditions

Stop and do not activate Phase 06 if current publication already mirrors all runtime rejections; the exact current
matrix cannot be derived from runtime identity requirements; a requested pairing needs new gameplay/idempotence
meaning; exact provenance requires public error/schema change; a live row becomes invalid without an approved
correction; dirty overlap is inseparable; a focused runner selects zero tests; or completion review is not
`Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
