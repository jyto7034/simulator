# P08 Final Core Evidence Index

Identity: `GS-CRAR-P2G50-002 / GOAL-CRAR-C2-P08-PERFORMANCE-FINAL-INTEGRATION / WP-CRAR-C2-09+10`

Scope: Core only. `game_server`, Unity, sibling resources, external consumers, deployment and product SLA evidence are
excluded and were not inspected.

## Ten Policy Acceptances

| Policy element | Current direct evidence | Result |
| --- | --- | --- |
| PE-001 preservation | 57/57 manifest; immutable P01 exact replay; full 1,159 tests | pass |
| PE-002 closed lifecycle | application selector 9/9; exhaustive lifecycle validation in full suite | pass |
| PE-003 atomic transaction | exact low/high rejected-command rollback; P03 currentness `_006` | pass |
| PE-004 single authority owner | one application commit port; route-owner registry tests; competing writer 0 | pass |
| PE-005 validated data | live embedded public guard; 25 RON integrations; data probes | pass |
| PE-006 private finite kernel | exact workload parity; Rapier 17/17; kernel compile-fail probe | pass |
| PE-007 public projection | live `CoreApplication`; 58/49/32/7 exhaustive mappings; public probes | pass |
| PE-008 performance | standard/observer receipts; 10.818511/8.343780 ms; public guards all green | pass |
| PE-009 bounded history/result | exact three caps; query/staging scale; event/result selectors; old Vec absent | pass |
| PE-010 checkpoint migration | V5 8.344 µs/17,956 bytes; checkpoint 15/15; V4 writer 0/importer certified | pass |

Missing, failed or unknown policy acceptance: `0`. Inventory: `57/57`.

## Ten Validation Dimensions

| Dimension | Non-zero current evidence | Result |
| --- | --- | --- |
| `success` | all five final performance/public receipts and 1,159-test suite | pass |
| `invalid_input` | application/data/kernel/history/checkpoint invalid corpus and six negative probes | pass |
| `failure_rollback` | exact public staging snapshot, application selector and checkpoint corruption corpus | pass |
| `retry_idempotency` | application rejection retry, checkpoint retry, query exact-repeatability | pass |
| `ordering_concurrency` | 21 exact 64-session/8-way samples plus kernel ordering suite | pass |
| `boundary_empty_max` | 8,192/4 MiB/180 s caps, 1,024/8,192 query, snapshot/checkpoint byte limits | pass |
| `disconnect_restart_persistence` | bounded resync tests and V4-import/V5-write continuation selector | pass |
| `live_data` | actual embedded data in public guard and full RON/live test suites | pass |
| `producer_consumer` | standalone default-release Core callers and public façade guard | pass |
| `legacy_fallback_absence` | manifest/static/rustdoc/API audits; V4 importer explicitly classified | pass |

## Frozen Performance Evidence

| Path | SHA-256 | Authority |
| --- | --- | --- |
| `evidence/P01_PERFORMANCE_BASELINE.json` | `86ccd5c7…5848` | immutable before input |
| `evidence/P08_FINAL_PERFORMANCE_LATENCY.json` | `eb8a2556…1dda` | standard-build latency/non-regression |
| `evidence/P08_FINAL_PERFORMANCE_ALLOCATION.json` | `71b20b60…7fee` | direct allocator operation/peak/retained |
| `evidence/P08_FINAL_P01_EXACT_RESULT.json` | `1c8e6949…6d9d` | immutable source replay |
| `evidence/P08_PUBLIC_PERFORMANCE_GUARDS.json` | `21475b8b…76a1` | public time/size/query/staging |
| `evidence/P08_PUBLIC_ALLOCATION_GUARDS.json` | `82ece8bc…93f9` | public staging allocation scaling |

The retained initial/profile receipts are failure diagnosis only. They are not substituted for final evidence.

## Current Commands

- `cargo run --release --manifest-path tools/runtime_architecture_final_validation/Cargo.toml --bin final_performance -- evidence/P01_PERFORMANCE_BASELINE.json evidence/P08_FINAL_PERFORMANCE_LATENCY.json`
- same command with `--features allocation-observer` to the allocation receipt
- `cargo run --release --manifest-path tools/runtime_architecture_final_validation/Cargo.toml --bin p01_exact_replay -- evidence/P08_FINAL_P01_EXACT_RESULT.json`
- `cargo run --release --manifest-path tools/runtime_architecture_final_validation/Cargo.toml --bin public_guards -- evidence/P08_PUBLIC_PERFORMANCE_GUARDS.json`
- same command with `--features allocation-observer` to the public allocation receipt
- `cargo test -p game_core --all-targets`
- `cargo check -p game_core --release --lib`
- `cargo doc -p game_core --release --no-deps`
- `cargo run --manifest-path tools/runtime_architecture_preservation/Cargo.toml --bin verify_feature_manifest -- docs/architecture/runtime_feature_manifest.v1.json`
- five established valid/invalid release probe pairs plus P08's default-fixture invalid probe

Every listed selector ran after the last production mutation. Result identities are recorded in
`evidence/P08_PERFORMANCE_FINAL_INTEGRATION_RECEIPT.json`.
