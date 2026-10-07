# P02 Successor Currentness Audit 005

Verdict: `Complete / High — current / expected P06 codec dependency delta`

## Identity and Trigger

- Record: `ATR-CRAR-C2-P02-GEX20-005`; completion basis remains correction target
  `ATR-CRAR-C2-P02-GEX10-002`.
- Triggering successor: P06 adds direct `sha2 0.10.9` for the Core V5 canonical checkpoint checksum and restores V4/V5
  only through an already constructed validated-data-backed `GameCore`.

## Direct Current Evidence

- `src/game/data/mod.rs`, the canonical data contract and P02 receipt remain byte-identical at
  `74ed2bcf…7270`, `eca05b81…7ab` and `f12f85f4…e7ce`.
- The Core/workspace and standalone probe locks change only because `sha2` becomes a direct production dependency.
  No builder, bare database conversion, fixture visibility or loader/publication path was reopened.
- Fresh valid release probe loads embedded input into `ValidatedGameData` and constructs `GameCore`, exit 0. The invalid
  release probe still fails with `E0432/E0603/E0277/E0599` for builder/parts/bare-Arc/removed helper access.
- Full Core is 1,150/1,150 and `ron_loading` remains 25/25. P06 canonical V4 and V5 imports both validate their restored
  identity/reference graph against the current data context before publication.

## Disposition

P02 remains `Complete / High`. The checksum dependency is an expected subordinate codec delta; opaque validated
publication and production capability isolation are unchanged. No external compatibility claim was used.
