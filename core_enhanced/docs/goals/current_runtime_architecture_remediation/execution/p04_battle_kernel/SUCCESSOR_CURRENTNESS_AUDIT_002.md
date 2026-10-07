# P04 Successor Currentness Audit 002

Verdict: `Complete / High — current / expected P05 private-kernel lifetime delta`

## Identity and Trigger

- Record: `ATR-CRAR-C2-P04-GEX20-002`; completion basis remains frozen
  `ATR-CRAR-C2-P04-GEX10-001`.
- Triggering successor: P05 moved diagnostic history to a bounded window and authoritative result statistics to a
  private incremental aggregate.
- Method: exact comparison of all 31 P04 frozen members plus fresh private-kernel probes and full Core validation.

## Direct Current Evidence

- Frozen comparison is 28/31 byte-identical. Expected deltas are only `battle/core/mod.rs`, `battle/core/sim.rs` and
  `docs/core_runtime_contract.md`.
- The Core deltas add a crate-private result accumulator, include it in rollback diagnostics, finish result stats from
  it and capability-gate the diagnostic log. They do not reopen `game::battle::core`, expose unit/kernel fields, add an
  event injection port, or change finite coordinate/radius/speed/time construction.
- All P04 finite types, build validation, movement/order members and skill-target contract retain exact frozen bytes.
  Its live release probe succeeds; the standalone raw kernel import still fails with `E0603`.
- Default release is warning-free and the complete Core oracle is 1,143/1,143.

## Disposition

P04 remains `Complete / High`. P05 changes the expected private-kernel result/diagnostic lifetime only; privacy,
finite rejection, ordering and sole-producer guarantees remain current. No external compatibility claim is added.
