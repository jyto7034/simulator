# Battle Setup Catalog Ref Correction

Status: complete

Implementation complete: true

Started: 2026-07-18

Master: `docs/goals/documentation_code_alignment_corrections/PLAN.md` G02

Finding: `DCA2-WIRE-003`

## Objective

Ensure `battle_setup_snapshot.catalog_refs.abnormality_ids` contains only sorted, deduplicated,
non-empty actual abnormality catalog IDs for every live encounter kind.

## Source Of Truth

- Master G02 scope
- external `core_unity_battle_transport_contract.md` actual-abnormality-only rule
- Core `BattleCoreState::live_setup_snapshot`
- live PVE preview/wave identity data

## In Scope

- Kind-aware, non-empty setup ref collection from primary identity, briefing and waves
- Corroded-only, abnormality and mixed representative regressions
- game_server pass-through confirmation and focused registered smoke when practical

## Out Of Scope

- Battle event presentation (excluded P1)
- Catalog preload implementation in Unity
- SpawnWave identity schema changes

## Implementation Plan

1. [x] Inspect current producer, catalog validation and tests.
2. [x] Reproduce the empty ref evidence and strengthen focused Core coverage.
3. [x] Correct actual-abnormality ID collection at the Core setup producer.
4. [x] Add exact corroded-only, actual and mixed assertions.
5. [x] Run combat-focused, server and registered live probe validation.

## Test Requirements

- Corroded-only setup refs empty
- Actual abnormality setup refs exact, non-empty and resolved
- `cargo test -p game_core game::world::tests::combat --lib -- --test-threads=1`
- `cargo test -p game_server -- --test-threads=1`
- registered smoke or documented substitute if unchanged external transport cannot be run

## Completion Conditions

- No setup can serialize an empty or corroded profile sentinel in `abnormality_ids`.
- Actual abnormality refs remain present, sorted and deduplicated.
- Server forwards the corrected DTO unchanged.
- Focused/broad relevant gates pass.

All completion conditions satisfied on 2026-07-18.

## Result

- Primary identity, enemy briefing and spawn-wave candidates now require a non-empty ID that
  resolves in the loaded abnormality catalog.
- Briefing and wave candidates additionally require `EnemyKind::Abnormality`.
- The resulting list remains sorted and deduplicated.
- Live RON regressions cover actual-only, actual-plus-corroded mixed and corroded-only encounters.
- The registered WebSocket probe observed a corroded-only setup with
  `catalog_refs.abnormality_ids: []`.

## Stop Conditions

- Fix requires changing SpawnWave/Unity public identity policy.
- Current external contract differs from Master-approved actual-abnormality-only meaning.
