# Core Enhanced Refactoring Audit Phase 02 — Experiment Notes

Status: complete

## Boundary decision

- Authority family: `GameCoreState`/`RunState`, run identity/RNG, map/floor/node progression, checkpoint and
  host active-run persistence seam.
- Direct predecessor: Phase 1 `Complete / High`.
- Published typed data is a confirmed input; Phase 1 loading candidates remain separate.
- Roster/economy, Event/VN session and battle internals retain independent later-Phase ownership.

## Source-of-truth distinctions

- Core owns gameplay run/map/checkpoint meaning and authoritative state mutation.
- Server may own storage envelope, file/checksum/backup/locking/atomic-write lifetime.
- A checkpoint/snapshot/persistence payload is not automatically a second authority; the audit must trace its
  producer and restore mutation before classifying it.
- Goal/history claims do not override current runtime/store behavior or current canonical.

## Phase 1 handoff caveat

All current 23 live RON files are included and strict publication is confirmed. However, generated preview
startup validation exercises baseline difficulty only despite a broader canonical claim. Phase 2 can use typed
map/run data but must not claim every manifested difficulty was startup-generated.

## Dirty worktree precautions

- Repository baseline remains start commit `f318414fb0156a67e04c5479000c2a2683fc0c09` plus current dirty tree.
- Core source/test/docs, live data and direct server persistence paths have existing user changes.
- Only Master/Phase audit artifacts are audit-owned.
- Already modified production files require content hashes, not status-set comparison alone, to prove no mutation.

## External systems and consumers

- `game_server` active-run store/bootstrap is confirmed supporting evidence and may be read.
- Unity restore/runtime is explicitly out-of-scope/unverified.
- Other public Rust/save consumers are unknown unless current workspace/canonical proves them.
- No compatibility or save migration decision is delegated by CRA-13 naming authority.

## Initial rejected assumptions

- Do not call server persistence the gameplay authority merely because it writes the durable file.
- Do not call Core the storage-transaction authority merely because it produces the payload.
- Do not merge roster/economy/session/combat candidates because their state is nested in one root struct.
- Do not classify backup/default/retry as fallback debt until success/failure lifetime and canonical intent are
  traced.

## Next safe work

1. Re-read this Phase's three required documents and direct Phase 1/Phase 0 dependency records.
2. Freeze scoped dirty/content baseline for Core run/map/checkpoint and server active-run paths.
3. Inventory command→state→map→checkpoint→store/bootstrap paths and focused tests.
4. Record complete candidate/disposition artifacts and independent completion audit.

## Investigation conclusions

- Run creation authority is split intentionally by lifetime: server supplies the initial seed and published data;
  Core creates `RunProgression` only after briefing/starter selection; the fixed map and pool state are persisted.
- `RunMap` plus authored template owns topology. Snapshot and admission derive selectability; Unity is not a graph
  authority.
- SavePoint payload owns model/anchor generation. `loads_used` and run peak remain host/run monotonic metadata.
- Host envelope and Core payload are independent strict V3 schemas; V2/missing-field migration is explicitly absent.
- Current/backup/tombstone recovery is approved explicit behavior, not fallback debt.
- The store protects backup monotonicity but not a lower new current publish: C01.
- Checkpoint import validates many current-catalog contracts but leaves seed/progression cross-field invariants:
  C02.
- Manual run↔payload field mapping and distributed progression mutation are C03/C04 respectively.
- Cross-player ownership of the singleton store needs policy; a mutex is serialization, not a logical lease.

## Candidate separation guard

- C01 must not be used to choose which actor's checkpoint payload wins; it only enforces already-approved host
  metadata high-water behavior.
- C02 is a correction for invalid current-version input. It must fail closed, not repair or regenerate.
- C03 preserves the exact Core V3 shape and explicit exclusions.
- C04 preserves all current navigation and serialized fields; field removal remains migration work.
- Final may share implementation seams only after keeping these work types and policy boundaries visible.
