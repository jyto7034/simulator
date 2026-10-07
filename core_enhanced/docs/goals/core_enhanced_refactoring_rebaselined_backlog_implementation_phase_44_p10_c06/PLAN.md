# Phase 44 CRA-P10-C06 Core Package Targets And Direct Dependencies Cleanup Goal

Status: complete
Implementation Complete: true
Goal Type: phase
Execution Mode: implement_and_verify
Production Mutation Authorized: yes — the user's 2026-08-11 remaining-cleanup approval explicitly includes
verified package/tool/public breaking cleanup after current standalone/consumer inventory and retained-tool controls
Parent Master: `core_enhanced_refactoring_rebaselined_backlog_implementation_master`
Phase: 44 of 44
Candidate / Work Type: CRA-P10-C06 / `cleanup`
Priority Axes / Known Subtotal: `2 / 1 / 2 / 3 / 2 / 2 / unknown = 12/18`
Direct Linear Dependency: Phase 43 CRA-P09-C07 `Complete / High`
Direct Technical Dependencies: Phase 30~34 P10-C01/C04/C05/C02/C03, Phase 42 P06-C07 and Phase 43 P09-C07 are
`Complete / High`; Phase 35 P10-C07 is also complete
Direct Policy: CRA-POL-006 plus the user's explicit remaining package/tool cleanup approval
Repository / Start Commit: `/mnt/f/work/simulator` / `f318414fb0156a67e04c5479000c2a2683fc0c09`
Initialization Date: 2026-08-11 (`Asia/Seoul`)

## Objective

Reconstruct the current Core Cargo target, feature, dependency, logger/hook, fixture-generator and standalone
consumer graph, then remove or normalize only proven package noise. Preserve the Core library API and behavior,
server path dependency, every retained fixture command and byte/content output, required feature behavior and all
supported standalone use, except the explicitly approved removal of a proven-unconsumed package logger/init API.
Preserve all gameplay/domain library API and behavior. Make retained runnable tools explicit and keep them out of
test-harness discovery where that is the current intended contract.

This Phase does not change gameplay/runtime/data/wire semantics, regenerate or overwrite checked-in consumer
fixtures, invent a new build/CI system, remove a tool from negative runtime-reference evidence alone, or clean
unrelated warnings.

## Initialization And Activation Gate

Before activation or current package/source inspection:

1. read this PLAN, `EXPERIMENTS.md` and `EXPERIMENT_NOTES.md` completely;
2. verify Phase 43's 210-line audit at `a9b674b4...` is `Complete / High` and Master Phase 44 is `ready`;
3. retain original P10-C06 package evidence, post-Boss reaudit and final catalog/order/wave/test records as
   historical hypotheses rather than current facts;
4. register this Child as the sole initialized Phase in Master and Child documents;
5. reread all initialized documents, then activate exactly one Child; and
6. after active reread, capture filename-only prospective fingerprints/status for manifests, lock/config, target
   roots, fixture bins/examples, logger/hook surfaces, source/test references, scripts/docs, accessible standalone
   consumers and protected marker before opening current candidate content.

No dependency, target, hook, logger or fixture tool is removed because an old audit or workspace search called it
unused.

## Historical Evidence To Refresh

- Original P10-C06 evidence reported no Core build script despite four build dependencies, a broad direct dependency
  set with no lexical use, an empty auto-discovered default binary, a no-op ctor hook, fixture bins participating as
  zero-test harnesses, Core logging without a workspace caller and unconditional `tokio_unstable` without Core Tokio
  use.
- The target model was a minimal explicit package: retain runnable fixture tools and output, remove empty/default
  graph and confirmed-unused direct/build dependencies, and mark generators non-test.
- Post-Boss reaudit found the package noise still present and no public Boss wire, but kept standalone Core API,
  logger and generator consumers as an external gate.
- Final order puts P10-C06 last because P10-C01~C05, P06-C07 and P09-C07 stabilize retained code/tool surfaces first.
  Every dependency is now complete.
- Phase 35 already removed only the disconnected empty `tests/test_actor/mod.rs`; this Phase must not conflate that
  completed test-root cleanup with current package normalization.
- The user later approved remaining package/tool/public breaking cleanup after current inventory, without a shim.

Historical dependency names and empty-target claims locate questions; they do not prove current use, compiled
feature reachability, tool ownership or removal scope.

## Current Questions To Resolve

1. What exact library, default binary, explicit/auto bin, example, integration test, bench and build-script targets
   does current Cargo metadata expose under default/all-target/feature modes?
2. Which current direct, dev and build dependencies are selected in the resolved graph, and which are actually
   referenced by library, bins, examples, integration tests, build scripts, macros, cfgs or generated code?
3. Does a current `build.rs`, proc-macro side effect, linker registration, ctor hook, logger surface or Cargo config
   have an observable standalone/server/tool effect?
4. Which fixture generators are authoritative retained tools, what commands/names/features do consumers use, and
   what exact outputs do they produce to disposable `/tmp` destinations?
5. Do root scripts, docs, CI, server, accessible adjacent repositories or user workflows invoke the default Core
   binary, logger API, package features or implicit bin discovery?
6. Is `tokio_unstable` global, Core-local or required by any retained dependency/target, and can its scope be
   narrowed without changing server/runtime behavior?
7. Do dependency changes alter features/default features, target-specific resolution, lockfile packages, license/
   platform support or standalone commands?
8. Which surfaces are package metadata only versus public Rust/API/tool compatibility, and what exact approved
   breaking migration is required?

Any meaningful current consumer narrows removal. Unknown deployed use is recorded honestly; the user's explicit
package cleanup approval permits removal only after the accessible inventory and retained-tool replacement are
complete.

## Required Package Authority Model

The target authority is:

`workspace/root Cargo configuration`
→ `minimal Core package manifest with explicit targets/features`
→ `dependencies justified by a retained compiled target`
→ `Core library + server path consumer + explicit standalone fixture tools`
→ `current output and behavior contracts`.

Cargo metadata, not filename convention or historical scripts, determines target membership. Source/compiled graph
and consumer inventory together determine dependency/tool retention; lexical absence alone is not sufficient.

## Preserved Invariants

### Library and server

- Core gameplay/domain public surface, feature semantics, gameplay, serde shapes, test discovery and runtime
  behavior remain exact. Only the proven-unconsumed logger/init package API may be removed under the user's explicit
  public-cleanup approval.
- `game_server` continues to resolve and compile the Core path dependency in debug/release and all-target modes.
- No normal runtime/logging or feature behavior is silently moved from Core to server or vice versa.

### Tools and fixtures

- Every retained generator keeps its stable Cargo target/command and exact semantic output.
- Generator runs use new disposable `/tmp` destinations; checked-in/reference/Unity fixtures are not overwritten.
- Runnable generators are not counted as zero-test harness coverage after normalization.

### Dependency and platform graph

- Removing a dependency must remove only an exact unused direct edge; transitive packages may remain for other
  owners and are not force-pruned.
- Default/features, build profiles, target triples, linker flags and lockfile behavior stay exact except the
  intended removal of unowned package edges/config.
- No new aggregator dependency, feature alias, compatibility bin, wrapper script or placeholder target is added.

### Dirty work and scope

- Existing user/completed-campaign changes remain attributed by activation fingerprints.
- Protected marker `game_server/logs/app.log.2026-08-09` remains exact.
- Master final completion audit is a separate step after this Child; this Phase does not mark the Master complete.

## Prospective Current Evidence Paths

Historical carrier classes are prospective until post-activation filename/hash capture:

- workspace/Core/server manifests, lockfile and Cargo configuration;
- Core library/default-main roots, logging/ctor/build surfaces and all bin/example/test target roots;
- server path dependency and any scripts/docs/CI that invoke Core package targets or features;
- retained fixture outputs, reference locations and accessible external/Unity/tool consumers;
- metadata/dependency-tree outputs under default/all-target/feature modes; and
- runtime/gameplay/data/server canonicals plus protected marker.

The active Phase must replace this class list with exact current paths/hashes/status before content inspection.

## Implementation Sequence

### P44-1 — Current metadata, source and consumer inventory

Capture Cargo metadata/target names, direct/resolved dependency graph, all source/test/tool references, config/linker/
feature effects, standalone commands and accessible consumers. Classify every target/dependency/surface as retained,
removed, made explicit, scope-narrowed or blocked.

### P44-2 — Pre-cleanup tool and build baselines

Before manifest/source mutation, record:

- exact Core/server metadata and target list;
- Core/server debug/release all-target and feature checks;
- sorted test lists/counts;
- every retained generator run to a fresh `/tmp` destination, with output path/size/hash/content validation;
- any logger/hook/default-bin consumer behavior that must be retained; and
- dependency tree/direct-edge fingerprints.

Every mandatory test/filter/command must actually select or run its claimed target.

### P44-3 — Minimal approved package cleanup

Make retained runnable tools explicit and non-test, remove only proven empty/no-op target or hook surfaces and exact
unused direct/build dependencies/config. Do not delete a retained generator, rename its target or regenerate a
consumer fixture. Avoid lockfile churn beyond resolved consequences.

### P44-4 — Exact after-parity and legacy absence

Repeat metadata, target, dependency, feature, standalone and fixture-output controls. Search for removed dependency/
target/config references, implicit replacement targets, compatibility wrappers and stale docs.

### P44-5 — Broad/static validation

Run Core and server debug/release all targets, workspace debug/release checks, relevant feature/no-default-feature or
target-specific checks established by inventory, normal Clippy, format/diff, metadata/dependency/test fingerprints,
canonical/manifest/status/marker hashes and every retained fixture invocation.

### P44-6 — Independent exit audit

Create `COMPLETION_AUDIT.md`, independently apply `goal_completion_review_guide.md`, rerun direct metadata/tool/
fixture controls and broad debug evidence, and require `Complete / High` before the Master final audit starts.

## Minimum Validation Contract

- exact before/after `cargo metadata` package-target/feature comparison;
- direct/resolved dependency graph comparison and exact removed-edge search;
- Core library/server path dependency debug/release all-target tests/checks;
- feature/default-feature/config checks required by current inventory;
- every retained Core fixture generator invoked to a fresh `/tmp` destination with exact output validation;
- standalone/root script/doc/accessibile external consumer inventory;
- zero-test harness list/count before and after;
- `cargo fmt --all -- --check`, `git diff --check` and normal Core/server Clippy;
- sorted Core/server test-list fingerprints, manifests/lock/config/canonical/status hashes; and
- protected-marker verification.

## Omitted Validation

- Unity build/runtime is omitted only if current inventory proves retained fixture formats/commands are unchanged and
  disposable generator outputs match current references; otherwise it becomes mandatory or a recorded block.
- Network/platform matrix builds are omitted unless a removed dependency/config is target-specific or current docs/
  CI claim those targets.
- Performance benchmarks are omitted because this Phase must not alter runtime code paths; a runtime delta stops the
  Phase.
- Master-wide cross-Phase policy/legacy validation is deferred to the separate Master final audit, not skipped.

## Completion Conditions

- Every current Cargo target, direct/build/dev dependency, config/feature, logger/hook and fixture tool is classified.
- Accessible standalone/server/script/doc/adjacent consumer inventory is complete.
- Retained tools have pre-cleanup output baselines and exact post-cleanup parity from disposable destinations.
- Only proven package noise is removed or normalized; no compatibility wrapper or implicit replacement is added.
- Core library/server behavior, features, tests, target names and retained fixture outputs remain exact.
- Metadata/dependency graph reflects the minimal justified direct edges and explicit tool targets.
- Both-profile broad/static/standalone checks are green with exact counts and hashes.
- Existing dirty work and protected marker are preserved.
- A separate completion audit returns `Complete / High`.

## Completion Audit Evidence Matrix

| ID | Condition | Required evidence | Status |
| --- | --- | --- | --- |
| P44-C01 | current package/target graph is exact | metadata targets, kinds, features and auto-discovery | pass |
| P44-C02 | dependency/config graph is exact | direct/build/dev/resolved edges, source/cfg/linker users | pass |
| P44-C03 | dirty work is preserved | activation/current fingerprints, scoped diffs and marker | pass |
| P44-C04 | every candidate surface is classified | target/dependency/hook/logger/tool disposition matrix | pass |
| P44-C05 | standalone/external consumer gate is exact | server/scripts/docs/adjacent/deployed classification and approval | pass |
| P44-C06 | retained tools are characterized before cleanup | commands, outputs, sizes/hashes and semantic checks | pass |
| P44-C07 | only package noise is removed | exact manifest/source/config delta; no wrapper/shim | pass |
| P44-C08 | metadata/dependency target result is exact | before/after package, target and edge parity/delta | pass |
| P44-C09 | fixture and feature behavior is exact | retained commands/output parity and feature checks | pass |
| P44-C10 | library/server/test behavior is exact | Core/server both-profile tests and list parity | pass |
| P44-C11 | broad/static validation is green | workspace, Clippy, format/diff and hashes | pass |
| P44-C12 | independent audit/Master boundary is current | `Complete / High`; Master final audit remains separate | pass |

## Stop Conditions

Stop and do not mark Phase 44 complete if a target/dependency/logger/hook has an unhandled consumer; a retained tool
name/output changes; required feature/platform behavior is lost; a wrapper/alias is needed; lockfile drift is
unexplained; checked-in/external fixtures would be overwritten; a mandatory target/test selects zero; runtime/data/
wire behavior changes; dirty overlap cannot be isolated; the marker changes; or independent audit is below
`Complete / High`.

## Initialization Dirty Baseline

- Expanded status before this Child: 603 rows at canonical C-sort SHA-256
  `1510b29c67ffd08be65ee8590088e6836652c98135559dd232813ae907a33847`.
- Phase 44 path was absent and active implementation Child count was zero.
- Phase 43 completion audit is 210 lines at SHA-256
  `a9b674b4c52f58ad228f4214584f2d9f3c331e4054489ebaf238ee2a0f7a2acb`, verdict `Complete / High`.
- Protected marker is 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`.
- These three new Child documents are the only expected status-set additions before activation evidence.
