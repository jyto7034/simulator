# Artifact Legacy Removal

Goal: G27

Finding: MSTR-02

Status: complete

Implementation complete: true

Updated: 2026-07-14

## Objective

Delete the legacy Artifact subsystem completely without replacement after G24 finalized ordinary
inventory and G28 removed every Shop dependency.

## Accepted Boundary

- Delete live Artifact RON, loader/database/builder/registry types and cross-catalog validation.
- Delete Artifact inventory slots/items/DTOs/errors, reward kinds/effects, admin commands/catalog,
  root snapshots, display variants, generated fixtures, server mappings, Unity admin contracts and
  mocks/tests.
- Delete battle scenario/runtime Artifact instances, stat modifiers, triggers, cooldown ownership,
  event-log events, validation counters and IDs.
- Remove canonical claims that Artifact is current gameplay. Historical completed goal evidence may
  retain the word only as history and is not compatibility authority.
- Do not migrate Artifact names, effects, prices, IDs or state into equipment, consumables, skill
  fragments, Shop, or a new relic/run system.
- Do not add compatibility decoding, aliases, empty placeholder fields/catalogs, tombstone state, or
  ignored tests.

## Validation

- Strict loading rejects the deleted `grant_artifact` reward vocabulary and no Artifact source file
  is required or embedded.
- Full Core and game_server suites pass after all schema/runtime variants are removed.
- Rust-generated inventory/Shop fixtures and Unity models/tests contain no Artifact fields.
- Full Unity EditMode and PlayMode suites pass with zero compile-console errors.
- Repository-wide current-source/live-data/server/Unity/canonical search finds no Artifact surface;
  historical goal records are explicitly excluded from the production compatibility audit.
- Independent completion review before R06 begins.

## Exit Gate

Complete / High only when Artifact has no live data, loader, registry, inventory, reward/admin,
battle/stat/trigger/log, DTO/server/Unity, fixture, current canonical, or ignored-test surface, and no
replacement or compatibility layer was introduced.

Result: Complete / High. The subsystem and its live content were deleted across every current
cross-layer surface, strict removed vocabulary fails closed, and no replacement or compatibility
state was introduced. R06 may begin.
