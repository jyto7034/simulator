# Core Enhanced Refactoring Audit Phase 02 — Candidate Catalog

Catalog Date: 2026-08-07
Status: investigation complete; final Phase disposition pending completion audit

Priority axes are ordered as: ownership duplication, correctness/atomicity, lifecycle complexity, change
amplification, test fragility, failure blast radius, change frequency. Current accumulated dirty work makes Git
history non-representative, so change frequency is `unknown` and no automatic total-band verdict is used.

## Confirmed Candidates

### CRA-P02-C01 — Enforce Host Monotonic Metadata On Every Current Publish

- Work Type: `correctness_correction`
- Problem Evidence:
  - `loads_used` and `run_peak_certified_power` are explicitly run-scoped host monotonic metadata.
  - `ActiveRunStore::publish()` max-merges the incoming values into the previous payload only when writing the
    rolling backup.
  - The incoming checkpoint is encoded and installed as current without comparing it to valid current metadata.
    Therefore a lower valid publish becomes the preferred next load even though the backup retains the maximum.
  - Current store tests prove backup fallback never decreases these values, but none publishes a lower valid
    current and loads it without corruption.
- Current Authority And Consumers: Core exports current values; the store owns durable monotonic lifetime and
  current/backup choice; player actors call publish after checkpoint/peak changes.
- Target Authority: before replacing current, the store max-merges any readable current monotonic values into the
  incoming envelope while preserving the incoming Core payload generation. The host store enforces its own
  invariant even if a caller is stale.
- Preserved Invariants: strict host/Core V3, incoming checkpoint payload/anchor, three-load ceiling, run peak high
  water, backup generation, checksum, tombstone, write/fsync/rollback behavior and error shapes.
- Removal Or Consolidation Scope: remove the current-only monotonic regression path. Do not choose multi-actor
  payload conflict policy, migrate old saves, or merge checkpoint-owned anchor/model with host metadata.
- Priority Axes And Total: `2 / 3 / 2 / 1 / 3 / 3 / unknown`; known subtotal `14/18`, no automatic band.
  Correctness risk `3` independently triggers priority investigation.
- Validation Safety Net: first add real-file tests for higher-current→lower-incoming publish for each field and
  both together; prove the incoming Core payload remains current while metadata stays at max; rerun all 13
  active-run server tests and Core round-trip tests.
- Policy/External Contract Decision: none; the current canonical already requires monotonicity. Cross-actor
  payload conflict/lease behavior remains separate deferred Signal S05.
- Readiness: `test_first_then_implementation`.
- Disposition: `priority_candidate`.

### CRA-P02-C02 — Validate Cross-Field Checkpoint Identity And Progression Before Restore

- Work Type: `correctness_correction`
- Problem Evidence:
  - Durable payload carries `RunProgression::run_seed` and a serialized private `UuidManager::run_seed`; Core also
    has `GameCore::run_seed`.
  - `validate_run_checkpoint_payload()` does not compare these values. UUID restore protects equality only with
    `debug_assert_eq!`, which is absent in release behavior.
  - The validator checks that progression IDs exist but not that current/completed/available lists, node states
    and graph-derived selectable IDs form one consistent progression.
  - Restore mutates run/root state and UUID continuation from these fields. A checksummed but inconsistent local
    document can therefore split RNG identity or install semantically inconsistent progress instead of returning
    an explicit system issue.
  - Existing negative tests cover version, weapon tactical schema, model/anchor, monolith, template and
    recruitment ownership, but not seed or progression consistency.
- Current Authority And Consumers: Core import validator gates `continue_from_active_run_checkpoint()` and
  explicit load; `MapProgression`, `RunMap`, `RunProgression` and `UuidManager` each own part of the invariant.
- Target Authority: one context-aware pre-mutation checkpoint validator always compares payload progression and
  UUID-manager seed, compares the active Core seed for explicit in-run load, and lets Continue adopt the validated
  payload seed before restore. It also calls an exact `MapProgression` invariant validator against the resolved
  authored template/map.
- Preserved Invariants: every valid V3 payload byte shape and restore result, deterministic IDs, VN counter
  preservation, current map assignment, explicit failure rather than repair, no compatibility reader and no
  partial mutation.
- Removal Or Consolidation Scope: remove acceptance of internally inconsistent current-version documents and the
  release-only reliance on a debug assertion. Do not regenerate a map, repair a save, or infer a missing field.
- Priority Axes And Total: `2 / 3 / 2 / 2 / 3 / 3 / unknown`; known subtotal `15/18`, no automatic band.
- Validation Safety Net: add mutated-V3 negative tests for progression seed/UUID seed mismatch, invalid current
  state, duplicate/misaligned completed IDs and stale available IDs; assert Core state/export are unchanged after
  every rejection; rerun map, facility, checkpoint and server active-run filters.
- Policy/External Contract Decision: rejecting inconsistent current-version documents follows existing strict
  restore policy. If repair/migration is desired, that is a separate `policy_or_feature_migration` Goal.
- Readiness: `test_first_then_implementation`; exact valid-progression predicate should be characterized before
  the validator is centralized.
- Disposition: `priority_candidate`.

### CRA-P02-C03 — Consolidate RunState To V3 Checkpoint Capture/Restore Mapping

- Work Type: `behavior_preserving_refactor`
- Problem Evidence:
  - `RunState` and `RunCheckpointPayload` repeat 13 run-owned fields.
  - `save_run_checkpoint()` manually copies those fields plus eight root fields into the payload;
    `restore_run_checkpoint_payload()` manually assigns the inverse mapping.
  - `RunState::for_next_floor_from()` maintains a separate hand-authored carry-forward subset. The lifetimes are
    intentionally different, but a new persisted run field requires coordinated edits with no compile-time or
    golden field-parity gate.
  - Current round-trip tests pass, so this is change-amplification/omission risk rather than a current behavior
    defect.
- Current Authority And Consumers: `RunState` is active authority; `RunCheckpointPayload` is a derived V3
  snapshot; save/restore functions own manual conversion; server treats Core bytes as opaque.
- Target Authority: a typed private checkpoint-owned run aggregate or explicit `capture_from`/`restore_into`
  conversion owns the 13-field mapping once. Root-only and deliberately noncheckpointed fields remain explicit.
- Preserved Invariants: exact V3 serialized JSON shape/order-insensitive meaning, all current field values,
  `combat_recovery_restriction: None` on restore, player/compendium/run peak exclusions, UUID/VN load behavior,
  Floor carry-forward behavior and public API.
- Removal Or Consolidation Scope: consolidate only the private bidirectional mapping. Do not merge active
  `RunState` with the persisted document, change version, add battle state, or generalize next-Floor persistence.
- Priority Axes And Total: `2 / 1 / 2 / 3 / 2 / 3 / unknown`; known subtotal `13/18`, no automatic band.
- Validation Safety Net: add a golden V3 field-set test and full round trip covering every run/root checkpoint
  field plus explicit exclusions; rerun the 11 facility, 18 node-session, 23 start/snapshot and 13 server tests.
- Policy/External Contract Decision: none while the byte shape and exclusions remain exact. Any field removal,
  version bump or compatibility reader is deferred migration work.
- Readiness: `ready_after_golden_field_parity_characterization`.
- Disposition: `candidate`.

### CRA-P02-C04 — Encapsulate MapProgression Mutation And Derived Availability

- Work Type: `behavior_preserving_refactor`
- Problem Evidence:
  - `MapProgression` publicly stores `current_node_id`, `available_node_ids` and `completed_node_ids`, while each
    `RunMap` node also stores availability/completion state.
  - Selection and snapshot use graph-derived `selectable_node_ids()`; the stored available vector is maintained
    by progression methods but is not the sole admission authority.
  - `boss_omen` and admin fixtures directly assign progression fields and node states outside the progression
    module, creating multiple mutation seams for one map invariant.
  - Current focused map tests all pass; no runtime drift was observed in valid production flows.
- Current Authority And Consumers: authored `FacilityTemplate` owns topology; `RunMap` and `MapProgression` jointly
  own current progress; world/boss/admin callers can mutate public components.
- Target Authority: `MapProgression` exposes typed transition/forced-terminal/fixture helpers that update stored
  IDs and node states together and can validate their invariant. Admission remains derived from authored graph.
- Preserved Invariants: public/V3 field shape, deterministic sorting/deduplication, completed-node transit,
  directed edges, concealed/obscured semantics, BossOmen exit reveal, admin fixture results and map DTO shape.
- Removal Or Consolidation Scope: remove direct production mutations outside the progression API. Do not remove
  `available_node_ids` from serialization or change navigation rules in this refactor.
- Priority Axes And Total: `3 / 2 / 2 / 2 / 2 / 2 / unknown`; known subtotal `13/18`, no automatic band.
- Validation Safety Net: characterize each transition's exact map/progression pair, BossOmen forced exit and
  invalid no-mutation cases; keep existing 25 map and 40 map-flow tests; add invariant assertions after restore.
- Policy/External Contract Decision: private mutation encapsulation needs none. Removing a V3 field or changing
  what Unity sees requires migration/consumer evidence and is excluded.
- Readiness: `ready_after_transition_characterization`.
- Disposition: `candidate`.

## Rejected, Deferred, And Unverified Signals

| Signal ID | Signal | Evidence and reasoning | Disposition |
| --- | --- | --- | --- |
| CRA-P02-S01 | Merge Core payload version and host envelope version because both are 3 | They protect independent schemas and are owned by Core and host storage respectively. Canonical explicitly names strict V3 at both layers. One constant would create the authority coupling the design avoids. | `rejected` |
| CRA-P02-S02 | Remove current→backup automatic recovery as fallback debt | Recovery is checksum-validated, explicit, restores current atomically, exposes recovery state and has real-file tests. It is approved failure behavior, not silent fallback. | `rejected` |
| CRA-P02-S03 | Add V2/legacy active-run migration while refactoring | Source, canonical and tests explicitly reject V2/missing/removed fields before current-body interpretation. Accepted versions and repair policy are not delegated to this audit. | `deferred`; `policy_or_feature_migration` only |
| CRA-P02-S04 | Route `ContinueRun` through ordinary `GameCore::execute()` | Host must first load/verify envelope and inject transport identity/host metadata. Core's ordinary route deliberately returns `InvalidAction`; merging the paths would erase the host bootstrap boundary. | `rejected` |
| CRA-P02-S05 | Multiple transport-player actors can receive the same local-install store and independently continue/publish one run | Source confirms per-player Core actors share one store mutex; the mutex serializes calls but does not assign a logical run lease or reject stale payload branches. Canonical defines local-install scope but not cross-player ownership/conflict resolution. C01 protects monotonic fields only and must not silently choose payload ownership. | `deferred`; requires explicit lease/conflict/player-identity policy Goal |
| CRA-P02-S06 | Ordinary admin mutation bypasses the active-run command transaction wrapper | Confirmed development-only handler calls Core admin mutation and snapshots directly. Whether admin mutations should persist, invalidate, or remain ephemeral is an admin/tooling policy outside current gameplay contract. | `deferred` to admin policy/consumer audit |
| CRA-P02-S07 | Delete unused `RunProgression::current_floor_seed()` | No workspace caller was found, but it is a public Rust method and external consumers are unknown. Absence of workspace use is insufficient deletion proof. | `unverified`; defer consumer inventory |
| CRA-P02-S08 | Remove `available_node_ids` from `MapProgression`/V3 payload as duplicate state | C04 can encapsulate mutation while preserving shape. Field removal changes the current persisted schema and needs save/external-consumer policy. | `deferred`; migration/API decision |
| CRA-P02-S09 | Generalize all non-battle commands under one cloned-state transaction because a fixture can make `StartNewGame` fail after early mutation | Current official loader rejects missing RunStart trigger data before runtime; no current live path or durable checkpoint mutation was shown for that fixture-only error. Transaction expansion without command-specific evidence could alter monotonic/profile behavior. | `rejected` as current candidate; retain as test-gap signal |
| CRA-P02-S10 | Merge `BattleEntrySnapshot`, world transaction snapshot and active-run rollback | They restore different lifetimes and deliberately preserve/exclude different monotonic aggregates. Similar field copies do not establish shared authority. | `rejected`; battle internals remain later Phase |
| CRA-P02-S11 | Replace validated map-generation `expect` with a fallback map | Publication/generation contracts require authored schedule generation to succeed and reject invalid data. A fallback would invent gameplay and conceal content failure. | `rejected` |
| CRA-P02-S12 | Audit checkpoint-owned economy/session/combat-preview internals in this Phase | They are copied only to establish lifecycle evidence. Their domain correctness belongs to Phases 3–5 and must not be inferred from successful round trip alone. | `deferred` cross-boundary handoff |

## Cross-Boundary Handoff

- Phase 3 may treat checkpoint capture/restore as a confirmed carrier for roster/inventory/resource fields but
  must audit their economic ownership separately.
- Phase 4 owns event/shop/VN session semantics; Phase 2 proves only that current session maps and checkpoint-scoped
  trigger history are carried/restored.
- Phase 5 owns combat-preview meaning. Phase 2 confirms the stored map/preview handoff and repeats the Phase 1
  caveat that startup publication currently validates baseline difficulty only.
- Final should examine whether C02 and C04 share one progression-invariant implementation seam while preserving
  their separate correction/refactor work types. C01 and deferred S05 must remain separate unless a cross-player
  ownership policy is approved.
