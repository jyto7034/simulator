# Phase 22 Goal — CRA-P03-C03 Encapsulate Bidirectional Equipment/Employee Loadout Transitions

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Candidate ID: CRA-P03-C03
Master Phase: 22 of 44
Work Type: behavior-preserving refactor
Production Mutation Authorized: yes — the user's Master execution request remains active; authorization is limited
to this Phase after Phase 21's `Complete / High` exit
Repository Root: `/mnt/f/work/simulator`
Activation Date: 2026-08-10 (`Asia/Seoul`)

## Objective And Completion Conditions

Consolidate production mutation of the employee loadout slot and owned-equipment `equipped_to` reverse binding
behind one private typed roster-inventory transition authority. Every current move, replacement, unequip, starter,
death and dismantle lifecycle must continue to produce the exact same two-ledger state and observable outcome.

This Phase is complete only when every current mutation and consumer of either side is inventoried; tests first
freeze an exact transition matrix over both full ledgers for empty equip, cross-employee move, same-target behavior,
slot replacement, bound rejection, unequip, starter setup, death recovery and dismantle; valid V3 state admitted by
F-C01 remains valid after each legitimate transition; invalid transitions remain atomic; one private typed boundary
owns paired writes for all production lifecycles; redundant direct production synchronization is removed; base and
instance identity, slot/type/bound rules, enhancement, displaced-result/diff ordering, existing errors and outer
transaction behavior remain exact; public Rust/V3/RON/DTO/wire compatibility is unchanged; focused and broad
debug/release validation passes; and an independent phase-exit audit returns `Complete / High`.

## User Or Contract-Visible Problem

The same equipment association is persisted twice: an employee slot contains an `EquippedRef`, while the owned
equipment instance records the employee UUID in `equipped_to`. Historical and Boss-era audits found that equip,
unequip, starter, death, dismantle and fixture/direct inventory seams coordinate those ledgers manually in different
staged states.

Current valid production paths and their selected tests pass. This is therefore an ownership and
change-amplification risk, not evidence that a normal command currently commits half a transition and not
authorization to change gameplay or save shape.

## Dependency And Policy Authority

- Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`, Phase 22 registry row.
- Direct linear predecessor: Phase 21 CRA-P02-C04, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_21_p02_c04/COMPLETION_AUDIT.md`.
- Correctness dependency: Phase 02 CRA-F-C01, reviewed `Complete / High` in
  `../core_enhanced_refactoring_rebaselined_backlog_implementation_phase_02_f_c01/COMPLETION_AUDIT.md`.
- Compatibility policy: CRA-POL-006 in
  `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/APPROVED_POLICY_DECISIONS.md`.
- Original evidence:
  - `../core_enhanced_refactoring_audit_phase_03_roster_inventory_and_economy/CANDIDATE_CATALOG.md`
  - `../core_enhanced_refactoring_audit_phase_03_roster_inventory_and_economy/BOUNDARY_AUDIT.md`
- Re-audited evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_03_roster_inventory_and_economy/CANDIDATE_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_03_roster_inventory_and_economy/TEST_REAUDIT.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_phase_03_roster_inventory_and_economy/BOUNDARY_AUDIT.md`
- Final order/test/gate evidence:
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/EXACT_LINEAR_IMPLEMENTATION_ORDER.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/TEST_REBASELINE_CATALOG.md`
  - `../core_enhanced_refactoring_post_boss_runtime_reaudit_final_catalog_consolidation/IMPLEMENTATION_WAVES_AND_GATES.md`

F-C01 now rejects contradictory roster/inventory/loadout graphs before restore. CRA-POL-006 permits private
encapsulation but forbids deleting either ledger, weakening strict V3/RON admission, narrowing public compatibility
or adding an unapproved migration/repair path.

## Priority And Current Disposition

- Priority axes: `3 / 2 / 2 / 3 / 3 / 3 / unknown`; known subtotal `16/18`.
- Re-audited disposition: `still_valid`, `behavior refactor; characterize after F-C01 economy validation`.
- Current live-tree disposition: complete; P22-C01–P22-C10 passed the independent phase-exit audit at
  `Complete / High`.

## Current And Target Authority

- Historical current authority: employee roster owns loadout slots; equipment inventory owns run-local instances
  and `equipped_to`; separately staged command/lifecycle callers coordinate both.
- Historical consumers: safezone equip/unequip, starter setup, combat-result death recovery, dismantle, fixtures,
  checkpoint capture/restore and snapshots/battle setup.
- Target authority: the smallest private typed transition aggregate receives the required staged roster and
  inventory, validates a whole transition once, updates both representations together and returns the existing
  outcome data in the existing order.
- F-C01 remains import admission authority. The transition boundary owns valid live mutation, not repair of an
  invalid checkpoint and not public/save schema.
- Exact current types, methods, callers, public compatibility surfaces and direct-write counts must be rediscovered
  only after the mandatory activation gates.

## In Scope

- inventory/equipment instance and employee/loadout ownership, safezone equip and unequip, starter loadout, death
  recovery, dismantle and any production fixture/direct mutation that writes one side of the association;
- enumerate each operation's preconditions, source and target employee/slot, displaced instance, paired effects,
  return/error/diff ordering and outer stage/commit owner;
- capture expanded dirty status, scoped diffs and prospective source/test/canonical hashes before content inspection;
- add a test-first transition matrix asserting both full ledgers and exact outcomes for success and invalid
  no-mutation cases;
- retain or strengthen post-transition F-C01 invariant and V3 round-trip/snapshot agreement evidence;
- introduce the smallest private typed transition authority and migrate legitimate production callers;
- remove only proven duplicate direct synchronization superseded by that authority;
- update the nearest durable equipment/loadout canonical only if ownership wording materially changes.

## Out Of Scope

- deleting, renaming, privatizing or changing either serialized/public association representation;
- changing V3, RON, DTO, wire, public Rust inventory/roster construction or compatibility behavior;
- repairing, rebinding or choosing a winner for contradictory imported ledgers;
- changing item definition/instance UUID meaning, metadata identity, equipment type or slot compatibility;
- changing bound rules, safezone admission, enhancement, dismantle recipes/yields or death recovery semantics;
- changing displaced-item/result/diff order, errors, checked arithmetic, UUID allocation or transaction timing;
- changing battle setup/snapshot projection or server/Unity behavior;
- Phase 23 five-ledger grant transaction work, Phase 25 HQ transaction work or any later cleanup/prebuild;
- broad formatting, data rewrite or dirty-worktree normalization.

## Preserved Invariants

1. Employee loadout slots and owned-equipment `equipped_to` describe the same one-to-one association.
2. Both current public/serialized V3 fields, names, types and exact values remain present and strict.
3. Equipment definition UUID, owned instance UUID and employee UUID remain distinct identities.
4. Existing item type/slot compatibility and bound-item rules remain exact.
5. Move and replacement preserve the current source clear, target install, displacement and outcome order.
6. Same-target, empty, missing, mismatched and otherwise invalid operations preserve existing result/error behavior
   and do not partially mutate either ledger.
7. Enhancement remains attached to the same owned instance through equip transitions.
8. Starter setup, death recovery and dismantle produce the same employee/inventory/material/result state and order.
9. F-C01 rejects contradictory imported state before restore; valid live transitions preserve its graph invariant.
10. Existing outer clone/stage/commit atomicity, checkpoint/snapshot/battle consumers and server/Unity contracts stay
    unchanged.
11. Existing user changes are preserved without reset, restore, clean, stash or unrelated formatting.

## Implementation And Dirty-Worktree Strategy

After all Child documents exist and Master records sole activation, read them completely and reread the full final
Phase 21 completion audit. Capture expanded status plus prospective source/test/canonical hashes before production
detail. Treat every existing hunk as user/campaign-owned until reconciled.

Use characterization-first proof in this order:

1. Trace both ledgers, every mutation entry point, all staging/commit owners and every projection/import consumer.
2. Build an exact transition/caller/direct-write table for move, replace, unequip, starter, death and dismantle.
3. Run current equipment/loadout/starter/death/dismantle/checkpoint/snapshot baselines with nonzero counts.
4. Add full-state valid and invalid characterization before structural changes.
5. Introduce one private typed paired transition boundary without changing public/serde types or caller outcomes.
6. Transfer legitimate callers and remove only obsolete paired-write duplication.
7. Re-run focused, complete Core debug/release, server debug/release and static/compatibility gates.

This is behavior-preserving, so intended behavioral red is not required. A structural red is acceptable only when a
new test directly proves the planned absent transition boundary or an outside production write; compile-only
method-count evidence is not sufficient by itself.

## Migration, Deployment And Compatibility

- No migration, fallback, repair or coordinated rollout is approved or required.
- Both ledgers and current public APIs remain because strict V3 and unknown external consumers may construct/read
  them. Negative search never authorizes narrowing.
- Server and Unity continue to consume the same snapshots/DTOs and never learn the private transition type.
- If safe consolidation requires schema, public API, gameplay, error or ordering change, stop as
  `policy_decision_required`.

## Validation Plan

- Baseline: equipment/loadout commands, inventory resources, starter/start snapshot, death recovery, dismantle,
  checkpoint validation/round trip and battle/snapshot consumers.
- Test-first matrix: empty equip, cross-employee move, same-target behavior, slot replacement, bound rejection,
  unequip, death, dismantle and starter with exact full roster+inventory and existing outcome assertions.
- Invalid/atomicity: missing/mismatched instance, employee, reverse owner, duplicate slot and incompatible/bound
  cases leave both staged ledgers and outer state unchanged at the existing error boundary.
- Restore: writer-produced V3 and every legitimate post-transition state satisfy F-C01 without repair.
- Behavior: definition/instance identity, enhancement, material grants, return/diff order and UUID continuation stay
  exact.
- Broad: complete Core debug/release, official integrations and complete server debug/release.
- Static/compatibility: public/serde field/API inventory, direct-write/caller search, scoped hashes/status/diff,
  formatter, canonical comparison and no Phase 23 prebuild.

## Completion Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P22-C01 | complete two-ledger mutation/consumer graph identified | caller/direct-write/staging inventory | complete |
| P22-C02 | dirty targets and concurrent work preserved | pre/post status, scoped diff and hashes | complete |
| P22-C03 | current transition behavior characterized first | exact full-state success/error matrix | complete |
| P22-C04 | one private typed paired transition authority exists | source/type/caller inventory | complete |
| P22-C05 | legitimate production callers use the authority | move/replace/unequip/starter/death/dismantle evidence | complete |
| P22-C06 | obsolete direct production synchronization is removed | negative direct-write/fallback audit | complete |
| P22-C07 | identity, rules, outcomes and atomicity remain exact | focused parity and invalid no-mutation evidence | complete |
| P22-C08 | F-C01/public/V3/RON/DTO/server/Unity remain exact | restore and compatibility evidence | complete |
| P22-C09 | focused and broad validation is green | nonzero debug/release/Core/server/static counts | complete |
| P22-C10 | canonical/downstream boundary and audit are current | no-prebuild audit and `Complete / High` review | complete |

## Stop Conditions

Stop and do not activate Phase 23 if either ledger/public API must be removed or narrowed; V3/RON/DTO/wire shape or
strict admission must change; gameplay, error, ordering, UUID, enhancement, bound, death or dismantle behavior must
change; an external consumer cannot be preserved; F-C01 would be bypassed or turned into repair; user-owned dirty
overlap cannot be separated; required tests select zero cases; or the independent audit is below `Complete / High`.

사용자와 의논하여 정해야 할 정책이 있을 경우 Goal을 종료한다.
