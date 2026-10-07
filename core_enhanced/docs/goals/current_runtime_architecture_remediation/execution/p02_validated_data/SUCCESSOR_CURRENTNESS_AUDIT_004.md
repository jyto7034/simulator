# P02 Successor Currentness Audit 004

Verdict: `Complete / High — current / P04 strengthens release boundary`

## Identity and Trigger

- Record: `ATR-CRAR-C2-P02-GEX20-004`
- Completion basis: replacement target `ATR-CRAR-C2-P02-GEX10-002`.
- Triggering successor: P04 made `battle::core` default-release-inaccessible and moved battle body construction behind
  checked finite preparation while continuing to consume the P02 validated publication inside Core.
- Method: read-only P02 boundary re-audit against current bytes plus fresh positive and negative release probes. No P02
  implementation, data contract or receipt was changed.

## Direct Current Evidence

- `cargo check -p game_core --release`: exit `0`, warnings `0`.
- Actual valid release probe: embedded authoring input -> `ValidatedGameData` -> `GameCore` construction/run, exit `0`.
- Default-release invalid probe: expected exit `101` with `E0432`, `E0603`, `E0277`, `E0599`; builder, parts,
  bare-Arc conversion and removed combat helper remain inaccessible.
- Current public `GameCore::{new,new_with_*}` constructors still require `impl Into<ValidatedGameData>`. P04 removes
  the former standalone public `BattleCore::new_from_scenario` route in default release, strengthening rather than
  weakening the validated publication boundary.
- `src/game/data/mod.rs` remains
  `74ed2bcf2bed377d13b4e1442d2c35ee0b83f3e7d86f1f2e26c308b660997270`; canonical data contract remains
  `eca05b81d64e20afeb3d327dca7e77a780cb6511308fd9a4587d42c3cfd287ab`.
- P02 receipt remains `f12f85f489f422a8cdf131cd8e2a0aa1c05034a28572ef2a45181c8b438fe7ce`.
- Fresh full Core suite: `1,138` passed, failed/ignored `0`, including live publication and `ron_loading 25/25`.
- The explicit `test-fixtures` capability remains non-default test support and creates no default-release data path.

## Disposition

P02 remains `Complete / High`. P04 consumes the same opaque validated publication through the private Core battle
owner and removes an independently callable release kernel constructor. Loader semantics, data identities and the
positive/negative release boundary remain current. No external compatibility claim was used.
