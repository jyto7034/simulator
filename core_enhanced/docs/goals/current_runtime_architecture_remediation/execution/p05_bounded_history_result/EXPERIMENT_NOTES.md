# P05 Bounded History and Result Lifetime Experiment Notes

## Intake Decisions

- “First cap reached” means every append enforces all caps and evicts the oldest entries until the retained window is
  within each inclusive limit. Exact limit is valid; limit+1 triggers deterministic eviction/drop.
- Encoded bytes means the compact JSON encoding of the retained entry array, including commas. Version/floor metadata
  is control state and not part of the 4 MiB event payload budget.
- Cursor `after_seq == floor_seq` is valid because every later sequence is retained. `after_seq < floor_seq` requires a
  snapshot resync. `after_seq > latest_seq` remains invalid future input.
- History is optional diagnostic state. Its append/encoding/accounting failure must never abort, undo or alter gameplay
  authority. Query errors may reject a command before application commit; they do not make history authoritative.
- Result statistics are authoritative aggregates, not diagnostic history. Incremental observation must reproduce the
  existing collector exactly for the current event corpus before the terminal log is allowed to expire.
- Test/debug event log inspection may remain behind the established capability to preserve detailed deterministic
  oracles. Default-release result and continuation types must not contain or expose the log.
- Result/event public transport redesign remains P07. P05 may add the one typed Core resync error required by its policy
  but must not introduce external wire/version compatibility claims.

## Evidence Rules

- Length-only tests are insufficient: count, byte and time exact-cap/cap+1 cases must independently force the first cap.
- Query tests must cover empty, floor-1/floor/latest/future, repeated reads and no-gap/no-duplicate sequence identity.
- A result equality test must compare bounded/expired/absent diagnostic history against the prior full-log collector for
  duration, kills, employee metrics, MVP, participant output and continuation decisions.
- Static field absence alone is insufficient: actual live kernel events must flow through the bounded window and actual
  world result transition must discard production diagnostic history.
- GEX-20 audits the frozen GEX-10 target in a separate pass. Any completion defect returns to correction GEX-10.

## GEX-10 Correction Notes

- Recorded event time is not guaranteed to be sequence-monotonic for every legal causal helper. Retention therefore
  uses the maximum and minimum time among retained entries; it does not reject or clear solely for time regression.
- Cursor validity is checked before a Battle-owned mutation whenever an active battle exists. Behaviors that operate
  after active battle teardown (for example result completion) must not be rejected merely because no active session
  exists.
- Preflight and response projection have distinct duties: an already expired/future client basis rejects before
  mutation, while eviction caused by the accepted mutation itself produces a successful typed resync checkpoint.
- The result accumulator is included in atomic debug snapshots so rollback equality covers all newly authoritative
  state, not only retained diagnostics and sequence counters.
- Compact byte cap intentionally covers the encoded `entries` array only. Floor/latest/version/accounting metadata is
  bounded control state and not charged to the 4 MiB presentation-event budget.
