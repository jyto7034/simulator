# P03 Successor Currentness Audit 007

Verdict: `Complete / High — current / expected P07 compile-time projection correction`

## Identity and Trigger

- Record: `ATR-CRAR-C2-P03-GEX20-007`; predecessor currentness
  `ATR-CRAR-C2-P03-GEX20-006`.
- P07 correction changes the selected-event projection source and one combat snapshot assertion, both members of the
  historical P03 impact surface. It does not change the P03 executor or commit port.

## Direct Current Evidence

- `src/game/world/application.rs` remains `7d9c8f70…df1`; one `commit_application` remains the sole live-root writer.
- Runtime test/benchmark switch search is zero after replacing `cfg!` with compile-time-isolated constants. Default
  release and `test-fixtures` release both build.
- Combat-result snapshot 1/1 and application/authority 9/9 pass; authority valid caller exits 0 and invalid caller has
  expected exit 101; full Core passes 1,159/1,159.

## Disposition

P03 remains `Complete / High`. Transaction staging, rejection equality, retry behavior and writer ownership are
unchanged; the changed projection path has direct equivalent-configuration evidence.

