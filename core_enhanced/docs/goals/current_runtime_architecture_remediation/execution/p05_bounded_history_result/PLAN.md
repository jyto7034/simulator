# P05 Bounded History and Result Lifetime Execution Plan

상태: `GEX-00 passed / GEX-10 exit criteria met; target freeze pending`

## Current Execution Identity

- Handoff / root target: `HA-CRAR-P2G60-002` / full `GS-CRAR-P2G50-002` goal-run.
- Selected member: `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P05-BOUNDED-HISTORY-RESULT)`.
- Phase / execution node: `PHASE-CRAR-C2-05-BOUNDED-HISTORY-RESULT` / `WP-CRAR-C2-06`.
- Goal Quality Contract: `2`; durable seam `IVA-C2-06`.
- Policy / decision: `PER-CRAR-009-R6-CODE-REVIEWED` / `DR-CRAR-008-R2`; realization decision
  `TD-CRAR-018`.
- Current pass: separated GEX-00 intake/design only. No P05 production mutation occurred before this document set.
- Authorization: Core-only mutation and validation. `game_server`, Unity, reconnect transport, external receipt/store,
  archive lifecycle and external consumers are excluded.

## Direct Dependencies and Currentness

- P01 `Complete / High`, current on `ATR-CRAR-C2-P01-GEX20-004`.
- P03 `Complete / High`, current on `ATR-CRAR-C2-P03-GEX20-002`.
- P04 `Complete / High`, frozen on `ATR-CRAR-C2-P04-GEX10-001`; its private finite kernel is current at P05 intake.
- P02 remains an inherited data prerequisite but is not a Goal-declared direct P05 dependency; live authoring schema is
  unchanged and the P05 `live_data` dimension is policy-approved N/A.

## Objective and Exact Meaning

- Keep the active Core diagnostic event window within all three limits: at most `8,192` retained events, at most
  `4 MiB` canonical compact-JSON encoded entry-array bytes, and at most `180,000 ms` of battle-time span. Append removes
  the oldest entries until every limit holds; an individually oversized/unencodable diagnostic entry is not retained.
- `180 battle-seconds` is a moving history-window limit, not a battle timeout. Battle simulation and authoritative
  results continue after any history eviction or diagnostic overflow.
- Track an explicit cursor floor and latest sequence independently of retained entries. A cursor below the floor returns
  a typed snapshot-resync requirement; a cursor above latest remains the existing typed invalid-future error; an
  in-window cursor returns the exact indexed delta with no gap or duplicate.
- Separate authoritative result statistics from diagnostic retention by observing events into a bounded aggregate as
  they occur. Final result/continuation state consumes that aggregate and participant state, never the retained history.
- Production `BattleResult`, durable `CombatBattleState`, checkpoint/continuation and public combat result state must
  not retain the diagnostic log. Existing test/debug event-log inspection may remain a gated non-production diagnostic
  capability.

## GEX-00 Current Inventory

- `BattleEventLog.entries` is an unbounded `Vec`; append is a direct push. `entries_after_seq` linearly scans and clones
  the entire matching tail. It has no count/byte/time limit, cursor floor, latest-sequence metadata or overflow state.
- `ActiveBattleSession::latest_event_log_seq` derives latest from the retained vector and validates only future cursors;
  it cannot distinguish a valid in-window cursor from a cursor whose events were evicted.
- `BattleCore::finish_battle` clones the entire log into `BattleResult`. World result conversion reads the log to derive
  statistics and then stores it again in `CombatBattleState`; a production-public attachment clones it once more.
- Result statistics are currently computed in two passes over the complete terminal log. Evicting history without a
  separate incremental authority would change duration, kill, damage, deployment and MVP results and is forbidden.
- Checkpoint continuation does not intentionally require the battle log, but current result structures make that
  independence implicit rather than type-enforced.

## Approved GEX-10 Design

1. Convert `BattleEventLog` into a bounded active window with private accounting, explicit `floor_seq`, `latest_seq`,
   latest battle time and sequence-exhaustion state. Keep serialized presentation entries compatible as a JSON array.
2. Centralize append in a diagnostic-only method. Account each entry's compact JSON bytes and comma overhead; evict
   oldest entries for count/byte/time, use checked/saturating arithmetic, and fail closed to an empty/resync-required
   window on accounting/encoding or sequence overflow without returning a gameplay error.
3. Replace filter-from-start lookup with sequence `partition_point`, so lookup work is bounded by the 8,192-entry
   window. Return an exact delta only when `floor_seq <= after_seq <= latest_seq`.
4. Add a typed `BattleSnapshotResyncRequired { requested, oldest_available_after_seq, latest }` Core error and stable
   error code/projection. Preserve `InvalidBattleResyncSeq` for future cursors.
5. Refactor result-stat collection into a cloneable incremental accumulator observed before diagnostic append. Finish
   it from participant state and policy, and place the completed `BattleResultStatsDto` directly in `BattleResult`.
6. Remove diagnostic-log retention from default-release `BattleResult` and `CombatBattleState`; gate test/debug fixture
   diagnostics, combat trace export and attachment behind the existing non-default capability.
7. Add cap±1, individually oversized, time-window, empty, future/old/in-window cursor, sequence overflow, repeated query,
   diagnostic-absence and result/continuation equality tests. Compare bounded and no-retention results against the
   existing full-log collector oracle.
8. Update the canonical runtime contract to state all three caps, cursor semantics, diagnostic failure independence and
   the non-timeout meaning. Do not change checkpoint version/schema (P06), public projection architecture (P07), or
   claim latency/memory PASS (P08).

## Completion Contract

- retained event count `<= 8,192`; encoded entry-array bytes `<= 4 MiB`; retained time span `<= 180,000 ms`
- exact-cap allowed and cap+1 evicts/drops deterministically; no unbounded fallback or second full history
- in-window query gap/duplicate `0`, identical repeated projection, and bounded indexed lookup
- old cursor returns typed snapshot-resync requirement with floor/latest; future cursor keeps typed invalid semantics
- result statistics, winner/outcome, participant state and next Core decisions are identical with full, expired, absent,
  oversized or sequence-exhausted diagnostic history
- default-release result/continuation/checkpoint history retention and production attachment: `0`
- P01/P03/P04 successor-currentness records direct and `Complete / High`; full Core oracle and canonical contract pass
- external reconnect/archive/receipt/store/client compatibility and Phase-8 performance PASS claims: `0`

## Stop Conditions

- any cap or diagnostic loss changes gameplay result, continuation, reward, participant state or next decision
- correct result independence would require retaining another unbounded event list
- old-cursor semantics require a new external reconnect/protocol policy
- checkpoint format/version, P07 public-projection architecture or P08 performance acceptance must change
- predecessor evidence becomes stale without a direct successor-currentness audit
- dirty overlap cannot be isolated safely, or policy/Goal/Phase/node meaning changes

## Pre-edit Identities

- `src/game/battle/event_log.rs` `5baad222a5cbb4d953b2b5b7c07afbd7a7f52b76d3bbc2863aac1da4a766d27d`
- `src/game/battle/recording.rs` `aa2fca886c8392ac3483353a9ee17cdf94c2ce7aed07dd3276142c5f311ac12c`
- `src/game/battle/result_stats.rs` `ea6202a12362ffcc23f556eeb001bc7e125d21907c73bbecd6191966681b2762`
- `src/game/battle/types.rs` `e1e9b8d735a9edc8b20a849d696918d0cda0de822c17ba978e956ab9b3702436`
- `src/game/battle/core/mod.rs` `61d259ba66e67acf3091174d4a663d95a70ccf60accfff8f66710951ad3ca73d`
- `src/game/battle/core/sim.rs` `358986c0bed6c68bcbdbbad6937a74688ca7597d56a072bf113157f00112a271`
- `src/game/world/state.rs` current P03-frozen identity `93dd4dffd14dfcb9e9fd8872b4f63e59aff7c5b5979f3b3e6028adc8398daded`
- `docs/core_runtime_contract.md` `fb48a50bfe3ebd5b053b49556291997b812fe1fcc8cae09e919b7e29be4d8499`

These hashes were captured immediately after GEX-00 and before the first production edit. All listed source paths are
pre-existing dirty/mixed user work; P05 owns only exact bounded-history/result-lifetime hunks.

## GEX-10 Implemented Boundary

- `BattleEventLog` now retains at most 8,192 entries, 4 MiB compact entry-array bytes and 180,000 ms retained
  battle-time span. Append centrally accounts bytes, enforces every inclusive cap and drops an individually
  oversized/unencodable entry without returning a gameplay error.
- Explicit floor/latest sequence metadata survives eviction. Exact in-window lookup uses `partition_point`; old
  cursors produce typed snapshot-resync state and future cursors keep the existing typed invalid error. A command that
  passes preflight but advances the floor while executing returns a `SnapshotResyncRequired` update instead of rolling
  back the valid domain change.
- A cloneable private accumulator observes each semantic event before diagnostic append, accumulates employee/result
  metrics and finishes with participant state. Terminal outcome, participant state and result stats no longer read the
  retained window.
- Default-release `BattleResult` and `CombatBattleState` contain no event-log field. The terminal attachment DTO/API,
  trace export and retained result diagnostic are limited to test/debug/test-fixtures capability.
- The complete transaction debug snapshot includes the new aggregate. Sequence exhaustion retains no duplicate seq;
  diagnostic closure cannot stop simulation or result accumulation.
- The canonical runtime contract records all caps, cap±1 behavior, moving-window/non-timeout meaning, cursor semantics,
  result independence and default-release lifetime. Checkpoint codec/version, P07 projection architecture and P08
  performance acceptance are unchanged.

## GEX-10 Exit Evidence

- `cargo fmt --all -- --check`: exit `0`; `cargo check --release --lib`: exit `0`, warnings `0`.
- Cap/cursor/result/sequence focused oracles and typed live recovery pass; the actual long live defense route crosses
  history eviction and reaches the same terminal result.
- Valid default-release probe: embedded validated data -> public `GameCore` -> actual live private battle, exit `0`.
- Invalid default-release probe: expected exit `101`, `E0432` for the gated attachment DTO and `E0599` for its gated
  `GameCore` method.
- Fresh full Core oracle: library `1,092` plus integrations `51`, total `1,143`; failed/ignored `0`.
- P01 `ATR-CRAR-C2-P01-GEX20-005`, P03 `ATR-CRAR-C2-P03-GEX20-003` and P04
  `ATR-CRAR-C2-P04-GEX20-002` are direct `Complete / High` successor-currentness records.
