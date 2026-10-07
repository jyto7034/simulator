# Phase 08 Goal — CRA-P07-C01 Derive Tactical Tile Effects Once From Full Final State

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P07-C01
Master Phase: 08 of 44
Work Type: correctness correction
Production Mutation Authorized: yes — the user's 2026-08-09 Master execution request remains active; authorization
is limited to this Phase after Phase 07's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-09 (`Asia/Seoul`)

## Objective And Completion Conditions

Make one final-state tactical-effect reconciliation own Focus, effective block and Platform Rest state after the
logical activity, stance, tile and combat profile are complete. Ground↔Platform relocation and swap with an
`Any`-profile field worker must never retain or duplicate a tile effect that its final aggregate forbids.

This Phase is complete only when current stance/position/field-work/tactical-snapshot and Rest-event authorities are
identified; cross-kind worker tests reproduce the split transition; one typed reconciliation applies the exact
effect delta from old to final state; ready/working, Attack/Defense, Ground/Platform, relocation/swap and stale Rest
semantics remain exact; unrelated modifiers, resource/cooldown/progress and public shape are preserved; focused and
broad validation pass; and the phase-exit audit returns `Complete / High`.

## User Or Contract-Visible Problem

Historical evidence found that relocation/swap changes tile Focus, block and Rest scheduling before field-work
occupancy reconciliation. A worker may therefore have final `FieldWorking` activity while retaining an incompatible
Ground Focus or Platform Rest, or may receive duplicate Focus arithmetic when it exits work across tile kinds.
Existing Ground-only tests cannot expose the full aggregate mismatch.

## Dependency And Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 08 registry row.
- Direct predecessor: Phase 07 CRA-P06-C05, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_07_p06_c05/COMPLETION_AUDIT.md`.
- Historical evidence:
  - `../core_enhanced_refactoring_audit_phase_07_battle_spatial_and_tactical/CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_07_battle_spatial_tactical_and_boss_movement/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_07_battle_spatial_tactical_and_boss_movement/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_07_battle_spatial_tactical_and_boss_movement/BOUNDARY_AUDIT.md`
- Final execution evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/APPROVED_POLICY_DECISIONS.md`

No candidate-specific policy ID applies. CRA-POL-006 applies as the cross-cutting compatibility boundary: internal
final-state consolidation must preserve current RON, DTO/wire/public Rust and checkpoint shape. This correction
restores the existing tactical contract and does not authorize new tactical balance or eligibility policy.

## Priority And Current Disposition

- Priority axes: `3 / 2 / 3 / 3 / 3 / 2 / unknown`; known subtotal `16/18`.
- Final disposition at activation: `still_valid`, correctness correction, failing Any/worker transitions first.
- Current live-tree disposition: discovery pending until Child initialization/read and scoped dirty inspection.

## Current And Target Authority

- Historical current authority: stance, relocation/swap position update and field-work transition independently
  mutate Focus, block and Rest; snapshot derivation separately describes the intended final aggregate.
- Target authority: one typed tactical-effect reconciliation compares old and fully resolved final
  `(activity, stance, tile kind, combat profile)` and applies exactly one Focus delta, exact block and one valid Rest
  epoch/deadline.
- Current function/type names and call ordering must be rediscovered after the required document gate; historical
  names are evidence, not mutation instructions.

## In Scope

- identify all producers and consumers of tactical activity, stance, tile kind, Focus modifier, effective block,
  Rest epoch/deadline/event and tactical snapshot;
- capture expanded dirty status, scoped diffs and hashes for every prospective source/test/canonical target;
- add `Any`-profile Ground↔Platform relocation and swap cases where a worker exits assignment and where the same
  worker remains assigned across the final occupancy;
- assert one final activity/stance/tile/focus/block/rest aggregate, exact unrelated modifier preservation and stale
  Rest-event no-op behavior;
- consolidate only the duplicated tile-effect transition logic and invoke it after logical final state is known;
- preserve ready Defense Ground/Platform, Attack, FieldWorking and same-tile facing/cadence semantics;
- update the nearest canonical only after runtime behavior is real.

## Out Of Scope

- Phase 09 CRA-P07-C02 atomic world tactical mutation/publication work;
- Direct/Rapier backend consolidation, movement tick cleanup or Boss goal/path ownership;
- changing field-work assignment rules, relocation/swap availability, resource cost, cooldown, progress or targeting;
- adding a live `Any` weapon, changing weapon/profile balance or authoring validation;
- changing damage formula semantics or unrelated modifier representation;
- changing RON, DTO/wire/public Rust, server/Unity or checkpoint/save shape;
- unrelated refactor, formatting or dirty-worktree cleanup.

## Preserved Invariants

1. Attack has no Ground Focus, bonus block or Platform Rest tile effect.
2. `FieldWorking` has no Ground Focus, block or Platform Rest regardless of its occupied tile.
3. Ready+Defense Ground applies the authored base+bonus block and Ground Focus exactly once.
4. Ready+Defense Platform has zero block and exactly one full-interval Rest cadence.
5. Same-tile facing-only mutation preserves the current Rest cadence; real effect changes invalidate stale events.
6. Relocation/swap while working, final occupancy, progress, cost, cooldown, revision and failure semantics remain
   unchanged within this Phase.
7. Unrelated incoming modifiers and their aggregate identity/value remain exact.
8. RON, policy, checkpoint, DTO/wire/public Rust and external consumer shapes remain unchanged under CRA-POL-006.
9. Pre-existing user changes are preserved without reset, restore, clean, stash or broad formatting.

## Implementation And Dirty-Worktree Strategy

After the three Child documents exist and Master records sole activation, read them completely. Then capture the
expanded dirty status and prospective target status/diff/hash before inspecting current source/tests/canonical.
Treat all existing changes as user-owned and edit only the exact final-state reconciliation, its safety net and the
nearest canonical.

Use test-first proof in this order:

1. Run current tactical stance/relocation/swap, field-work and world tactical families with nonzero counts.
2. Trace every Focus/block/Rest mutation and snapshot derivation, including Rest event epoch/deadline admission.
3. Add Any-profile worker Ground↔Platform relocation and swap matrices for exit and remain-assigned final states.
4. Demonstrate a full final-state mismatch while asserting unrelated modifier preservation.
5. Introduce the smallest typed old→final tactical-effect reconciliation and call it only after logical state is
   complete; remove only superseded manual tile-effect arithmetic/scheduling.
6. Re-run exact cases, stale Rest events, ready relocation/swap, Ground-only progress and owning-package suites.

## Migration, Deployment And Compatibility

- No data, schema, save/checkpoint, DTO/wire or version migration.
- No live balance change and no new authoring capability; typed `Any` profiles already exist and are valid test input.
- Server/Unity continue to consume the same tactical snapshot fields and require no rollout change.
- If the correction requires a public shape, new policy, live-data normalization or compatibility path, stop for
  user direction rather than expanding this Phase.

## Validation Plan

- Baseline: tactical stance/relocation/swap Core families, field-work family and live world tactical commands with
  nonzero counts.
- First proof: Any worker Ground↔Platform relocation and swap, covering exit/remain assignment and full aggregate.
- Focused final: activity, stance, Focus, exact block, Rest epoch/deadline, stale event, unrelated modifier,
  same-tile cadence, cost/cooldown/progress and snapshot projection.
- Broad: affected movement/tactical/field-work/world families, live RON, complete Core package and server compile if
  the shared projection boundary is affected.
- Static: complete Focus/block/Rest caller/mutator inventory, removed duplicate transition paths, policy/data/public
  shape audit, scoped status/diff/hashes, canonical comparison, formatting and `git diff --check`.
- Zero-test filters do not count as proof.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P08-C01 | tactical effect producers/consumers and final-state seam identified | complete call/type/state/event inventory | completed |
| P08-C02 | dirty targets and concurrent work preserved | pre/post status, diff and hashes | completed |
| P08-C03 | Any worker cross-kind paths reproduce split final aggregate | relocation/swap exit/remain matrix | completed |
| P08-C04 | one final-state reconciliation owns Focus/block/Rest | exact old→final effect evidence | completed |
| P08-C05 | Attack/Defense/Ready/FieldWorking/cadence behavior is exact | focused invariant matrix | completed |
| P08-C06 | costs/progress/modifiers and compatibility remain unchanged | focused/static/live validation | completed |
| P08-C07 | focused and affected broad validation passes | nonzero counts and static checks | completed |
| P08-C08 | phase-exit review is Complete / High | `COMPLETION_AUDIT.md` | completed — Complete / High |

## Stop Conditions

Stop and do not activate Phase 09 if current code already derives the complete final aggregate once; the defect can
only be reproduced by invalid policy/data; one transition cannot preserve established same-tile/Rest or unrelated
modifier semantics; relocation/swap assignment policy must change; public/data/save shape must change; dirty overlap
is inseparable; a focused runner selects zero tests; or completion review is not `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.

## Completion Result

- Phase exit audit: `Complete / High`.
- Improvement class: `none`.
- Full-final-state derivation and one reconciliation now own Focus/block/Rest for stance, relocation/swap and
  field-work lifecycle.
- Complete Core validation: 948 passed, 0 failed; no data, DTO/wire/public or persistence migration.
- Master may mark Phase 08 complete and make Phase 09 ready.
