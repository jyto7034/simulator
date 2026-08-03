# Documentation–Code Alignment Corrections Master Notes

Status: complete

Document class: non-canonical orchestration working notes

Started: 2026-07-18

## Current Position

- Master complete after independent combined review.
- G01 complete: demonstrated nested gameplay RON paths now reject unknown fields through actual
  outer source parsing.
- G02 complete: setup catalog refs now contain only non-empty, catalog-resolved actual abnormality
  IDs; the registered live smoke observed an empty list for a corroded-only encounter.
- G03 complete: a direct named regression protects the final Endless response `RunComplete` branch
  and proves planning-return delivery remains pending.
- G06 complete: GrenadeLauncher is removed from current schema/contracts and remains only in
  fail-closed evidence, historical records and explicit future concepts.
- G04 complete: all 29 live weapons explicitly own block capacity, unarmed is zero, and armor/
  employee profile do not contribute.
- G05 complete: formal/current/future/placeholder and recipe-gated Maintenance wording is canonical
  without migrating temporary content.
- G07 complete: source count, current-domain terminology and external enum inventories match the
  final loader/Core shape.
- No live RON, durable canonical or external Unity artifact has been changed by G02.
- Combined Core/server/Unity validation is green; the independent review verdict is Complete / High.

## Policy Decision Register

| ID | Decision | Status |
| --- | --- | --- |
| P01 | Base block capacity is explicitly authored by equipped weapon RON only. | accepted |
| P02 | Formal fragments remain active-skill-centered; current BasicAttackModifier entries are placeholders. | accepted |
| P03 | Formal Boss/Elite/special abnormalities require authored range; current generic entries are placeholders. | accepted |
| P04 | WhiteNight detailed mechanics are future/undecided and not current implementation scope. | accepted |
| P05 | Maintenance equipment operations require explicit per-item recipe; no fallback recipe. | accepted |
| P06 | Remove GrenadeLauncher from current schema and retain only a future concept note. | accepted |

## P01 Authoring Detail

- Every current weapon entry must own an explicit `block_capacity`; serde/runtime archetype defaults are
  not permitted.
- Default content review guideline:
  - Shield 3
  - Sword/Spear 1
  - Bow/Gun/Staff 0
  - no weapon 0
- Armor and employee CombatAptitude do not contribute.
- “각 무기 RON에서 정한다” means individual values are visible and reviewable in live data. It does
  not authorize arbitrary per-weapon deviations without content rationale.

## Placeholder Interpretation

- Placeholder data remains loadable/current behavior until its real content goal replaces it.
- “Placeholder” does not mean tests may ignore schema integrity or that runtime may silently invent
  values.
- Formal policy docs must not claim placeholder mechanics are completed design.
- Live index/evidence docs may describe exact placeholder reality as long as they label it current
  derived data rather than policy.

## Finding-To-Child Coverage

| Child | Findings | Nature |
| --- | --- | --- |
| G01 | DCA2-DATA-002 | Core data-schema correction |
| G02 | DCA2-WIRE-003 | Core setup projection correction |
| G03 | DCA2-TEST-001 | focused evidence correction |
| G04 | DCA2-GAME-001 | gameplay/data/runtime/wire implementation |
| G05 | DCA2-GAME-002, DCA2-BATTLE-001, DCA2-CONTENT-001, DCA2-GAME-003 | docs-only approved disposition |
| G06 | DCA2-BATTLE-002 | current schema removal + future note |
| G07 | DCA2-DATA-001, DCA2-DOC-001, DCA2-WIRE-002 | current canonical cleanup |

Coverage check: 12 included findings, 12 assigned exactly once.

## Exclusion Guard

- `DCA2-WIRE-001` must not be “fixed” by weakening/removing required event presentation wording.
- `DCA2-DOC-002` must not be “fixed” by deleting `/mnt/f/unity projects/ark/docs - 복사본` without
  exact user authorization.
- Child tests may expose these findings, but they remain outside the mutation scope.

## Cross-Child Overlap Risks

- G04, G05 and G07 can all touch `game_rulebook.md`; execute sequentially and re-read current diff
  before each patch.
- G06 and G07 can both touch external `unity_core_contract.md`; G07 must use the post-G06 enum set.
- G01 may touch shared types used by G04 weapon data. Complete/rebase strictness before adding the
  required block field so negative tests describe the final schema.
- G02 may update setup tests also used by the registered live smoke probe; preserve message ordering
  and do not expand into P1 presentation.

## Child Initialization Rule

When a child starts:

1. Confirm its planned directory does not already exist or inspect existing content if it does.
2. Read the exact current runtime/data/docs/tests named by its finding.
3. Record current dirty diffs for touched files.
4. Create that child’s `PLAN.md`, `EXPERIMENTS.md`, and `EXPERIMENT_NOTES.md`.
5. Copy only the relevant accepted policy from this Master; do not copy unrelated child scopes.
6. Run focused baseline before mutation.

## Validation Interpretation

- A green full suite does not prove nested unknown-field rejection, catalog membership cleanliness or
  placeholder/current wording without their exact assertions/searches.
- G03 is deliberately a test-only correction because runtime tracing already found the branch
  aligned.
- G05/G07 are deliberately docs-only. If their work reveals behavior does not match the accepted
  wording, stop rather than editing the wording to hide implementation drift.

## Open Policy Questions

None required to initialize G01–G07.

Potential child stop questions:

- weapon-specific exceptions to P01 defaults;
- deployed compatibility requirement for GrenadeLauncher;
- a supposedly placeholder mechanic that is actually shipped formal content;
- an existing nested field whose intended replacement is ambiguous.

## Completion Review Checklist

- [x] G01 complete and independently rechecked.
- [x] G02 complete and independently rechecked.
- [x] G03 complete and independently rechecked.
- [x] G06 complete before final G04/G07 enum inventory closure.
- [x] G04 complete and weapon values reviewed from live RON.
- [x] G05 complete without content implementation.
- [x] G07 complete against final current code/data.
- [x] Included finding coverage remains exactly 12/12.
- [x] Excluded findings remain untouched and explicitly reported.
- [x] Combined validation and completion review recorded.

## Final Validation Snapshot

- Core all-targets: 786 passed, 0 failed/ignored.
- game_server check: passed; tests: 30 passed, 0 failed/ignored.
- Unity EditMode: 241 passed, 0 failed/skipped.
- Rust formatting, Core/scoped external diff checks and exact legacy searches passed.
- Completion review: [COMPLETION_REVIEW.md](COMPLETION_REVIEW.md), verdict Complete / High.
