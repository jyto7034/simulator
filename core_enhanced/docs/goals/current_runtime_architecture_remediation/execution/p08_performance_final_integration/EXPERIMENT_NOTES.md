# P08 Performance and Final Integration Experiment Notes

## Intake Decisions

- The accepted baseline is the successor P01 v2 root receipt, not the stopped `001` execution artifact with the same
  filename. P08 will fail closed if the v2 receipt no longer matches its immutable source/input identity.
- Like-for-like means the P01 scenario construction, live data augmentation, seeds, 5,000 ms event-heavy advance,
  64 sessions, eight lanes, 100×10 ms steps, allocator fields and deterministic serialization identity remain exact.
  A new final harness may add samples/guards, but it may not silently make the old workload easier.
- Baseline latency observations are historical fixed evidence. P08 will retain final interleaved raw repetitions and
  use an exact one-sided rank/permutation method suitable for the small baseline corpus; the raw distribution and test
  definition are part of the receipt so a verdict is reviewable without trusting a summary ratio.
- The representative kernel seam preserves exact P01 comparability. It does not satisfy the final public projection
  obligation by itself. Separate `CoreApplication` measurements own immutable result/snapshot encoding and V5
  checkpoint cost.
- Direct heap figures are attributed only to the standalone process-global allocator observer around a named workload.
  Warm-up and live-data load remain outside sample windows where P01 placed them. Cross-thread 64-session allocation
  is valid because the scoped lanes are the only measured concurrent work in that process.
- Retained-memory-independent staging is a scaling property, not merely a low absolute number. The guard compares the
  same staged operation with small and cap-scale diagnostic history and records raw allocation/time; retained history
  may not be copied linearly into each transaction.
- Existing TD-018/019 numeric limits are policy inputs. TD-010 supplies evidence-reviewed Core engineering guards for
  history/query/snapshot/checkpoint/staging where applicable. Any revised/extra numeric threshold must be derived from
  the initial retained P08 observations and documented before it becomes a guard.
- P08 optimization begins only after the failing current measurement identifies a hotspot. Benchmark-only behavior in
  production, relaxed validation, output normalization, fixture-name branches, fallback and expectation weakening are
  forbidden.
- `WP-CRAR-C2-10` is validation-only and cannot repair a performance failure. Any source correction returns to
  `WP-CRAR-C2-09`; any final audit defect returns to correction GEX-10 before a new frozen target.

## Evidence Rules

- A p95 gate uses nearest-rank p95 over retained post-warm-up samples and the strict `<` relation stated by TD-019.
- Exact output means event count, serialized byte count and SHA-256 match in every sample and match P01, not merely
  within one final run.
- “No statistically supported regression” is not inferred from an absolute-cap pass. Relative and absolute gates are
  independently reported and both must pass.
- Allocation count, peak delta and retained delta are reported separately. A smaller end-retained value cannot hide a
  higher peak or a newly linear staging copy.
- History guards include functional cap identity and query scale. Serialization guards measure actual application
  documents/checkpoint bytes, not a substitute DTO assembled outside the façade.
- Final 10/10 acceptance uses current executions. Earlier P01–P07 results are traceability inputs and require direct
  currentness review after affected P08 changes; they are not substitutes for P08's final aggregate run.
- GEX-20 reads an immutable Audit Target in a separate pass. Its completion audit is excluded from that target; any
  source/evidence defect reopens correction GEX-10.

## Retained Initial Measurement and Profile Decisions

- `evidence/P08_INITIAL_PERFORMANCE_LATENCY.json` is the unoptimized 21-sample entry receipt. It failed both absolute
  workload caps: event-heavy p95 `25.735434 ms` (also a statistically supported regression, exact one-sided
  `p = 0.003861356`) and concurrent p95 `13.486429 ms`. Output parity was exact, so semantic relaxation was not an
  available remedy.
- System `perf` sampling was unavailable in the environment. P08 used source-instrumented, exact-output A/B probes and
  retained the two decisive raw receipts. Removing blocker evaluation reduced event-heavy p95 from the failing range
  to `10.553365 ms`; removing the redundant first movement sort reduced concurrent p95 to `10.589951 ms`. These
  diagnostic branches were never acceptance evidence and do not remain in production.
- The actual public entry measurement separately showed V5 export p95 `123.394 µs > 100 µs`. After the exact save-time
  V5 cache fixed that failure, the first complete staging-scale guard still showed `24.413 µs` at 128 entries versus
  `176.113 µs` at 8,192 and failed retained-history independence. The raw entry, checkpoint and pre-COW staging receipts
  are preserved rather than replaced by the green result.
- The public numeric caps were not reverse-fit to final results. Snapshot/checkpoint byte and time caps came from the
  approved Core engineering budget. The bounded-tail and staging scaling formulas were fixed in the initial verifier
  before the failing observations. The allocation scaling rule is stronger and non-arbitrary: high-history maximum
  operations, peak and retained delta may not exceed the low-history maxima at all.

## Optimization Admissibility

- Event-log changes preserve exact compact JSON and three independent caps. Deferred accounting carries a conservative
  upper bound and reconciles exact lengths before a cap can be crossed. Count, time, oversized-entry, cursor-floor and
  serde tests remain direct gates.
- Result accumulation skips construction only when the scenario has no Employee source, where the published employee
  result is necessarily empty. It does not infer from event names or benchmark fixture identity.
- Movement/blocking/targeting fast paths apply only from existing structural facts: empty static obstacles, unchanged
  body translation, no eligible blocker/enemy, deterministic streaming minimum, and positive base attack's existing
  minimum-one-damage rule. Generic/zero-base paths remain intact and focused differentials stay green.
- `BattleEventEntries(Arc<Vec<_>>)` makes root clone share immutable diagnostic history. Mutation uses `Arc::make_mut`,
  so committed battle work gets an isolated vector and rejected/read-only staging has no O(history) allocation. Custom
  serde retains the original JSON array shape; no consumer-visible wrapper field exists.
- Checkpoint payload bytes are cached only after the semantic payload validates and the canonical V5 encoder succeeds.
  The cache is private, serde-skipped and populated on SavePoint and continue. Export validates current checkpoint
  metadata, then clones exact cached bytes; a cache miss invokes the same V5 encoder.
- The cap-scale event-log installer exists only behind `test-fixtures`; default-release compile-fail and rustdoc audits
  prevent it from becoming a production API. No production branch reads tests, filenames, workload identity, affinity,
  allocator state or guard thresholds.

## Final Measurement Interpretation

- Standard receipt SHA-256 `eb8a2556…1dda`: event-heavy p95 `10.818511 ms`, concurrent p95 `8.343780 ms`; both strict
  caps pass, exact one-sided regression p is `1.0`, supported regression is false and deterministic P01 parity is true.
- Allocator receipt SHA-256 `71b20b60…7fee`: event-heavy is `14,295 allocations / 25,896 reallocations / 260,550 peak /
  260,550 retained bytes`; concurrent is `170,740 / 140,036 / 8,001,938 maximum peak / 4,196,928 retained`. Every raw
  sample is within the corresponding P01 operation, peak and retained maximum.
- Public standard receipt SHA-256 `21475b8b…76a1`: snapshot `226.508 µs / 48,493 bytes`; checkpoint
  `8.344 µs / 17,956 bytes`; newest ten query `0.202/0.212 µs`; staging `23.399/22.284 µs`; all exact/cap/scale gates
  pass. Public allocation receipt SHA-256 `82ece8bc…93f9` records five exact low/high staging samples at
  `675 / 0 / 84,409 / 0` allocation/reallocation/peak/retained.
- `evidence/P08_FINAL_P01_EXACT_RESULT.json` SHA-256 `1c8e6949…6d9d` independently executes the immutable P01 source and
  preserves its output identities. Instrumented receipt timing is observational only; standard builds own latency.
