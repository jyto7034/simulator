# Core Enhanced Refactoring Audit Phase 02 — Boundary Audit

Audit Date: 2026-08-07
Status: investigation complete; phase-exit audit pending
Execution Mode: investigation_only
Production Mutation: none authorized or performed

## Scope Reconciliation

| Surface | Current evidence | Phase disposition |
| --- | --- | --- |
| Root run authority | `world.rs`, `world/state.rs`, `world/{node_flow,active_run}.rs` | primary |
| Map authority | seven files under `src/game/map/**`; direct world consumers in `node_flow.rs`, `map_encounters.rs`, `facility.rs` | primary |
| Run identity/RNG | `GameCore::run_seed`, `RunProgression::run_seed`, `UuidManager`, `determinism.rs` | primary |
| Checkpoint/rollback | `RunCheckpointState/Payload`, SavePoint capture/load, active-run export/import, non-battle rollback | primary |
| Typed published inputs | `RunPolicyData`, `GameplayMapCatalogs`, facility templates/pools/assignment profiles | confirmed Phase 1 handoff |
| Host persistence | `game_server` player/load-balance actors, `ActiveRunStore`, persistence helpers/settings/startup | confirmed supporting producer/consumer |
| Core focused tests | map (25), facility (11), map flow (40), node sessions (18), start/snapshot (23), UUID (2), transaction filter (4) | primary safety net |
| Server focused tests | 13 active-run-matching tests | supporting safety net |
| Canonical | `core_runtime_contract.md`, `game_rulebook.md`, `data_loading_contract.md`, routed by `README.md` | current durable contract |
| Economy/session/combat internals | state is carried only where checkpoint/run lifecycle requires it | assigned to later Phases |
| Unity and other external consumers | not inspected | out-of-scope/unverified |

The physical `world` module is not one refactor boundary. This Phase follows only the state that constructs a run,
fixes a generated map, advances node/Floor progress, captures/restores the last SavePoint, or crosses the host
active-run seam.

## Run Creation, Identity, And Published-Data Trace

1. `PlayerGameSession` derives an initial process-local seed from authenticated `player_id` and the server's
   random `current_run_id`, then `LoadBalanceActor` constructs `GameCore` with the published
   `Arc<GameDataBase>`.
2. `StartNewGame` initializes the transport player and opens the mode-specific HQ briefing. It deliberately has
   no `RunState` or map yet.
3. After the briefing, `SelectStarterEmployees` consumes `pending_game_mode`, creates `RunProgression` from
   `GameCore::run_seed`, initializes run-scoped research/recruitment/omen state, and calls
   `generate_current_floor_map()`.
4. Floor generation reads the current certified power, `RunPolicyData`, facility schedule/pool/template catalogs,
   persisted `FacilityRunState`, mode and Floor index. It fixes the Floor anchor, full facility assignment,
   deterministic node IDs/content, encounters/difficulty, and next facility-pool state before installing
   `RunState`.
5. `FacilityMapGenerator` receives the run seed and Floor index directly. `RunProgression::current_floor_seed()`
   expresses the same deterministic namespace but has no workspace caller; generation persists the completed map
   rather than regenerating it on reconnect/checkpoint load.
6. `UuidManager` keeps independent namespace counters for owned equipment, employees, VN sessions and battles.
   SavePoint load rewinds checkpoint-owned streams while preserving the VN-session counter monotonically.

Phase 1 proves that the typed run policy and map catalogs are published through one strict loader. Phase 2 does
not rely on Phase 1's broader, currently unimplemented claim that startup preview validation exercises every
manifested difficulty stage.

## Map, Node, And Floor Mutation Trace

- `RunMap` stores template identity/version, the fixed node assignment, typed terminals and node state. Authored
  template connections remain topology authority; no runtime edge list or coordinate fallback is stored.
- `MapProgression` stores current, available and completed IDs. Selection admission calls
  `selectable_node_ids()` against the current map and authored template, so Unity/node array order is not
  authority.
- `SelectMapNode` creates only a `NodeSession`/preview and moves to `NodeConfirm`; it does not move the current
  progression position.
- Confirmed entry calls `MapProgression::enter_node()`. Completion stages cloned map/progression, validates the
  result, and commits through `with_state_transaction()` or a domain-specific staged transaction.
- Nonterminal completion marks the node complete and derives reachable successors through completed transit
  nodes and directed authored connections.
- Terminal completion either stages the next Floor (preserving run-scoped research, recruitment, facility pool
  history and other approved run state while clearing Floor-local caches) or stages `RunComplete`.
- Gate confirm and interactive completion use different entry seams but share the same `RunProgression`, map
  generator and carry-forward rules. Gate is not a SavePoint.
- `available_node_ids` is also mirrored by `MapNodeState::Available`; selection and snapshot recompute the
  selectable set rather than trusting only the stored vector. `boss_omen` and admin fixtures also mutate these
  public fields directly. This is Candidate C04, while removing the serialized field is deferred as a migration.

## Checkpoint, Retry, And Rollback Trace

### Process-local SavePoint

1. SavePoint completion stages recovery, research delivery and node completion.
2. Only after the final state is approved does `save_run_checkpoint()` clone 13 `RunState` fields plus eight
   root-owned fields into `RunCheckpointPayload`.
3. `loads_used` is outside the payload. A SavePoint overwrite replaces the payload without reducing it.
4. `LoadRunCheckpoint` admits only map/confirm/failure-decision states, clones the payload, validates it, checks
   the three-load ceiling, restores it in a state transaction, then increments `loads_used` exactly once.
5. Restore clears node content/session/active battle and returns to `ViewingMap`. It restores checkpoint-owned
   model/anchor and UUID streams but not run peak, transport identity, encounter compendium or VN-session
   monotonic identity.

### Strict active-run payload

- Core export serializes only the last completed SavePoint as strict JSON version 3 and returns payload bytes plus
  host metadata `loads_used` and `run_peak_certified_power`.
- Continue first reads a small version header, rejects every non-V3 version, deserializes the strict V3 body,
  validates it, injects the current transport player, restores the run seed and requests the current map.
- Continue is not a checkpoint load and does not increment `loads_used`.
- The validator checks current power model/anchor, manifested difficulty, facility template/slot/terminal
  contract, progression references to existing nodes, recruitment ownership and monolith state.
- It does not validate equality among `GameCore::run_seed`, `RunProgression::run_seed` and the private serialized
  `UuidManager::run_seed`, and the UUID restore path uses only `debug_assert_eq!`. It also does not prove that
  progression lists/node states equal the graph-derived progression. This is Candidate C02.

### Rollback ownership

- `with_state_transaction()` clones non-battle state and restores it on an operation error while union-merging
  the separate encounter compendium.
- The server captures the same non-battle rollback before a command that may need durable publication.
  Persistence failure restores run-local state, max-merges the run peak, preserves the compendium union and does
  not report command success.
- Battle entry snapshot, ordinary world transaction and host durable rollback have different lifetimes. Similar
  field families do not justify merging them.

## Server Active-Run Persistence Trace

1. Startup resolves an OS application-data path, opens one local-install `ActiveRunStore`, obtains an exclusive
   process-lifetime file lock, and shares the store through `LoadBalanceActor` to player actors.
2. Startup does not eagerly install a run into Core. A title snapshot calls `load()` to expose `ContinueRun` or
   `ResetActiveRun`; Continue performs the Core import.
3. The host envelope is strict version 3. It Base64-encodes opaque Core V3 bytes, stores only the two monotonic
   fields, hashes the canonical unsigned envelope with SHA-256, and rejects malformed/unsupported/checksum data.
4. Publish writes and fsyncs a temp file. When current is valid, it writes the previous payload as rolling backup
   after max-merging the incoming monotonic metadata into that backup, replaces current, and fsyncs the directory.
5. Current missing/corrupt plus valid backup restores backup to current and exposes a recovery status. Two invalid
   copies remain an explicit system issue. Tombstone invalidation prevents a terminal/old run from reappearing.
6. The player actor compares Core checkpoint export before/after a successful non-continue command. A terminal or
   confirmed replacement invalidates; a changed checkpoint publishes; a host error invokes Core rollback.
7. Encounter-compendium persistence is a separate transaction and is not rolled back with the active run.

The store max-merges monotonic metadata only into the backup. A valid current with `loads_used = 2` followed by a
new valid publish carrying `loads_used = 1` makes the lower value the new current; the same applies to run peak.
Existing tests protect fallback-to-backup but do not cover lower incoming current. This violates the current
run-scoped monotonic contract and is Candidate C01.

## Ownership And Representation Findings

| Representation | Role | Authority classification |
| --- | --- | --- |
| `GameCoreState` | in-memory root and current command mutation | authoritative runtime state |
| `RunState` | fixed current map/progression and run/Floor state | authoritative active run |
| `MapProgression` + `RunMap` | navigation/progression state plus fixed map | joint runtime invariant; C04 targets encapsulation |
| `RunCheckpointPayload` | last completed SavePoint snapshot | derived persisted/restorable representation |
| `ActiveRunCorePayloadV3` | strict Core version wrapper | Core serialization authority |
| `ActiveRunEnvelope` | checksum/storage/host monotonic wrapper | server storage authority |
| rolling backup/tombstone | explicit host recovery/lifetime state | intentional persistence lifecycle |
| snapshot/map DTOs | Core-derived current projection | consumer view, not policy authority |

`RunState` and `RunCheckpointPayload` repeat 13 run fields; capture and restore then repeat the full field mapping in
opposite directions. The current mapping is correct and round-trip tests pass, but future field additions have no
mechanical completeness gate. Candidate C03 consolidates this mapping without changing V3 JSON.

## Canonical Alignment

- Current canonical and code agree on Core V3 plus host V3, strict rejection of V2/missing versions, no migration,
  last-completed-SavePoint semantics, three explicit loads, continue not consuming a load, host monotonic metadata,
  current/backup recovery and tombstone invalidation.
- Code and canonical agree that fixed map assignment and facility pool state are persisted and not regenerated.
- Canonical says run peak and load count never decrease. Backup recovery enforces this, but ordinary store publish
  does not independently enforce a lower incoming value; C01 records the mismatch.
- Canonical describes strict facility and reference validation but does not decide simultaneous ownership of the
  local-install singleton by different transport-player actors. That signal is deferred rather than silently
  assigning a lease policy.

## Consumer And Compatibility Classification

| Consumer or seam | Classification | Evidence |
| --- | --- | --- |
| `game_server` startup/store/player actor | confirmed workspace consumer/producer | direct constructors, load/publish/invalidate/import calls |
| load-balance actor | confirmed shared-store distributor | one `Arc<Mutex<ActiveRunStore>>` cloned to player actors |
| Core public active-run APIs | confirmed server consumer; other Rust consumers unknown | workspace calls plus public methods |
| local active-run JSON files | confirmed host runtime documents | real-file store tests |
| admin commands | confirmed development seam; persistence policy deferred | gated server handler bypasses ordinary active-run transaction wrapper |
| Unity | explicitly excluded/unverified | Master/Phase scope |
| other external Rust/save tools | unknown | no deployment inventory |

Core and host version numbers are separate layers even though both are currently 3. Combining them into one
constant would erase independent ownership; accepting V2 or adding a migration reader requires a separate policy
Goal.

## Focused Safety Net And Gaps

Executed runner filters produced 136 passing executions representing 135 distinct named tests; one transaction
round-trip test was intentionally selected by two filters. No selected test failed or was ignored.

Strong current coverage includes deterministic map generation, authored topology/navigation, node/Floor
transitions, SavePoint capture/load count, visible/anchor restoration, run peak preservation, exact shop and
recruitment payload state, V2/missing/tactical-field rejection, monolith typed state, transaction rollback,
checksum/version failure, backup recovery, tombstone behavior, explicit replacement and reset.

Missing characterization evidence:

- lower incoming `loads_used`/run peak cannot become the new valid current;
- two player actors cannot branch or overwrite the same local-install active run, or an approved conflict policy;
- checkpoint `run_seed` equals UUID-manager seed and the active Core seed;
- imported current/available/completed progression is semantically consistent with node state and authored graph;
- field parity between `RunState`, V3 checkpoint capture and V3 restore is mechanically complete;
- production admin mutation and durable active-run persistence have an approved relationship.

No focused file contains `#[ignore]`. Runner-reported filtered counts are nonmatching tests, not skips.

