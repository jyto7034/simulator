# Phase 03 Boundary Audit

## Authority And Consumer Result

- Static employee/equipment/fragment/reward catalogs authorize definitions; root roster, inventory, fragment
  inventory, UUID manager and Enkephalin own run-local player assets.
- Employee slot references and inventory `equipped_to` are one logical association stored in two persistent ledgers.
- `GrantExecutor` owns ordered effect semantics but current callers own clone/commit atomicity independently.
- V3 checkpoint is a carrier for these roots; its validator is the admission authority and currently lacks their
  internal graph predicate.
- HQ owns recruitment/emergency/shop session decisions, while generic `with_state_transaction` owns complete
  non-battle rollback semantics.
- Boss setup consumes player equipment/enhancement state and BossOmen consumes fragment/run state. Private Boss
  runtime owns neither persistent ledger nor grant/checkpoint/HQ mutation.

## Candidate And Cross-Boundary Result

- CRA-P03-C02/C03/C04 remain current and independent.
- CRA-F-C01's P03 economy-graph half remains current and composes with, but does not replace, P02 seed/progression
  validation evidence.
- CRA-F-C02's P03 HQ rollback half remains current; P04 must inspect the interactive node/session half before Final
  resolves the merged identity.
- Current Boss changes expand consumers and regression coverage, not the P03 target authorities.

## Consumer/Gate Classification

- workspace-confirmed: Core command handlers, checkpoint writer/reader, battle setup, snapshots, reward/shop/event/
  combat/admin callers and current live RON.
- policy gated: old-save repair/rebind, recipe balance/content, removal of serialized binding or changed checkpoint
  lifetime.
- external gated: public `GrantExecutor`/inventory signature removal and unknown standalone Rust/save tooling.
- explicitly excluded: Unity economy UI, server-independent economy redesign and Boss content/balance work.

## New Candidate Result

No `CRR-P03-*`. BossOmen/active Boss runtime remains a run/battle lifecycle consumer and creates no independent
roster, inventory, grant, checkpoint economy or HQ rollback defect.

## Handoff

P04 receives CRA-F-C02 with the HQ field-list evidence still open and audits interactive session lifecycle. Final
receives CRA-F-C01's confirmed P02+P03 contributions as one identity. P03 exits with five still-valid rows or merged
contributions and no new candidate.
