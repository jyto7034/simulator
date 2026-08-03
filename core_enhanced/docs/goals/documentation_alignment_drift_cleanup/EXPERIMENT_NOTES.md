# Documentation Alignment Drift Cleanup Notes

Status: complete

## Starting Evidence

- Reaudit counted 21 non-manifest gameplay RON sources in loader/test while durable data docs said 23.
- Current runtime canonical used historical goal labels G15/G24 as domain names.
- External Unity canonical omitted three current map categories and listed removed
  `SplashClusterFirst`.
- G06 has finalized current weapon archetypes; G04 has finalized weapon profile fields.

## Resume Point

- Child complete. Resume at the Master combined review.

## Final Evidence

- `LIVE_GAMEPLAY_RON_SOURCE_COUNT` and its embedded-source test both state 21.
- `MapNodeCategory` is `Start`, `Combat`, `Support`, `Maintenance`,
  `HeadquartersContact`, `Event`, `Shop`, `Boss`, `Gate`, `Reward`.
- `TargetingProfile` is `DefaultForward`, `AirFirst`, `LowDefenseFirst`,
  `LowMagicResistFirst`; `SplashClusterFirst` is not a current variant.
- Unity's category label function preserves unknown category text, while the external enum inventory
  now enumerates the exact Core variants. No Unity production implementation was changed.
- Current `core_runtime_contract.md` contains no `G15`/`G24` domain vocabulary.
