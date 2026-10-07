# Phase 01 Test Reaudit

## Current Runner Results

All commands ran against the P00-frozen current dirty production tree.

| Command | Meaningful tests | Passed | Failed | Ignored/skipped | Runner-filtered nonmatches |
| --- | ---: | ---: | ---: | ---: | ---: |
| `cargo test -p game_core embedded_source_inventory_matches_every_physical_gameplay_ron_exactly -- --nocapture` | 1 | 1 | 0 | 0 | lib 929 + integration 45; zero-test bin targets |
| `cargo test -p game_core live_candidate_rejects_preview_failure_unique_to_manifested_difficulty -- --nocapture` | 1 | 1 | 0 | 0 | lib 929 + integration 45; zero-test bin targets |
| `cargo test -p game_core live_candidate_rejects_boss_binding_that_cannot_build_runtime_setup -- --nocapture` | 1 | 1 | 0 | 0 | lib 929 + integration 45; zero-test bin targets |
| `cargo test -p game_core --test ron_loading -- --nocapture` | 25 | 25 | 0 | 0 | 0 |
| Total meaningful executions | 28 | 28 | 0 | 0 | filtered counts are not passes/skips |

The three package-wide name filters each ran exactly one lib test; their listed zero-test binaries and filtered
integration targets are not counted as executed tests.

## Family Dispositions

| Family/test | Unique current invariant | Production reachability | Disposition | Safety net relation |
| --- | --- | --- | --- | --- |
| `embedded_source_inventory_matches_every_physical_gameplay_ron_exactly` | macro/direct includes exactly equal every physical production RON and instance inventory order | official embedded source constructor | `retain` | closes C01 exact-set requirement |
| `embedded_source_set_parses_all_versioned_gameplay_ron` | all current domain fields parse and remain consumed by the aggregate; fixed cardinality characterizes current set | same parser path | `retain` | distinct parse/field smoke; literal count is not sole authority |
| `empty_embedded_source_is_rejected_by_inventory_before_domain_parse` | source inventory owns empty-source failure/provenance | official parser pre-domain gate | `retain` | C01 failure ordering |
| Boss compiler unit matrix | raw graph/ID/parameter/result/loop/capability defects reject before publication | raw catalog compile called by live validation | `retain` | P01 compile contract; P06~P09 semantic handoff |
| live candidate Boss unknown/dangling/setup negatives | parse/compiler/PVE/setup failures reject whole candidate with exact source/domain | official `load_live_sources` | `retain` | P01/P05 cross-boundary safety net |
| manifested-only preview negative | baseline succeeds while one current manifested instance fails publication | official post-assembly candidate validation | `retain` | direct C02 closure |
| preview difficulty/setup unit tests | runtime difficulty resolution and typed setup feasibility | same production generator/executor functions | `retain` | P05 owns full setup assertion detail |
| `tests/ron_loading.rs` 25-test suite | actual embedded bundle/current content/reference contract | official `load_live_embedded` helper | `retain` | distinct integration layer; current 25/25 green |
| `live_skill_catalog_audit` | full published skill/reference behavior | production loader, domain P08/P09 | `retain`, P08/P09 review semantics | not duplicated by generic RON smoke |
| builder/builtin/cache tests | fixture/programmatic panic/default contracts | non-production but active test/server-test consumers | `retain` | separate lifecycle; not C03 production mapping proof |

## Candidate Safety-Net Conclusions

### C01

The exact physical/source comparison is stronger than the historical tautological count assertion and directly
failed if a production RON is added or omitted without the macro inventory. Parse-all and empty-source tests add
distinct parse/field/failure semantics. No test removal is justified by C01 closure.

### C02

The current manifested-only negative directly proves a case baseline validation would miss. Source inspection shows
the same generated preview reaches warning and full setup validation. Boss setup failure adds a distinct resolved
binding edge. The three layers are not duplicate.

### C03

Current green live tests protect published behavior but do not characterize every field/Arc/derived registry and
exact validation provenance across a future internal conversion. A private implementation Goal must add that
explicit parity table before consolidating mapping or retaining compiled Boss definitions.

## Cleanup Decision

No Phase 01 current test is a proven remove/merge candidate. The old parse-all fixed-count assertion is not the
inventory safety net by itself, but the test still uniquely touches the complete parsed aggregate. Removing its
literal or merging it into the exact inventory test can be considered only inside a future private C03/C01 cleanup
with assertion transfer; it is not required for current C01 closure.

## Omitted Runners

- Full `game::data::*`, combat-preview and workspace suites were not rerun. P00 proves production content identity
  matches the current Boss completion tree, and the focused current 28 tests directly cover P01 conclusions.
- Server/Unity runners are omitted because P01 changes no public contract and P10 owns external mapping.
- Formatter is omitted because production/canonical/test code was not changed.

