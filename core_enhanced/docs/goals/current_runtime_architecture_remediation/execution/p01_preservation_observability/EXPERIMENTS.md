# P01 Preservation and Observability Experiments

## 2026-08-22 — GEX-00 Intake

- Run mode: Goal Set `goal-run`.
- Selected member: `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P01-PRESERVATION-OBSERVABILITY)`.
- Direct predecessor: none; strict sequence makes P01 the sole eligible Phase.
- Existing exact-identity execution history: none. The only P01 execution set is historical `GS-CRAR-P2G50-001` and
  was pre-read before current code/status/test investigation as required.
- Canonical directory selected: `execution/p01_preservation_observability`.
- Upstream digest check: Freeze, Plan, Receipt and Goal Set all exact-match `HA-CRAR-P2G60-002`.
- Production mutation in this GEX-00 pass: none.

### Current Core-Only Baseline

- Revision / branch: `f318414f…c09` / `cleanup/simulator-dirty-worktree-20260803`.
- `git status --porcelain=v1 --untracked-files=all -- .`: modified 134, deleted 23, untracked 3,132, total 3,289;
  SHA-256 `1358edd3…a85`.
- Candidate exact status: `docs/game_rulebook.md` modified; feature manifest and support tool untracked.
- Candidate manifest SHA-256 `c81f7ffc…1321`, matching the reviewed current identity.
- Expected durable receipt is absent and remains GEX-10 output.
- cargo `1.91.1`; rustc `1.91.1`; standalone tool workspace is available.
- Validation/test execution: none. GEX-00 performed intake and runner availability checks only.

### GEX-00 Result

- Result: `passed`.
- Production implementation: none.
- No predecessor audit required; no upstream contradiction or P01 selection blocker found.
- Dirty isolation is feasible for P01 because its boundary is support/docs only and production behavior mutation is
  prohibited.
- Next authorized pass: separated `GEX-10` under the active Goal Set goal-run.

## 2026-08-22 — GEX-10 Current Rebind and Implementation

### Pre-Mutation Rebind

- Selected Goal, upstream basis and Core-only authorization remain unchanged.
- Pre-mutation candidate identities: manifest `c81f7ffc…1321`, baseline source `1e9b01ac…c9de`, verifier
  `5a2f55aa…993f`, support Cargo manifest `928a1a70…7bee`.
- Existing baseline receipt: absent.
- The support tool records deterministic latency/event/encoded-size samples but has no direct allocation/retained heap
  fields. This is the exact technical implementation gap owned by P01.
- Mutation boundary: dedicated standalone support tool and its durable receipt; production runtime/data behavior remains
  unchanged.

### Direct Heap/Allocation Observation

- Added a standalone `System` global-allocator observer. Each workload sample records baseline/end/peak live bytes,
  peak delta, signed retained delta, allocation/deallocation/reallocation counts.
- Workload/receipt schema advanced to `crar-preservation-baseline/2` / schema `2`.
- No production crate allocator or runtime branch was changed; instrumentation exists only in the support binary.
- Release command:
  `cargo run --release --manifest-path tools/runtime_architecture_preservation/Cargo.toml --bin
  preservation_baseline -- evidence/P01_PERFORMANCE_BASELINE.json` → exit 0.
- Event-heavy 5/5: 186 events, 65,720 bytes and output SHA-256 `2ada19d4…333a` exact-repeat; allocation count
  31,315, retained/peak delta 276,174 bytes in every sample; elapsed 24.283–26.139 ms.
- 64-session/8-way 5/5: 11,840 events, 3,881,088 bytes and output SHA-256 `68da2cf9…3ca9` exact-repeat;
  allocation count 226,164 and retained delta 4,196,928 bytes exact-repeat; observed peak delta
  8,991,118–9,044,410 bytes; elapsed 11.988–12.805 ms.
- This is a pre-change observation, not an optimization or PASS. P08 owns the final `<25 ms` event-heavy and `<10 ms`
  64-session p95 gates and like-for-like architecture comparison.
- Replay command wrote audit-excluded `target/P01_PERFORMANCE_BASELINE_REPLAY.json` and exited 0. A direct diff of
  output/event/encoded-size/allocation/deallocation/reallocation/retained-delta rows against the durable receipt was
  empty. Replay whole-file SHA-256: `15c701f5…124` (timestamps/latency/peak values intentionally not exact gates).

### Manifest and Receipt Currentness Seam

- Manifest purpose now binds `IVA-CRAR-C2-01/09` and records the exact release command, workload version, sample count
  and required direct metrics.
- Verifier now rejects stale tool hashes, non-release receipts, missing/non-positive direct metrics, non-sequential
  samples and divergent deterministic identities, and emits a composite identity for every current locator content.
- Current verifier command exited 0: features 57, headings 57, bundles 14, Validation Dimensions 10, legacy targets 10,
  unknown/unmapped/duplicate 0; current locator content 42 with composite SHA-256
  `ed92f21c016129419f01356c39a8f14200d865a4b6cca1e151377f796ae07ac4`.
- Current manifest SHA-256 `c25d3e97…13c7`; receipt SHA-256 `3eb781e1…0c44`; tool source SHA-256
  `d9f7fd47…815d`.
- Historical `../game_resources` direct locator was replaced by the owning Core loader's embedded-source registry
  locator. The manifest/currentness surface is now Core paths only; live data execution remains proven by the current
  Core loader and `ron_loading` tests.

### Focused and Broad Validation

- `cargo test --manifest-path tools/runtime_architecture_preservation/Cargo.toml --all-targets` → exit 0;
  binaries compile, no unit tests are defined. This command is compile coverage, not the non-zero seam result.
- `cargo fmt --manifest-path tools/runtime_architecture_preservation/Cargo.toml --check` → exit 0.
- `cargo clippy --manifest-path tools/runtime_architecture_preservation/Cargo.toml --all-targets -- -D warnings`
  initially found one `manual_try_fold` lint; the iterator was corrected and the exact command then exited 0.
- `cargo run ... --bin verify_feature_manifest ...` → exit 0 with the non-zero closure above.
- `cargo test -p game_core` → exit 0: lib 1,080; integration 4 + 4 + 25 + 14 + 4; total 1,131 passed,
  failed 0, ignored 0. Doc tests selected 0 and are not counted as positive evidence.
- No Core source/data/test member changed after that broad run; later edits were confined to the standalone verifier and
  execution evidence.

### Validation Dimension Evidence Map

| Dimension | Current direct evidence |
| --- | --- |
| `success` | 57-feature/42-content manifest closure plus current 1,131-test Core suite |
| `invalid_input` | executed live RON publication rejection matrix including invalid equipment-before-publication and missing/unknown source fields |
| `failure_rollback` | executed reward/event late-failure, transaction rollback and no-partial-restore Core tests |
| `retry_idempotency` | executed at-most-once event grant, raw-roll reuse, request identity and retry fixtures |
| `ordering_concurrency` | event-heavy ordering plus 64-session/8-way deterministic receipt/replay and current ordering tests |
| `boundary_empty_max` | executed empty embedded source, revision overflow, future cursor and workload boundary tests |
| `disconnect_restart_persistence` | executed V4 checkpoint continue/capture, facility round-trip and checkpoint restore corpus |
| `live_data` | `ron_loading` 25/25 plus current live publication tests and manifest live input locator |
| `producer_consumer` | executed command result/root snapshot, battle update and Core public projection routes in the current manifest |
| `legacy_fallback_absence` | ten current removal-target locators identified; support source review confirms new production branch/fallback 0 |

### GEX-10 Result and Audit Target Freeze

- All assigned P01 implementation boundaries and required evidence rows are present; unresolved material policy,
  consumer or dirty-isolation blocker: none.
- Production behavior delta: 0. No prior success result was used instead of current execution.
- Unrelated Core dirty baseline after excluding exact P01 paths: modified 133, deleted 23, untracked 683, total 839;
  fingerprint `3688d42e…b672`.
- GEX-10 result: `exit conditions met`; no Phase completion verdict claimed.
- Exact audit target: `ATR-CRAR-C2-P01-GEX10-001` in `evidence/P01_AUDIT_TARGET.json`.
- Next pass: fresh read-only GEX-20.
