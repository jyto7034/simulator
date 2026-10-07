# Phase 07 Battle Spatial Tactical And Boss Movement Reaudit Goal

Status: complete
Implementation Complete: not_applicable
Goal Type: phase
Execution Mode: investigation_only
Production Mutation Authorized: no
Direct Dependency: P06 `Complete / High`

## Objective And Scope

Reaudit CRA-P07-C01~C04 against current battlefield/movement/tactical commands, including accumulated tactical
stance/relocation/swap work, and deeply inspect Boss target-lock/path/directive integration. Verify the two independent
and two deferred Boss dispositions without absorbing the tactical campaign itself.

Artifacts: `CANDIDATE_REAUDIT.md`, `TEST_REAUDIT.md`, `BOUNDARY_AUDIT.md`, `COMPLETION_AUDIT.md`.

## Invariants

Preserve final-state tactical effects, failure-no-mutation, board/roster/resource/revision parity, deterministic
continuous movement, Boss goal/suppress/fallback and public/external gates. Production/history/dirty content is
read-only.

## Validation And Gates

Read original P07 catalog and current tactical/movement/world/tests/canonical. Run focused tactical/Boss movement
tests. New IDs require distinct `CRR-P07-*` evidence. Backend/public API removal and gameplay movement policy remain
gated.

## Evidence Matrix

| ID | Condition | Status |
| --- | --- | --- |
| P07-C01 | P07-C01 disposition | pass — still valid |
| P07-C02 | P07-C02 disposition | pass — still valid |
| P07-C03 | P07-C03 disposition | pass — still valid |
| P07-C04 | P07-C04 disposition | pass — still valid |
| P07-C05 | Boss/new-candidate impact | pass — no new candidate |
| P07-C06 | tests/gates/preservation | pass — 101 tests and exact hashes |
| P07-C07 | Complete / High audit | pass |
