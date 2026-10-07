# Execution evidence

## GEX-00 / 2026-09-09

Read current request, operating rules, pipeline readers and GEX-00/10 contracts; current policy/review/Goal/Handoff and direct source. Six Handoff source/context hashes match. Relevant production/data/test/build paths match the previous review baseline. Created baseline hashes and preserved existing dirty status. `rustc 1.91.1 (ed61e7d7e 2025-11-07)`; `cargo 1.91.1 (ea2d97820 2025-10-10)`. No game tests or production changes in this pass. Detailed code design and tests belong to the following GEX-10.

## GEX-10 / 2026-09-09

All commands run at `/mnt/f/work/simulator`, default Cargo features. Captured command output is in [evidence](evidence/RUN_RESULTS_001.json); historical failures are retained. The pre-edit focused `cargo test -p game_core --lib game::data::skill_data -- --nocapture` passed 13 tests (transcript evidence only).

| Run / command | Actual terminal result and disposition |
| --- | --- |
| check-001: `cargo check -p game_core --all-targets` | Failed: old direct constructor/match consumers; migrated exact wrapper fields. |
| check-002: same | Exit 0; intermediate compilation. |
| skill-data-001: `cargo test -p game_core --lib game::data::skill_data -- --nocapture` | 12 pass / 1 fail: pre-existing raw range-error text. Preserved old direct serde cause, retained enriched public loader diagnostics. |
| authoring-001: `cargo test -p game_core --lib game::ability::authoring -- --nocapture` | Exit 0, 4 pass; current cases included again in final full run. |
| check-003: all-target check | Failed: direct command envelope call sites; corrected area crit identity and direct test arguments. |
| check-004: all-target check | Exit 0. |
| runtime-001: new RON runtime test selector | Compile failure: Uuid versus UnitInstanceId test comparison; use explicit `.into()`. |
| runtime-002: `cargo test -p game_core --test skill_runtime_contract authored_literal_ron -- --nocapture` | Exit 0, 1 pass; subsequently expanded from four to six actual battles, included in final full run. |
| live-001: `cargo test -p game_core --lib game::data::live_ron -- --nocapture` | Exit 0, 80 pass; later added failed-publication test, included in final full run. |
| proc-001: `cargo test -p game_core --lib executed_proc_effect -- --nocapture` | Exit 0, 1 pass; real item binding/cast/source and context cleanup. |
| withdraw-001: `cargo test -p game_core --lib withdraw_preserves -- --nocapture` | Exit 0, 3 pass; basic/skill projectile and fixed area survival. |
| core-full-001: `cargo test -p game_core --lib --tests` | Exit 101, library 1,113 pass / 1 fail. Legacy-op rejection fixture still used old outer grammar; updated only wrapper, retained rejected opcode assertion. |
| core-full-002: same | Exit 0. Library 1,114; integrations 4+4+25+15+4: **1,166 pass**, no failed/ignored/filtered tests. [Exact inputs](VALIDATION_INPUT_001.json). |
| check-005: `cargo check -p game_core --all-targets` | Exit 0; final production and typed consumer compilation. |
| release-001: `cargo check -p game_core --release --lib` | Exit 0, optimized profile, 14.30 s. No debug-only production dependency. |

Before audit, added direct empty predicate/tag and missing-address rollback/empty-ID assertions to the existing authoring tests. Production bytes did not change. The subsequent core-full-003 completed with exit 0: 1,166 passed / 0 failed / 0 ignored / 0 filtered. Its result and [inputs](VALIDATION_INPUT_002.json) are recorded in the terminal evidence manifest; no running command is treated as passed.

The implementation-to-acceptance/dimension map is [ACCEPTANCE_EVIDENCE_001.md](ACCEPTANCE_EVIDENCE_001.md). [Scoped diff](evidence/R01_SCOPED_DIFF.patch) uses preserved pre-edit files rather than the unrelated dirty Git baseline. All 22 modified existing Rust files passed scoped rustfmt before mutation; three new Rust files were also formatted. [Format scope](evidence/FORMAT_SCOPE.json) records this without formatting the repository. Final preservation evidence distinguishes allowed R01 source/contract changes from parent operating-summary changes.

No required numeric performance SLA or external service baseline belongs to this Single Goal. Source-level memory/lifetime disposition uses existing bounded event entries and shared immutable source records. No personal save, network deployment, server or Unity test is claimed. Existing checkpoint/continuation, public application, catalog, item, preview, fragment, damage/HP gate and cancellation regressions run in the full core suite. Prior CRAR audit is historical intake evidence; current changed schema is supported by these new runs.
