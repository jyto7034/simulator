# Artifact Legacy Removal Notes

Goal: G27

Status: complete

## Policy Source

- Master PD-17: remove Artifact completely without replacement.
- G24: ordinary inventory remains equipment instances plus consumable definition stacks and reserve
  high-water; Artifact was deliberately excluded.
- G28: Shop is complete and contains no Artifact merchandise or contract dependency.

## Guardrails

- No run relic, containment relic, passive collection, translated equipment, compensation grant, or
  other replacement state.
- No empty `artifacts` list/catalog, removed enum variant alias, old reward decode, save migration,
  or deprecated Unity field.
- Preserve historical goal text as evidence; update only current canonical/master/active-goal text.
- Preserve unrelated dirty worktree content and do not reset, restore, stash, clean or baseline-commit.

## Removal Trace

- Static/live data: `artifact_data.rs`, `live_ron.rs`, `data/mod.rs`, validation, live RON.
- Inventory/reward/admin/DTO: `resources/inventory.rs`, `reward.rs`, `reward_data.rs`, `behavior.rs`,
  `world/admin*`, `world/snapshot.rs`.
- Battle: scenario, combat setup, core build/types/ids/triggers/commands/cooldown, stat pipeline,
  event log and validation.
- External: server mappings/docs, Rust fixture generators, Unity models/admin commands/mocks/tests,
  and current canonical contracts.

## Completion Boundary

- Artifact has no live schema, data, loader, registry, inventory, reward, admin, battle, DTO, server,
  generated fixture, Unity or current canonical surface.
- Ordinary inventory remains exactly G24 equipment-instance plus consumable-definition-stack state;
  Shop remains exactly G28's typed deterministic contract. No translated item or compensation was
  created.
- Historical goal documents intentionally retain superseded observations as evidence and are not a
  decode, runtime, content, public contract or current canonical surface.
- Completion review verdict: Complete / High; no correction goal is required. Activate R06.
