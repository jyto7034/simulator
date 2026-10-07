# Phase 02 Boundary Audit

## Authority And Consumer Result

- Core owns V3 checkpoint payload capture/validation/restore and treats server bytes as the serialized producer.
- Host `ActiveRunStore` owns envelope checksum/version, current/backup choice, tombstone and host monotonic metadata.
- Server actor owns continue/bootstrap ordering and publish calls, not Core payload meaning.
- `MapProgression`/`RunMap` jointly own active navigation; graph-derived selectability remains admission authority.
- Current active `BattleCore`, including Boss runtime, tactical state and event queue, is not in V3 and is cleared on
  checkpoint restore. This is approved checkpoint lifetime, not missing serialization.

## Candidate And Cross-Boundary Result

- C01/C03/C04 remain current and independent.
- F-C01 P02 seed/progression half remains current; P03 asset graph evidence is mandatory.
- P01's fully implemented publication gate ensures current data validity but does not validate a user document's
  cross-field checkpoint graph.
- P10 must retain host/public consumer gates; P06/P10 own live battle reconnect/checkpoint projection distinction.

## Consumer/Gate Classification

- workspace-confirmed: Core V3 writer/reader, server actor and local active-run store.
- policy gated: multi-actor logical run lease/conflict, old version repair/migration, admin durability.
- unknown: external public Rust/save tooling and deployment transfer of local files.
- explicitly excluded: designing new Boss cross-process restore or compatibility.

## New Candidate Result

No `CRR-P02-*`. Private Boss exclusion does not create parallel save authority; it is the existing explicit
checkpoint lifetime. Later world changes do not introduce a distinct persistence root cause beyond assigned CRA
rows/signals.

## Handoff

P03 receives F-C01 with seed/progression evidence still open and audits roster/inventory/economy edges. P02 does not
close the merged identity. Current Phase result is four still-valid rows/contributions and no new candidate.

