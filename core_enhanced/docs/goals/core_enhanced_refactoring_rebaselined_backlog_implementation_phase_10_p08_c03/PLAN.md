# Phase 10 Goal — CRA-P08-C03 Preserve Declared Stat Ranges With Saturating Percentage Arithmetic

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P08-C03
Master Phase: 10 of 44
Work Type: correctness correction
Production Mutation Authorized: yes — the user's 2026-08-09 Master execution request remains active; authorization
is limited to this Phase after Phase 09's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Make every scoped stat-percentage calculation total over its declared typed input domain. AttackIntervalMs modifier
arithmetic, shared percentage-ceil calculation and draft Attack/MaxHealth scale must use range-preserving wide
intermediates and apply the destination stat's existing floor/cap exactly once, without narrowing wrap or debug-only
overflow.

This Phase is complete only when the complete current percentage-arithmetic call graph is identified; boundary tests
first reproduce the interval, helper and draft-scale defects; the smallest common typed arithmetic boundary removes
unchecked casts/multiplication and duplicated unsafe clamps; 0, 1, integer maxima, `u64::MAX` interval and extreme
signed percentages have identical debug/release results; the authored 207% draft stage respects Attack/MaxHealth
ranges and derives current HP by the existing rule; ordinary/live values, modifier and stat-pipeline order, rounding,
defense sign and certified-power behavior remain exact; no public/data/save/wire shape changes; focused/broad
validation passes; and the phase-exit audit returns `Complete / High`.

## User Or Contract-Visible Problem

Historical evidence found three valid-input arithmetic holes. AttackIntervalMs percentage modification narrows its
`u64` base to `i64` and multiplies without a wide saturating intermediate. A percentage-ceil helper computes a wide
quotient and then narrows a value above `u32::MAX` with `as u32`. Draft scaling assigns the result to Attack and
MaxHealth without reapplying the stats' declared range, so large values can wrap and MaxHealth can escape its
existing cap. Current authored values are modest, but typed inputs and public constructors do not enforce those
implicit bounds.

## Dependency And Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 10 registry row.
- Direct predecessor: Phase 09 CRA-P07-C02, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_09_p07_c02/COMPLETION_AUDIT.md`.
- Historical evidence:
  - `../core_enhanced_refactoring_audit_phase_08_battle_effects_and_stats/CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_audit_phase_08_battle_effects_and_stats/TEST_AUDIT.md`
  - `../core_enhanced_refactoring_audit_phase_08_battle_effects_and_stats/BOUNDARY_AUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_08_battle_effects_stats_and_boss_leaf_seams/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_08_battle_effects_stats_and_boss_leaf_seams/TEST_REAUDIT.md`
- Final execution evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/REBASELINED_CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/IMPLEMENTATION_WAVES_AND_GATES.md`

No candidate-specific policy ID applies: saturation restores already-declared numeric domains. CRA-POL-006 still
requires exact public/RON/save/checkpoint/wire compatibility. This Phase deliberately precedes Phase 12 CRA-P08-C02
so numeric cap/current-HP behavior is frozen before HP/death-aware runtime stat transitions.

## Priority And Current Disposition

- Priority axes: `1 / 2 / 1 / 2 / 2 / 2 / unknown`; known subtotal `10/18`.
- Final disposition at activation: `still_valid`, correctness correction, boundary tests first.
- Current live-tree disposition: implemented and validated; independent completion audit pending.

## Current And Target Authority

- Historical current authority: `UnitStats`, a shared employee percentage helper and the battle-entry stat pipeline
  each own part of percentage arithmetic, with narrowing or destination-cap gaps at extreme valid inputs.
- Target authority: typed wide-intermediate signed/unsigned percentage operations preserve existing truncation/ceil
  rules, while each destination stat applies its declared minimum/maximum once before publication.
- Current function/type names, complete callers, existing helpers and exact destination ranges must be rediscovered
  after the required document gate; historical names are evidence, not mutation instructions.

## In Scope

- identify every current stat percentage producer/consumer involved in AttackIntervalMs modifiers, shared
  percentage-ceil calculation, draft Attack/MaxHealth scale and current-HP derivation;
- capture expanded dirty status, scoped diffs and hashes for every prospective source/test/canonical target;
- add a table-driven boundary safety net covering 0, 1, relevant integer maxima, `u64::MAX` interval and extreme
  positive/negative signed percentage modifiers;
- add direct draft-scale cases using the published 207% stage at high Attack/MaxHealth and assert exact saturation,
  MaxHealth cap and current-health derivation;
- prove focused arithmetic has the same result and no panic/wrap in debug and release profiles;
- replace only the unsafe narrowing/multiplication and duplicated range clamp required by the tests;
- retain or strengthen exact ordinary/live-value goldens and update the nearest canonical only after runtime behavior
  is implemented.

## Out Of Scope

- changing authored balance percentages, encounter stage scales, stat application order or rounding policy;
- changing certified-power inputs/exclusions, consumable/condition semantics or persistent-to-runtime ownership;
- implementing Phase 11 trigger occurrence dispatch or Phase 12 HP/death-aware runtime stat mutation;
- changing positive MaxHealth healing, lethal modifier attribution, kill credit or event/death policy;
- broad numeric utility consolidation unrelated to the three proven holes;
- changing RON/data schema, public stat/DTO types, save/checkpoint version, result/wire shape or server/Unity behavior;
- unrelated refactor, formatting or dirty-worktree cleanup.

## Preserved Invariants

1. Ordinary and all current live authored stat outputs remain exactly equal.
2. Modifier and battle-entry pipeline order remains exact.
3. Existing signed percentage truncation and unsigned scale-ceil rules remain exact.
4. AttackIntervalMs remains at least 1 and accepts the full declared `u64` base domain without narrowing.
5. Attack and MaxHealth remain inside their existing destination domains; MaxHealth keeps its existing cap.
6. Draft current health is derived from the capped final MaxHealth by the existing condition rule and never exceeds
   final MaxHealth.
7. Defense sign/application, movement and all non-scoped stat behavior remain unchanged.
8. No gameplay, publication, data, public Rust, checkpoint/save, DTO/wire or external compatibility policy changes.
9. Pre-existing user changes are preserved without reset, restore, clean, stash or broad formatting.

## Implementation And Dirty-Worktree Strategy

After the three Child documents exist and Master records sole activation, read them completely. Then capture the
expanded dirty status and prospective target status/diff/hash before inspecting current source/tests/canonical.
Treat all existing changes as user-owned and edit only the narrow arithmetic owners, exact boundary tests and nearest
canonical.

Use test-first proof in this order:

1. Run nonzero UnitStats, stat-pipeline, employee/live-stat and draft/battle-build baselines.
2. Trace all scoped call sites, result types, rounding rules, floors/caps and current-health derivation.
3. Add the extreme AttackIntervalMs modifier table and establish debug/release pre-fix behavior.
4. Add direct percentage-ceil maximum and 207% high draft-scale cases; require declared saturation/cap, not wrapping.
5. Introduce/reuse the smallest typed wide arithmetic helpers and apply destination ranges at the existing owners.
6. Re-run exact boundary tables, ordinary goldens, battle setup/live data and complete affected suites.

## Migration, Deployment And Compatibility

- No data, schema, save/checkpoint, DTO/wire, API or version migration.
- Extreme valid values that previously panicked or wrapped now saturate to the already-declared stat domain.
- Current authored/live values and serialized shapes remain identical, so server/Unity require no rollout.
- If completion requires a balance/rounding change, wider public type or HP/death policy, stop for user direction
  rather than expanding this Phase.

## Validation Plan

- Baseline: nonzero UnitStats arithmetic, employee/stat-pipeline, battle draft/build and live-data families.
- First proof: table at 0/1/type maxima, `u64::MAX` AttackIntervalMs, extreme signed percent and high percentage-ceil;
  high Attack/MaxHealth at the authored 207% stage.
- Focused final: exact interval floor/saturation, signed truncation, unsigned ceil, Attack/MaxHealth cap and current-HP
  result in both debug and release profiles.
- Preservation: ordinary modifier, movement/defense, stat pipeline order, certified power and current live-value
  goldens.
- Broad: affected battle setup/world integration, live RON/data validation and complete Core; server all-target if a
  shared/public carrier is touched.
- Static: complete helper/caller inventory, unchecked cast/multiplication search, public/data shape audit, scoped
  status/diff/hashes, canonical comparison, formatting and `git diff --check`.
- Zero-test filters do not count as proof.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P10-C01 | complete scoped arithmetic authority/caller graph identified | function/type/caller/range/rounding inventory | completed |
| P10-C02 | dirty targets and concurrent work preserved | pre/post status, diff and hashes | completed |
| P10-C03 | extreme interval/helper/draft defects reproduced test-first | debug/release boundary red evidence | completed |
| P10-C04 | typed wide arithmetic preserves declared destination ranges | implementation and static caller proof | completed |
| P10-C05 | ordinary/live/order/rounding/current-HP invariants remain exact | focused preservation matrix | completed |
| P10-C06 | focused and affected broad validation passes | nonzero counts and static checks | completed |
| P10-C07 | canonical and compatibility boundary are current | contract diff and consumer audit | completed |
| P10-C08 | phase-exit review is Complete / High | `COMPLETION_AUDIT.md` | completed — Complete / High |

## Stop Conditions

Stop and do not activate Phase 11 if current code already proves total range-preserving arithmetic for every named
branch; declared destination caps or rounding rules are ambiguous; tests cannot distinguish debug panic/release wrap
from correct saturation; a balance percentage, HP/death policy or public/data/save shape must change; dirty overlap is
inseparable; a focused runner selects zero tests; or completion review is not `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.

## Completion Result

- Phase exit audit: `Complete / High`.
- Improvement class: `none`.
- Full-domain signed interval, unsigned ceil, authored draft caps and exact post-scale HP ratio now saturate without
  debug/release divergence while preserving live values, order and all public/data/save/wire shapes.
- Complete Core library validation: 958 passed, 0 failed; all live integrations and game_server all-target compile
  pass; no migration or rollout action is required.
- Master may mark Phase 10 complete and make Phase 11 ready.
