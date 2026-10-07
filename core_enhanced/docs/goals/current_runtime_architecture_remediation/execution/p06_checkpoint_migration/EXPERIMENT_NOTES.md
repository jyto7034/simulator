# P06 Checkpoint Migration Experiment Notes

## Intake Decisions

- “Canonical V4 corpus” means a test-owned exact V4 document built from a completed SavePoint payload without invoking
  the production V5 writer. Its root keys, 19-field checkpoint shape, semantic assertions and canonical SHA identity
  form the migration oracle. A test fixture builder is not a production V4 writer or dual-write path.
- V4 is imported exactly as it existed: `{version, checkpoint}` with strict unknown-field rejection. It does not gain a
  synthetic checksum, aliases or fallback defaults.
- V5 checksum covers canonical JSON for the checkpoint value, not incidental envelope formatting. Object keys sort
  lexically at every depth, arrays preserve order, and scalar representation uses `serde_json`. This gives semantic
  corruption detection without depending on `HashMap` randomized iteration.
- SHA-256 is encoded as exactly 64 lowercase hexadecimal characters. Wrong length, non-hex/case variants and mismatch
  all reject as invalid static data before authority assignment.
- Version dispatch is the only compatibility choice. A document is never tried as another schema after a parse error,
  and unsupported versions do not fall back to the current version.
- The removal gate remains closed in code and has a direct negative test. It records the frozen requirement that V4
  reader removal needs a separate migration-certification decision; this Goal does not create that decision.
- Continue/load failure equality must include run/player/checkpoint/loads/run-power/seed/UUID authority, not merely
  `state.run.is_none()`. The P03 draft executor remains the sole publication mechanism.
- P05 diagnostic history remains outside `RunCheckpointPayload`. P06 tests the absence but does not change history
  types, caps, cursor semantics or result authority.
- P07 owns transport-neutral public projection architecture and P08 owns measured checkpoint cost/performance PASS.
  P06 may record encoded bytes and correctness but may not optimize or claim a performance result.

## Evidence Rules

- A version-number assertion alone is insufficient: V4 import must continue in a fresh actual Core and be compared
  against V5 continuation through restored state and next decisions.
- A writer-produced “legacy” document is insufficient because it couples both sides of the differential to the new
  implementation. The V4 corpus constructor and expected canonical digest must remain independent of V5 serialization.
- A checksum test must alter a semantically meaningful nested checkpoint value while retaining the old checksum and
  prove exact pre-mutation rejection. Merely corrupting JSON syntax tests only the parser.
- Map-order stability must be direct: an equivalent V5 document with recursively reordered object members must verify
  and continue with the same semantics.
- Strictness must cover missing and extra envelope/checkpoint fields for both supported versions, plus an unsupported
  version. No broad `Value` conversion may silently drop an unknown field before the strict typed parse.
- GEX-20 will audit a frozen target in a separate read-only pass. Any completion defect returns to correction GEX-10;
  it is not silently edited into a passing verdict.

## GEX-10 Findings

- Canonicalizing a typed `serde_json::Value` is sufficient only because objects are recursively sorted explicitly;
  relying on `serde_json::Map` or source `HashMap` iteration would not be an acceptable format guarantee.
- Checksum comparison also enforces exactly 64 lowercase hexadecimal characters. Uppercase is rejected even when it
  represents the same digest, keeping one canonical spelling.
- Strict typed parsing must occur before checksum comparison. An extra checkpoint field is rejected as a schema error
  instead of being dropped by a broad `Value` conversion and then validating a reduced object.
- The live V4 reader is the compatibility mechanism; V4 is not rewritten in place and has no checksum. Its test corpus
  builder is deliberately confined to test code and has no production export entry point.
- The application draft already supplies complete rollback. P06 retains that owner and proves codec/graph failures
  happen before assignment; it does not introduce a second persistence transaction or durable-intent path.
- Existing tests that intentionally mutate a valid current checkpoint must recompute the V5 checksum when they intend
  to exercise downstream graph/cache semantics. Tests for raw corruption intentionally keep the stale checksum.
- The inventory has no arbitrary total-cap policy. The applicable collection boundary is its persisted reservation
  high-water: exactly 50 occupied slots with reservation 50 is valid; 51 with stale reservation is an explicit graph
  rejection. No new checkpoint byte-size cap was invented before P08 measurement.
- Adding the direct checksum dependency changes standalone probe locks even though their source APIs are untouched.
  Currentness treats those as exact dependency-resolution deltas and reruns every probe.
- P08 remains the owner of serialization/checkpoint timing, allocation and retained-memory acceptance. P06 correctness
  tests and encoded bytes are not a performance conclusion.
