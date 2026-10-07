# P08 Performance and Final Integration Execution Plan

상태: `GEX-00 passed / GEX-10 WP-CRAR-C2-09 passed / WP-CRAR-C2-10 passed / GEX-20 pending`

## Current Execution Identity

- Handoff / root target: `HA-CRAR-P2G60-002` / full `GS-CRAR-P2G50-002` goal-run.
- Selected member: `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION)`.
- Phase: `PHASE-CRAR-C2-08-PERFORMANCE-FINAL-INTEGRATION`.
- Strict internal execution-node order: `WP-CRAR-C2-09` performance stabilization, then
  `WP-CRAR-C2-10` final evidence reconciliation.
- Goal Quality Contract: `2`; durable seams `IVA-CRAR-C2-01/02/06/08/09/10/11`.
- Policy / decision / realization: `PER-CRAR-008-R6-CODE-REVIEWED`, `DR-CRAR-002-R1`, `TD-CRAR-018/019`
  (with the evidence-derived Core guards recorded by `TD-CRAR-010`).
- Authorization: current Core runtime, Core-owned benchmark/validation tooling, Core tests, manifest and canonical Core
  documentation only. `game_server`, Unity, sibling resources, external consumers, deployment and product SLA claims
  are excluded and will not be inspected.

## Dependencies and Entry Gate

- P01 through P07 are each `Complete / High`; P07 froze `ATR-CRAR-C2-P07-GEX10-001` and all predecessor successor-
  currentness records are current through P07.
- P01's accepted immutable comparison input is root `evidence/P01_PERFORMANCE_BASELINE.json`, schema/workload
  `2` / `crar-preservation-baseline/2`, with source SHA-256 `64debfd6…546a` and receipt SHA-256 `86ccd5c7…5848`.
- The similarly named receipt under historical `execution/p01_preservation_baseline` is bound to stopped Goal Set `001`,
  schema/workload `1`, and is not a P08 input.
- P01 raw samples are five deterministic release observations per workload. Event-heavy is 186 events / 65,720 bytes /
  `2ada19d4…333a`; 64-session/8-way is 11,840 events / 3,881,088 bytes / `68da2cf9…ca9`. Both include direct
  allocation, peak and retained-byte observations.
- `GAP-CRAR-P2G20-002` is only half closed: P01 owns the pre-change direct observation; P08 must publish the final
  like-for-like result and permanent guard before any performance or Goal PASS.

## Objective and Measurement Authority

- Measure only the functionally complete P07 architecture, compare the exact P01 workloads without changing their
  scenario/input/seed/duration/session/lane/step meanings, and optimize only a hotspot demonstrated by the initial
  current measurement/profile.
- Bind final results to the exact P01 receipt, P01 tool/input hashes, current benchmark source, release profile,
  environment and raw samples. Deterministic event count, encoded bytes and output SHA-256 must equal P01 exactly.
- Use repeated interleaving after warm-up. Report raw samples, nearest-rank p95 and a documented one-sided exact
  non-parametric comparison; do not disguise a percentage allowance as non-regression. A supported regression,
  deterministic mismatch or missing direct heap result fails the node.
- Treat the P01-compatible private-kernel capability as the exact before/after comparison seam. Separately measure the
  actual `CoreApplication` public boundary so snapshot/result serialization, checkpoint export and application staging
  are not hidden by a domain-only benchmark.
- Numeric safety gates are implementation guards, not external SLAs: event-heavy release p95 `<25 ms`, 64-session/8-way
  p95 `<10 ms`, exact deterministic output, TD-018 history caps (`8,192 events OR 4 MiB OR 180 battle-seconds`) and
  current evidence-derived Core query/serialization/checkpoint/staging caps. No arbitrary additional cap is permitted.

## Approved GEX-10 Sequence

### `WP-CRAR-C2-09` — Performance Stabilization

1. Add a standalone Core-owned final-validation workspace. Keep the P01 source/receipt immutable; copy and independently
   verify the exact workload semantics rather than rebasing the baseline.
2. Run the unoptimized current final architecture first and retain its raw receipt/profile. Classify each absolute,
   deterministic, direct-allocation and relative gate before changing production code.
3. Measure direct allocation/peak/retained delta for both representative workloads. Measure bounded-history newest-tail
   and resync queries at ordinary and cap-scale windows, immutable application snapshot/result serialization,
   V5 checkpoint export, and transaction staging with small versus cap-scale retained diagnostic history.
4. Derive any guard not already numeric in TD-018/019/010 from the retained current evidence. Record the derivation and
   enforce it in the standalone verifier or a focused Core test; never create a production branch keyed to the harness.
5. If a gate fails, profile the exact failing workload and change only the proven Core hotspot. Preserve authoritative
   outputs, rejection/rollback, ordering, caps, continuation and public projection exactly; re-run the full receipt.
6. Freeze a performance receipt only when every relative/absolute/deterministic/memory/query/serialization/checkpoint/
   staging guard passes. Until then `WP-CRAR-C2-10` remains ineligible.

### `WP-CRAR-C2-10` — Final Evidence Reconciliation

7. After `WP-CRAR-C2-09` passes, reconcile the 57-feature manifest and all ten Validation Dimensions against current
   source/tests/data. Publish a ten-acceptance evidence index with non-zero current commands and exact result identities.
8. Run focused success/invalid/rollback/retry/order/boundary/restart/live-data/public-producer-consumer evidence, the
   full Core suite, release warnings/rustdoc, manifest verifier and every Core API privacy probe.
9. Audit current Core source and public API for all manifest removal targets, competing application writers, raw domain
   mutation exposure, unbounded active history, V4/legacy writer, compatibility fallback, test-dependent production
   branch and benchmark-dependent production branch. Required count is zero; an explicit V4 importer remains only if
   its P06 certification/removal gate still requires it and is not misclassified as a fallback.
10. Refresh P01–P07 successor-currentness for every affected frozen member, freeze a P08 GEX-10 Audit Target, then stop
    mutation. A later fresh read-only GEX-20 owns the completion verdict.

## Completion Contract

- exact P01 workload/input/output parity and no statistically supported final regression
- event-heavy p95 `<25 ms`; 64-session/8-way p95 `<10 ms`; release-profile raw sample count non-zero
- direct allocation, peak and retained-memory result for every representative sample; `GAP-CRAR-P2G20-002` closed
- bounded history/query cost and cap identity, public snapshot/result serialization, V5 checkpoint export and retained-
  history-independent staging each have a current numeric guard and passing evidence
- 57/57 feature inventory, 10/10 policy acceptance and all ten Validation Dimensions current
- full Core suite and all positive/negative Core API probes pass; legacy/bypass/dual-writer/unbounded-history counts zero
- no external consumer evidence, production promotion or deferred feature-policy implementation is used

## Validation Dimension Plan

| Dimension | Direct P08 evidence |
| --- | --- |
| `success` | final representative workloads, normal application calls and full Core suite |
| `invalid_input` | aggregate invalid data/command/non-finite/checkpoint matrix with exact typed rejection |
| `failure_rollback` | transaction/kernel/checkpoint fault corpus plus pre/post authority equality |
| `retry_idempotency` | repeated rejection/query/import/result corpus with identical meaning |
| `ordering_concurrency` | interleaved event-heavy and 64-session/8-way deterministic ordering |
| `boundary_empty_max` | empty/max snapshot, exact history caps, cursor overflow and checkpoint bounds |
| `disconnect_restart_persistence` | bounded resync plus V4-import/V5-write continuation identity |
| `live_data` | current embedded RON publication through final Core |
| `producer_consumer` | actual standalone caller using `CoreApplication`; external consumer claim zero |
| `legacy_fallback_absence` | current static/API audit for superseded roots, writers, fallback and bypass |

## Stop Conditions

- P01 receipt/source/input identity is stale, ambiguous or cannot be reproduced like-for-like
- a supported regression, `<25 ms`/`<10 ms` failure, nondeterministic output, missing direct heap result or any permanent
  guard failure remains after evidence-backed in-scope hotspot work
- optimization would weaken semantic parity, rollback, history caps, checkpoint continuation or public boundary
- `WP-CRAR-C2-10` would have to begin before `WP-CRAR-C2-09` passes
- completion would require external source/schema/consumer inspection or an arbitrary cap/allowance
- predecessor evidence becomes stale without successor-currentness, dirty overlap cannot be isolated, or Goal meaning
  changes

## GEX-00 Pre-edit Identities

- `evidence/P01_PERFORMANCE_BASELINE.json` `86ccd5c78b0dcdfab502b1c5842a48c8a9df8963557e36cbf49b3ea5ec2b5848`
- `tools/runtime_architecture_preservation/src/main.rs` `64debfd6546afb617636caf520fcb9e8ee49acc1ee2377306cf8a19afbd8c631`
- `src/game/battle/event_log.rs` `ac5dd0795048b146fd344657300c12db888fa098b168bc71920ca703ea8e6d09`
- `src/game/world/application.rs` `7d9c8f707a90ff5c32cf3de2355d4188f2906045248445ccdcac336d56e20df1`
- `src/game/world/combat.rs` `13813d8b3c8eaaf79d4c22c8c266ffb1de13bd9bd3679539366864bd71f966e8`
- `src/application.rs` `08889156071241bd5d38bbbafe9220bfa7fe4fa9b3caa9ac0f3a9c769c5acf42`
- `docs/core_runtime_contract.md` `c01cf2d0e99a94c2320187858ca4634f4bc8577eb9f4a992a13f50c2ab066f0f`
- `docs/architecture/runtime_feature_manifest.v1.json` `2a8cfa963f1fafe0169f3015e49fcdede8b07ba40af8846eedaf1b33a51d7cd7`
- `tools/runtime_architecture_preservation/src/bin/verify_feature_manifest.rs`
  `9ad420948292c592f507a6790e8917b98bce8b25ca523b6e0c9bb20d25c65b37`
- `evidence/P07_AUDIT_TARGET.json` `44d81414fe5c13e143091c9e13085f986428871f01b68359567fda63789859ae`
- `tools/runtime_architecture_final_validation`, P08 execution directory and P08 receipts: absent before P08.

The revision/branch are `f318414fb0156a67e04c5479000c2a2683fc0c09` /
`cleanup/simulator-dirty-worktree-20260803`; cargo/rustc are `1.91.1`. Existing dirty/mixed Core files remain user work
outside the exact P08 hunks. P07's unrelated-Core baseline is 82 modified, 23 deleted, 679 untracked, total 784,
fingerprint `f3e50fe9…9fba`; GEX-10 must recompute it with exact cumulative P01–P08 exclusions.
