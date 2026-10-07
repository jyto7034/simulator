# Phase 44 CRA-P10-C06 Completion Audit

Audit Mode: phase_exit
Audit Target: Phase 44 CRA-P10-C06 Core Package Targets And Direct Dependencies Cleanup
Audit Date: 2026-08-11 (`Asia/Seoul`)
Current Status At Audit Start: active
Implementation Complete Claim At Audit Start: false — exit claim intentionally withheld pending this audit
Policy Source: CRA-POL-006 plus the user's 2026-08-11 explicit verified package/tool/public cleanup approval
Goal Documents: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
Repository / Start Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Direct Predecessor: Phase 43 CRA-P09-C07, `Complete / High`, audit `a9b674b4...`

## Audit Method

This is a separate phase-exit pass under `goal_completion_review_guide.md`. The auditor fully reread all three Child
documents and the 567-line review guide after implementation validation. A wording conflict was found before verdict:
several preservation bullets said every public Core API was immutable even though the Phase authorization and
historical candidate explicitly permit removal of the inventoried unconsumed logger/init API. Those bullets were
corrected to preserve every gameplay/domain public API while naming the one approved package-API exception. No
production correction was required.

The audit then independently reran locked metadata assertions, removed-surface searches, retained-root hashes, five
fresh `/tmp` generator invocations, both Core/server broad debug suites, default/feature checks, format/diff, dirty
status and marker controls. Earlier both-profile and Clippy evidence remains current because no production path changed
between that evidence and this audit.

## Completion Evidence Matrix

| ID | Completion condition or invariant | Policy basis | Current implementation/contract evidence | Verification evidence | Result |
| --- | --- | --- | --- | --- | --- |
| P44-C01 | Current package/target graph is exact | P44-1, Cargo metadata authority | Core manifest has `autobins = false`; metadata has one lib, five named bins, five integrations and no default bin | locked metadata assertion checks every kind/name/count and `test = false` | pass |
| P44-C02 | Dependency/config graph is exact | P44-1, minimal positive edges | eight normal edges, no build/dev edges; member config absent; feature retained | locked metadata + direct `cargo tree` + no-default/feature checks | pass |
| P44-C03 | Dirty work is preserved | dirty-work invariant | five activation-clean candidate paths add five rows; one canonical subsection is isolated in an already-dirty path | 606→611 exact status delta, `c83e7bc0...`; marker exact | pass |
| P44-C04 | Every candidate surface is classified | P44-1 disposition requirement | library/integrations/feature/tools/workflow retained; default/main/logger/config and unused edges removed | manifest/source/config/target disposition tables below | pass |
| P44-C05 | Standalone/external consumer gate is honest | CRA-POL-006 + explicit approval | no workspace/accessible sibling consumer; inaccessible deployment remains unknown; tool contract retained | exact repository/sibling searches and preserved commands/output | pass |
| P44-C06 | Retained tools were characterized before cleanup | P44-2 | all five names, source hashes, byte counts, JSON keys and stdout hashes captured | E04 pre-controls selected and ran every target | pass |
| P44-C07 | Only proven package noise was removed | P44-3 + no-shim approval | exact logger/ctor/main/config/direct-edge delta; no tool source, wrapper, alias or CI change | scoped diff, absence search, retained hashes, compile/test parity | pass |
| P44-C08 | Metadata/dependency result is exact | package authority model | 38 normal + 4 build → 8 normal; 524 → 311 lock packages | before/after metadata, lock counts, locked workspace check | pass |
| P44-C09 | Fixture and feature behavior is exact | preserved tool/feature invariant | five commands unchanged; `event_log_checks` unchanged | all stdout byte/hash/key parity; feature and no-default checks pass | pass |
| P44-C10 | Library/server/test behavior is exact | library/server invariant | gameplay/domain sources and server manifest untouched; test list exact | Core 1,063 and server 63 in debug/release; sorted hashes exact | pass |
| P44-C11 | Broad/static validation is green | P44-5 | workspace resolves reduced lock and server path edge | workspace debug/release/locked checks, Clippy, fmt and diff pass | pass |
| P44-C12 | Independent audit/Master boundary is current | P44-6, Master final gate | this audit is separate; Master remains incomplete pending its own audit | current direct assertions and broad debug reruns pass | pass |

## Target, Surface And Dependency Disposition

| Surface | Current disposition | Evidence |
| --- | --- | --- |
| `game_core` library | retain | sole Core runtime package target; server path consumer compiles/tests |
| implicit `game_core` default bin / `src/main.rs` | remove | one-line empty main; no caller; metadata now has no default bin |
| five `generate_*_contract_fixture` bins | retain and declare explicitly | names/paths/source/output exact; `test = false`, `bench = false` |
| five integration tests | retain | metadata names and 4 + 4 + 25 + 14 + 4 execution exact |
| `event_log_checks` | retain opt-in | live `cfg` references; feature-enabled all-target check passes |
| Core `LoggerManager` / `init_logging` / disabled ctor | remove | no Core external workspace import; no accessible sibling consumer; server owns another logger |
| Core `.cargo/config.toml` `tokio_unstable` | remove | Core-local, no retained Tokio edge/consumer; both package/server profiles pass |
| member coverage workflow | retain byte-exact | CLI is installed by workflow; source remains 31 lines at `42462da0...` |
| root/server manifests and server examples | retain | no Phase-owned diff |

Retained normal dependencies are exactly `rand`, `rapier2d`, `ron`, `serde`, `serde_json`,
`serde_path_to_error`, `tracing` and `uuid`.

Removed normal edges are exactly `actix`, `actix-web`, `actix-ws`, `argon2`, `async-tungstenite`, `base64`,
`byteorder`, `cargo-llvm-cov`, `clap`, `console-subscriber`, `criterion`, `ctor`, `dotenv`, `futures`,
`futures-util`, `gnuplot`, `hex`, `lazy_static`, `parking_lot`, `paste`, `rand_core`, `rayon`, `reqwest`,
`serial_test`, `structopt`, `tokio`, `tokio-console`, `tracing-appender`, `tracing-subscriber` and `url`.
Removed build edges are exactly `globwalk`, build-kind `serde`, build-kind `serde_json` and `serde_yaml`; no
`build.rs` exists. Every removed edge had zero retained compiled-root reference, and all-target/feature builds prove
there is no hidden required edge.

## Runtime, Data, DTO And Persistence Evidence

This Phase changes package construction and one unconsumed logger API only. It does not change gameplay/runtime
modules, live RON, serializer DTOs, persistence/version readers or server mappings. Their relevant cross-component
control is stronger than a source-only assertion:

- every retained generator serializes the same JSON bytes and key set before/after;
- Core's entire 1,063-test inventory and server's 63-test inventory remain name/hash exact and pass in both profiles;
- the server continues to compile and run against the reduced Core path dependency; and
- all five generator source roots remain byte-exact.

No data migration, save cutover, wire rollout or checked-in fixture regeneration applies. No runtime module was
formatted or opportunistically changed.

## Consumer Evidence

- Confirmed workspace consumer: `game_server` is the only package depending on `game_core`; it consumes no Core
  logger symbol and owns its own process logger.
- Confirmed workspace tool callers: none outside historical Goal evidence. The commands are nevertheless retained as
  supported standalone interfaces.
- Checked-in JSON consumer fixture: no matching fixture artifact was found in this repository.
- Accessible adjacent repositories: exact searches of `/mnt/f/work/contract`, `/mnt/f/work/familiy`,
  `/mnt/f/work/mail` and `/mnt/f/work/naver-trader` find no target/logger/default-bin/config consumer.
- Canonical external consumer: earlier historical evidence mentions Unity fixture workflows, but no accessible Unity
  tree is present under the allowed adjacent root. Tool names and exact outputs are preserved, so no migration is
  required for such a consumer.
- Inaccessible/unknown deployed consumer: existence cannot be proven. The user's explicit post-inventory public and
  package cleanup approval permits removal of the zero-consumer logger/default-bin surfaces without a shim.

## Fixture Output Parity

The audit created a new `/tmp/phase44-fixture-audit.*` directory, ran every retained command into a separate file,
parsed each file as JSON, checked byte length/SHA-256/top-level keys, then removed only that temporary directory.

| Tool | Bytes | SHA-256 | Result |
| --- | ---: | --- | --- |
| event | 6,839 | `54b3d90db52c574db6d6790524ba5c4e57773a67e9ac4575144118585900ae2b` | exact |
| encounter compendium | 815 | `ec9c851dcdc0b6b5ec8f1984a6cc55b760fa3d045d6b1a94682243163587a052` | exact |
| shop | 3,101 | `88fb016e4e205daebbdc70a73777d505fe331979b7715f358ff00cbf41e1a529` | exact |
| inventory | 2,225 | `9ec7f114cfdf59116d992d13b6ab069d4179eb44142562fd34a3e1a17abc3d86` | exact |
| research delivery | 456 | `f7c507116c5cae8766ad83b16e3d6525a85b7a35d1af3cdf280064bfaf8f8d25` | exact |

## Legacy And Compatibility Audit

- `src/main.rs` and Core `.cargo/config.toml` are absent.
- Core source/test/manifest search has no `LoggerManager`, `init_logging`, `init_tracing_for_tests`, `ctor::ctor` or
  `tokio_unstable` result.
- Core manifest has no build-dependency section or removed direct edge.
- Metadata has no default bin, build/dev edge, alias feature or test-enabled fixture bin.
- No compatibility logger, wrapper executable, dependency aggregator, feature alias or implicit replacement target
  was added.
- Five retained tool commands are intentionally not legacy: their source and output remain authoritative and exact.

## Scoped Diff And Dirty Preservation

Activation fixed 606 expanded status rows at `1d7fae2a...`. Implementation changes exactly five previously clean
status paths: modified `Cargo.lock`, modified `core_enhanced/Cargo.toml`, deleted Core config, modified Core `lib.rs`
and deleted Core `main.rs`. The current pre-audit-file state is 611 rows at `c83e7bc0...`, exactly activation + five.

The durable engineering workflow was already dirty at activation (`044d853b...`), and the only Phase-owned content
there is the isolated `Core package와 fixture 도구` subsection. Root/server manifests, all generator roots, the
coverage workflow and server examples have no Phase-owned change. `git diff --check` passes. Protected marker
`game_server/logs/app.log.2026-08-09` remains 57 bytes at `614d2fbd...`.

Cargo-generated lock churn is explained: 524 packages/5,244 lines at activation become 311 packages/2,954 lines,
with 64 additions and 2,354 deletions in the lock diff. Cargo alone resolved the consequences of direct-edge removal;
no transitive package was edited or force-pruned.

## Canonical Alignment

The closest durable package policy is now `core_enhanced/docs/engineering_workflow.md`. It states that Core has no
default executable, the server owns process execution/logging, lists all five exact generator commands, records their
non-test/bench role and preserves `event_log_checks` as opt-in. It matches manifest metadata and actual commands.
No gameplay/runtime/data/wire canonical required a semantic update because those contracts did not change.

## Validation Results

- Pre/post sorted test list: Core 1,063 at `2db28d25...`; server 63 at `beca9e7a...`.
- Normal `cargo test -p game_core`: 1,063 passed, 0 failed; five integrations; zero fixture/default-bin harness runs.
- `cargo test -p game_core --all-targets`: 1,063 passed, 0 failed in debug and release. Explicit `--all-targets`
  forces five zero-test bin harnesses even though normal metadata/test selection marks them non-test.
- `cargo test -p game_server --all-targets`: 63 passed, 0 failed in debug and release; four inherited server
  bin/example zero-test groups.
- Independent audit debug rerun: Core 1,063 passed across 11 result groups (five forced zero groups); server 63 passed
  across five groups (four zero groups).
- `cargo check --workspace --all-targets`: pass in debug, release and locked debug modes.
- Core `--no-default-features --locked` and `--no-default-features --features event_log_checks --locked` all-target
  checks: pass.
- Normal Core Clippy: pass with 80 inherited lib warnings and the recorded test warnings; normal server Clippy: pass
  with inherited warnings. No lint auto-fix ran.
- `cargo fmt --all -- --check`: pass before and during audit. `git diff --check`: pass.
- Locked metadata assertion, removed-surface assertion, retained-root hashes and protected marker assertion: pass.

Baseline failures: none. All warnings pre-exist this package cleanup and are outside scope.

Omitted validation:

- Unity build/runtime: no accessible Unity consumer exists in scope, while every retained command and byte output is
  exact; therefore no consumer migration occurred.
- Network/platform matrix: no retained target-specific dependency/config claim exists; workspace both profiles and
  member no-default/feature modes pass.
- Performance benchmarks: no runtime path changed and fixture bins are explicitly non-bench targets.

## Cross-Component Findings

No material finding. The server/Core path seam compiles and its full tests pass in both profiles. Server logger
ownership is independent from the removed Core logger. Reduced Core lock resolution does not remove server-owned
Actix/Tokio/logging edges; workspace metadata and builds still resolve them through `game_server`.

## Long-Term Direction Review

| Candidate ID | Work type | Current→target authority result | Preserved invariant result | Removal/consolidation result | Verification | Result |
| --- | --- | --- | --- | --- | --- | --- |
| CRA-P10-C06 | cleanup | implicit broad package → explicit lib + five tools + positive edges | gameplay/domain, server, feature and tool output exact | empty/default/no-op/unowned edges/config removed without shim | metadata, fixture parity, both-profile suites, static checks | pass |

- Fit: High
- Improvement class: none
- Reason: Cargo metadata now names the intended target authority directly, dependencies have retained compiled owners,
  process logging stays with the executable server, and the cleanup creates no new wrapper, alias or duplicate state.

## Primary Verdict

- Complete
- Long-term fit: High
- Confidence: High

All twelve Phase completion conditions pass. There is no material legacy, compatibility shim, unexplained lock drift,
unhandled accessible consumer, output change, test drift, dirty-work violation or canonical conflict.

## Status Transition

- Recommended Child transition: `active / false` → `complete / true`
- Recommended Master registry transition: Phase 44 `active` → `complete`
- Next Phase activation: not applicable; this is Phase 44 of 44
- Master completion: denied until the separate Master final audit returns `Complete / High`

## Remaining Risk And Required Action

Residual risk is limited to inaccessible deployed callers of the removed no-consumer logger/default-bin surface. It
is explicitly accepted by the user's verified public/package cleanup approval; retained fixture callers require no
migration because command and output are exact. The required next action is the independent Master-wide final audit,
not further Phase 44 implementation.
