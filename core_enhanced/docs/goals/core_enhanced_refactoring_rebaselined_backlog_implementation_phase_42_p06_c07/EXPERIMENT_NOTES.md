# Phase 42 CRA-P06-C07 Experiment Notes

Status: complete
Implementation Complete: true

## Structural Decision

- This is one battle-log API/debug-boundary cleanup, not durable battle history or replay work.
- Historical method/caller names locate candidates but do not prove current use, output or public consumers.
- Target ownership keeps in-memory/serde content on the data type and filesystem lifetime/error policy at the real
  explicit debug/trace boundary.

## Direct Dependency

- Phase 41 `Complete / High` is the exact linear predecessor; its audit hash is `b800f55a...`.
- Closed event ordering/result atomicity and server transport behavior remain inherited invariants.
- Phase 43 targeting/range and Phase 44 package/dependency work remain separate.

## Policy And Compatibility

- The user approved all remaining registered cleanup, including verified public/package breaking removal without a
  shim after each candidate's inventory and behavior guards.
- Approval is not authority to change event-log JSON, compression, trace failure, persistence or wire meaning.
- Any real readback/durable/external consumer is evidence to retain or stop, not to add fallback compatibility.

## Dirty Worktree

- Pre-Child expanded status is 590 rows at `21f63c04...`; existing rows are user/completed-campaign work.
- Phase 42 path did not exist. The three initialized documents are the only expected pre-activation additions.
- Candidate carriers may already be dirty; current paths/hashes/diffs remain unopened until sole activation and
  prospective fingerprinting.
- Protected marker `game_server/logs/app.log.2026-08-09` must remain 57 bytes at `614d2fbd...`.

## Current Evidence Gate

After sole activation, filename-only discovery fixes prospective event-log/world-trace/server/test/consumer hashes
before content inspection. Current discovery then reconstructs every method, visibility, caller, file path, error,
serde/compression behavior and external consumer before any cleanup.

## Current Disposition

- `BattleEventLog` retains public versioned serde and compact `to_json_string`; server gzip transport consumes only
  that compact method.
- `write_json`, `read_json` and `to_pretty_json_string` have no current caller. `write_pretty_json` has exactly three
  test-only debug-output callers and no production consumer.
- Production file ownership already lives in private `world::combat_trace`, which directly serializes its richer
  trace envelope and is best-effort at the authoritative result transition.
- Standard serde round trip and exact server compression content are now directly tested with the old API present;
  trace success/failure controls are already green.
- User-approved public cleanup therefore applies to all four general file/pretty helpers. The three test helpers must
  perform standard pretty serde locally; no replacement public wrapper or durable readback is introduced.
- Cleanup is implemented: the four public file/pretty helpers are absent and all three test-only debug writers use
  local standard pretty serde. Compact JSON, public versioned serde, server compression and production trace policy
  are unchanged.
- Focused serde/compression/trace/writer/finished-result controls pass. Core debug/release all-target suites each pass
  1,060 tests; server debug/release suites each pass 63. Workspace checks, normal Clippy, fmt and diff checks pass.
- The implementation is ready for independent rubric review; status remains active until that audit artifact exists.
- The separate audit passes all 12 conditions and returns `Complete / High`, fit `High`, improvement class `none`.
  Phase 42 is complete; Phase 43 may become ready only under the Master transition gate.

## Out-Of-Scope Handling

Do not add replay/history, change event-log schema/order, absorb range/package cleanup or modify trace policy.
Record independent findings as later-Phase or out-of-scope evidence only.
