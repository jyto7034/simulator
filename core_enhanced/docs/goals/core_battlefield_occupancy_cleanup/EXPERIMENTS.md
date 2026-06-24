# Experiments

This file records implementation attempts, validation commands, failures, fixes, and final verification for `core_battlefield_occupancy_cleanup`.

## Log

| Date | Scope | Attempt | Result | Follow-up |
| --- | --- | --- | --- | --- |
| 2026-06-23 | Goal setup | Created the subgoal document for removing single-tile battlefield occupancy semantics. | Pending implementation. | Start with an inventory of all occupant and position ownership use sites. |
| 2026-06-23 | inventory | Searched `occupant`, `PositionOccupied`, `place_allowing_unit_overlap`, `place`, `remove`, and `unit_pos` across battlefield/runtime/world/tests. | Single occupant storage was localized to `BattlefieldTile`, `Battlefield::occupant`, `place`, `place_allowing_unit_overlap`, `remove`, and helper tests. Gameplay overlap behavior already used continuous movement/body state elsewhere. | Remove `Tile`/`tiles`/`occupant` and update tests/helpers to multi-occupant `units_at`. |
| 2026-06-23 | implementation | Removed `Tile` and `tiles` from `Battlefield`; removed `Battlefield::occupant` and `place_allowing_unit_overlap`; made `place` allow unit overlap while still rejecting static obstacles/out-of-bounds/duplicate unit placement; added `units_at(pos)` derived from `unit_pos`. | `cargo check --lib` passed and legacy occupant search has no battlefield occupant storage/use matches. | Update focused tests and record validation. |
| 2026-06-23 | deployment policy test update | Updated `live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock` to assert allied deployment may share a tile and withdrawing one unit leaves the other deployed. | Focused deployment test passed. | Run broad validation in master final pass. |

## Failed Approaches

Record failed approaches and why they were abandoned.

## Validation Commands

Record focused and broad validation commands here.

- `cargo check --lib` (pass)
- `cargo test battlefield --lib -- --test-threads=1` (pass)
- `cargo test movement --lib -- --test-threads=1` (pass)
- `cargo test live_deployment --lib -- --test-threads=1` (pass)
- `cargo test live_defense_deploy_and_withdraw_manage_cost_and_redeploy_lock --lib -- --test-threads=1` (pass)
- `rg -n "occupant|place_allowing_unit_overlap|tiles: Vec<Tile>|struct Tile|\\.tiles\\[" src/game/battle src/game/world tests -S` (pass: no single occupant storage/use remains; only `TileRangePattern` substring matched in a broad search)
