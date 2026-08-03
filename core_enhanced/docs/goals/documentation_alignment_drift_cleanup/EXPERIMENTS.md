# Documentation Alignment Drift Cleanup Experiments

Status: complete

## 2026-07-18 — Child Initialization

- Read Master G07 scope and final-shape dependency after G04–G06 completion.
- Confirmed the child path was available and no child dirty state existed.
- Mandatory child documents created before source/enum/vocabulary search or mutation.

## 2026-07-18 — Direct Enumeration And Correction

- Enumerated `GameplayRonSourceSet`, `LIVE_GAMEPLAY_RON_SOURCE_COUNT` and the embedded-source test:
  21 non-manifest gameplay sources.
- Enumerated Core's ten `MapNodeCategory` and four `TargetingProfile` variants, then inspected the
  current Unity category display mapping.
- Changed only current durable canonical: source count 23→21, current-domain `G15`/`G24` wording,
  three missing map categories and removed targeting-profile wording.
- Preserved historical goals, the excluded duplicate docs tree, production code, live data and tests.

### Validation

- Exact stale-term searches found no `manifest 외 23`, current `G15`/`G24`, or external
  `SplashClusterFirst` occurrence in the affected current canonicals.
- External `MapNodeCategory` inventory exactly lists all ten current Core variants.
- `cargo test -p game_core embedded_source_set_parses_all_versioned_gameplay_ron --quiet --
  --test-threads=1`: 1 passed, 0 failed.
- An initial `--exact` invocation selected zero tests; it was discarded and replaced by the executing
  filtered command above.
