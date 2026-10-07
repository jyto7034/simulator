# Persistence and Restore Boundary Map

- 기준선: `B-001`
- 상태: 최종

## PS-001. In-memory run checkpoint

- capture: completed SavePoint → `RunCheckpointPayload::capture`
- durable scope: run map/progression/facility/encounter/session previews/attempt/recruitment/record, checkpoint-consumed VN
  triggers, economy/roster/fragments and UUID manager
- explicit transient exclusions: active/node session, active battle, pending sweep, recovery restriction, live VN session;
  player-profile and run peak remain monotonic outside checkpoint rewind
- load: validate complete payload → transaction wrapper → restore → increment sideband load count → map projection

## PS-002. Active-run export/continue

- envelope: JSON version 4, `deny_unknown_fields`, no migration; non-current/missing version rejected
- bytes: `core_payload_json`
- host sideband: `loads_used: u8`, `run_peak_certified_power: u64`
- bootstrap-only: transport player ID; core run seed comes from validated checkpoint and UUID manager
- continue: deserialize header/document → validate graph → assign/restore → `MapState`

## PS-003. Existing behavioral evidence

- E-002/E-011: strict field/version and malformed graph rejection, no-mutation assertions in mutation matrix
- writer payload → continue → recapture JSON semantic equality
- explicit checkpoint load restores visible state/anchor but preserves load count/run peak semantics
- gate replay after load reproduces the same next anchor and map
- gap: generic `continue(state, command sequence)` equivalence is not exhaustively proven (GAP-003)

## Adjacent durable store

## PS-004. Adjacent active-run file store (`ADJ-PERSIST-001`)

- strict/checksummed outer V4 envelope; core payload base64 plus monotonic loads/peak
- single-writer exclusive file lock
- current + previous-generation backup, checksum/version validation and backup repair
- synced temp → platform atomic replace → directory sync
- invalidation tombstone synced before cleanup; next publish does not reuse invalidated backup
- actor transaction: capture core rollback/checkpoint-before → core command → changed checkpoint publish/terminal invalidate →
  restore core on ordinary persistence failure
- standard completion: profile journal/publication → active-run invalidate → journal cleanup/recovery
- evidence: E-012. Adjacent store tests were inspected but not executed by core baseline.
