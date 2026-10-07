# P05 Successor Currentness Audit 004

Verdict: `Complete / High — current / expected P08 bounded-log and accumulator optimization`

## Identity and Trigger

- Record: `ATR-CRAR-C2-P05-GEX20-004`; predecessor record `_003` remains the P07 currentness basis.
- P08 optimizes encoded-length accounting, cache fields, ordinary reserve size and result accumulation, and replaces the
  old unbounded `Vec` field shape with a bounded COW entry wrapper.

## Direct Current Evidence

- All count/encoded-byte/time caps remain exact; old/future cursor behavior and newest-tail query identity are covered
  by the 8/8 release event-log selector.
- Cap-scale last-10 query p95 is 0.212 µs versus 0.202 µs at 1,024 entries; both return exactly ten entries and respect
  the `8,192 / 4 MiB / 180 s` limits.
- Result-statistics release selector passes 7/7. Representative outputs remain byte/hash-identical and concurrent
  retained memory remains 4,196,928 bytes, no worse than P01.
- Manifest target `CRAR-LEGACY-008` now asserts the old `Vec` field absent and binds `BattleEventEntries` as replacement;
  verifier errors are zero. History/result valid and invalid API probes retain their expected outcomes.

## Disposition

P05 remains `Complete / High`. Diagnostic history is bounded and separate from durable result/continuation authority;
COW changes storage cost, not cursor, eviction, result or serialization meaning.
