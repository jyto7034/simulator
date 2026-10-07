# Phase 35 CRA-P10-C07 Empty Disconnected Test Module Cleanup Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — the user's 2026-08-09 instruction to execute the Master campaign
Parent Master: `core_enhanced_refactoring_rebaselined_backlog_implementation_master`
Phase: 35 of 44
Candidate / Work Type: CRA-P10-C07 / `cleanup`
Direct Dependency: Phase 34 CRA-P10-C03 `Complete / High`
Direct Policy: none; CRA-POL-006 remains the campaign-wide compatibility invariant
Repository / Start Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Initialization Date: 2026-08-11 (`Asia/Seoul`)

## Objective

Remove only the empty disconnected `core_enhanced/tests/test_actor/mod.rs` artifact, after current-tree proof that
it contains no assertion/setup/code, has no module edge and is absent from Cargo's test targets. Preserve Cargo
metadata, discovered test listing, every real integration/inline test and all production behavior exactly.

## User/Contract-Visible Problem

Original and post-Boss P10 audits found a file that looks like a test module by path but supplies no test target,
module, fixture, assertion or invariant. Retaining it creates false test-graph evidence and maintenance noise. The
approved cleanup removes the empty artifact only; it must not be combined with the broader package/zero-test target
cleanup reserved for CRA-P10-C06.

## Audit Finding ID

CRA-P10-C07 and test-cleanup row CRA-TC-12. Both original and post-Boss evidence classify it `still_valid`,
`cleanup`, priority Tier E, known subtotal `3/18`, `implementation_ready` and requiring unchanged metadata/test
listing plus a green Core runner. No behavioral replacement is required for a disconnected empty file.

## Current Runtime And Live Data Evidence

Historical/final evidence identifies the prospective artifact at `core_enhanced/tests/test_actor/mod.rs`, described
as only a newline/two-byte empty module, absent from Cargo metadata and without a module edge. Current path-only
initialization checks confirm that the path exists and is clean, but no Phase-35-directed content, module graph,
metadata, test listing, source, data or canonical inspection has occurred yet.

Current emptiness, directory membership, module reachability, metadata targets and exact test listing are discovery
pending after sole activation and prospective fingerprinting. Historical audit statements are inputs, not current
proof.

## Source Of Truth And Ownership

### Current prospective authority

- Cargo manifests and Rust module edges determine the executable test graph.
- Inline `#[cfg(test)]` modules and Cargo-recognized integration-test roots own actual tests.
- The prospective empty file owns no known behavior, but that must be re-proven on the activated current tree.

### Target authority

Cargo's actual targets and reachable Rust modules remain the complete test graph. The disconnected empty artifact
and its directory, if empty after the file removal, cease to imply a nonexistent test family.

No replacement module, manifest entry, placeholder, compatibility file or ignored test is created.

## Invariants

- Cargo package/target metadata is byte/structure equivalent before and after, except filesystem absence is not a
  Cargo target and therefore must produce no metadata delta.
- The complete discovered Core test name list and nonzero test count remain exact.
- Every real integration root, inline test module, fixture binary and production target remains present.
- Core debug/release behavior remains green with actual nonzero counts.
- No manifest, production Rust, data/RON, DTO/wire, save/checkpoint, server, Unity or canonical file changes.
- No CRA-P10-C06 package dependency, empty main or generator-harness cleanup is pulled forward.
- Existing user/campaign dirty work and the protected Phase-27 approval marker remain exact.

## Refactor Candidate And Behavior-Preservation Boundary

- Candidate ID: CRA-P10-C07.
- Work type: `cleanup`; `still_valid`; Tier E; implementation ready.
- Priority axes: ownership `1`, correctness/atomicity `0`, lifecycle `0`, change amplification `1`, test fragility
  `1`, failure blast radius `0`, change frequency `unknown`; known subtotal `3/18`.
- Removal scope: only `core_enhanced/tests/test_actor/mod.rs` and the containing directory only if it is empty after
  removal.
- Retained authority: Cargo target discovery, all reachable test modules, all real test bodies and fixtures.
- First safety net: exact current metadata target table and complete test listing/count before removal, plus direct
  proof of whitespace-only content, no module edge and no sibling artifact.
- Behavior-preservation boundary: any metadata target, test name/count, build graph, production behavior or consumer
  difference stops the cleanup.

## Approved Policy And Evidence

- Original P10 C07 evidence: the file is empty, not a Cargo target and has no module edge; delete only that
  file/directory and preserve test discovery/count.
- Post-Boss evidence: the same artifact remains disconnected and implementation-ready; Boss runtime adds no
  dependency.
- Final test catalog: remove it with before/after metadata and test-list equality; no behavioral replacement is
  needed.
- Final linear order: item 35 starts Stage D after Phase 34 and has no entry dependency. It was moved from old Wave
  0 because it enables no later correction, not because risk changed.
- CRA-POL-006 remains a negative invariant: do not let this private empty-file cleanup alter public/package/runtime
  compatibility or expand into CRA-P10-C06.

## Open Policy Decisions

None at initialization. If the current file contains code, is reachable, is a Cargo target or has an unknown
consumer/sibling role, stop rather than treating historical emptiness as deletion authority.

## In Scope

- current-tree byte/content classification of the one prospective file;
- exact directory membership and Rust module/caller edge inventory;
- Cargo metadata package/target inventory before and after;
- complete Core test listing/name/count capture before and after;
- deletion of only the proven-empty disconnected file and empty directory;
- Core focused/broad debug/release validation proportionate to the test-graph boundary;
- scoped diff, status/hash and protected-marker preservation;
- separate completion audit.

## Out Of Scope

- any real integration or inline test deletion, merge, assertion transfer or rename;
- `src/main.rs`, fixture-bin harness, dependency, feature, logger, ctor or `tokio_unstable` cleanup from P10-C06;
- public/API, package layout, Cargo manifest/workspace membership or external tool changes;
- production runtime, gameplay, data/RON, DTO/wire, save/checkpoint, server or Unity changes;
- ignored tests, replacement placeholders, compatibility modules or broad formatting;
- Phase 36 mission/helper/test cleanup or any later Stage-D candidate.

## Implementation Scope

After activation and current-tree proof:

1. fingerprint the target file/directory, manifests and likely test-list inputs without changing them;
2. prove the file is whitespace-only, the directory contains no other artifact and no `mod test_actor`/path/include
   edge reaches it;
3. capture normalized Cargo package/target metadata and the complete Core test listing/count as pre-removal
   artifacts or hashes;
4. delete the file with an auditable patch and remove the directory only if it is then empty;
5. recapture and compare metadata/test listing exactly;
6. run the Core runner and proportionate debug/release/static gates;
7. prove no manifest/source/data/contract/external path changed and preserve dirty work;
8. perform an independent completion audit before closing the Phase.

## DTO, Data, Save And Migration

- DTO/wire migration: none.
- Live RON/data migration: none.
- Save/checkpoint migration: none.
- Public Rust/package migration: none; the disconnected test artifact is not a compiled module or target.
- Unity/external migration: none; no external consumer is implicated by a private unreachable empty path.
- Compatibility: no alias, stub module, ignored test, dual path or fallback is approved.

## Deployment And Compatibility

Core and server deployment artifacts must be identical because the removed path must not enter Cargo's graph.
Before/after metadata and test-list equality are the direct proof. No coordinated rollout or external repository
change is permitted. Unknown standalone/package consumers are irrelevant only if current proof confirms the path is
not a public target/module; otherwise the Phase stops.

## Focused Validation

Required after discovery:

- byte/content and directory-member proof for the exact target;
- exhaustive workspace module/path/include search with no reachability edge;
- normalized `cargo metadata --format-version 1 --no-deps` package/target equality before/after;
- complete Core `--list` name/count equality before/after, with zero-test harnesses classified separately;
- deletion diff proving no other path was removed.

## Broad Validation

At minimum after the removal:

- `cargo test -p game_core --all-targets` with actual nonzero pass/fail/ignored counts;
- representative release Core all-target validation;
- workspace all-target checks in debug/release if target discovery could differ by profile;
- `cargo fmt --all -- --check`, candidate-scoped diff/static audit and normal Clippy diagnostics as proportionate
  static gates;
- final expanded status/path hash and protected-marker check.

Stage-D repository-wide final validation remains a later Master boundary and is not substituted by this Phase's
Core proof.

## Omitted Validation

- game_server behavior tests are provisionally omitted because a disconnected Core test file cannot affect the
  server target graph; reconsider if metadata shows otherwise.
- Live RON suites are included only insofar as Core all-target runs already execute them; no data-specific extra
  matrix is required.
- Unity, WebSocket and performance validation are omitted because no compiled runtime, wire, consumer or hot path is
  changed.
- Canonical edits are omitted because removing a nonexistent test-graph artifact changes no durable contract.

## Completion Conditions

- Current evidence proves the target is whitespace-only, unreachable, absent from metadata and the sole directory
  member.
- Only that file and the now-empty directory are removed.
- Normalized Cargo package/target metadata is exact before/after.
- Complete Core test names/count and actual runner behavior are exact before/after.
- No real test/assertion/fixture, production path, manifest, contract or consumer changes.
- No compatibility stub, ignored test, placeholder or CRA-P10-C06 work is introduced.
- Focused/broad debug/release/static validation is green with actual counts and classified zero-test targets.
- Dirty work and the protected marker are preserved.
- A separate completion audit returns `Complete / High` with no material finding.

## Completion Audit Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P35-C01 | current target is exactly empty and disconnected | bytes, directory listing, module-edge and metadata inventory | complete — audit pass |
| P35-C02 | dirty targets and concurrent work are preserved | pre/post status, scoped diff, hashes and protected marker | complete — audit pass |
| P35-C03 | pre-removal test graph is characterized | normalized metadata and complete Core test-list artifact/count | complete — audit pass |
| P35-C04 | only the empty artifact/directory is removed | deletion diff and no sibling/manifest/source change | complete — audit pass |
| P35-C05 | metadata and test discovery remain exact | before/after normalized equality and hashes | complete — audit pass |
| P35-C06 | runtime/test behavior remains exact | Core debug/release actual counts | complete — audit pass |
| P35-C07 | no compatibility/placeholder/adjacent cleanup remains | static scope and legacy audit | complete — audit pass |
| P35-C08 | DTO/data/save/public/external contracts are unchanged | unchanged prospective hashes and scope proof | complete — audit pass |
| P35-C09 | focused/broad/static validation is green | actual counts and failure classification | complete — audit pass |
| P35-C10 | independent completion audit is current | `Complete / High` review and next-Phase boundary | complete — audit pass |

## Activation Protocol

Before Phase-35-directed target-content, module-graph, metadata or test-list inspection:

1. Master records Phase 34 `complete — Complete / High`, Phase 35 as the sole `ready` Child and Phases 36~44
   pending.
2. Change all three Child headers from `initialized` to `active`; `Implementation Complete` remains `false`, and
   change Master Phase 35 from `ready` to `active`.
3. Fully reread this PLAN, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`, the Phase-34 completion audit, original/post-Boss
   P10-C07 evidence, Final catalog/order/test inputs and Master activation contract.
4. Capture HEAD, expanded status count/canonical C-sort hash and prospective hashes/status for the target,
   directory, manifests and relevant test-list inputs using path discovery only, without reading target content or
   executing Cargo metadata/tests.
5. Record the activation evidence in the Child before current-tree content/reachability/metadata investigation.

Historical audit observations do not waive this gate.

## Stop Conditions

Stop and do not activate Phase 36 if the file is nonempty; any module/path/include/build edge reaches it; Cargo
metadata exposes it; another directory member would be removed; metadata or test listing changes after deletion;
any real test/assertion/fixture is lost; cleanup requires a manifest/public/package/consumer decision; dirty overlap
cannot be separated; required tests fail or select zero behavioral tests; the protected marker changes; or the
independent audit is not `Complete / High`.

## Initialization Dirty Baseline

- Expanded porcelain status before this Child: 543 rows.
- Canonical C-sort SHA-256:
  `04f967cbbf1377c1d156530ffe7776d5e2b31d4f828b2efcd3f49cc0d6c07c9b`.
- The target path exists and is clean by path/status-only initialization checks; content remains unread.
- Protected marker: 57 bytes, SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`.
- This new Child directory is the only expected status-set addition before activation evidence is recorded.
- All existing rows are user/campaign-owned and must be preserved.
