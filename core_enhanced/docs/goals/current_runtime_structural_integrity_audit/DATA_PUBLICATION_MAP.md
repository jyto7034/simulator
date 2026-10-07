# Data Deserialize, Validation, Compile and Publication Map

- 기준선: `B-001`
- 상태: 최종

## P-001. Official embedded live path

```text
include_str GameplayRonSourceSet
  -> parse_gameplay_ron_sources (source-aware parse)
  -> ParsedGameplayRon::into_validated
  -> ValidatedGameplayRon { parsed, compiled boss runtime }
  -> private GameDataBase::from_validated_live_catalogs
  -> generated combat-preview validation
  -> Result<Arc<GameDataBase>, LiveRonError>
```

- typed/source-aware error detail and validation coverage: follow-up bounded C07 investigation
- partial publication: no `Arc<GameDataBase>` returned before full path success in observed top-level function

### Source and validation coverage

- manifest: exact schema version 1, unknown/missing fields rejected
- 24 compile-time sources: shops, rewards, events, VN/triggers, boss omen/runtime/summons, abnormalities/corroded profiles,
  employees/equipment/consumables/buffs/skills/fragments, encounters/policy, map and battlefield catalogs
- parse: source/domain/nested field path, trailing-token rejection
- validation/compile: per-domain contracts + catalog key/UUID/pool uniqueness + cross-catalog references + RunStart/extra-wave/
  map/fixed authoring + summoned/boss compile + generated preview contract
- publication parity test compares parsed catalogs with the published database and derived item registry
- evidence: E-013 and E-002

## P-002. Public builder/parts path

```text
GameDataBuilder::empty/with_*
  -> build/build_arc
  -> public GameDataBase::new(GameDataBaseParts)
  -> validate_indexes (duplicate paths can panic)
  -> expect-based cross-reference/policy checks
  -> bare GameDataBase
```

- no validated/unvalidated type distinction at `GameCore::new(Arc<GameDataBase>, seed)`
- can omit runtime-required RunStart trigger and defer error into mutating StartNewGame
- findings: RUNTIME-AUDIT-006/007

## P-003. Audited embedded bytes

- all manifest + 24 source files exist and are non-empty
- exact size/hash inventory: `LIVE_DATA_INVENTORY.md`
- B-001 compile/test evidence: E-002; boundary evidence: E-013/E-014

## P-004. Direct production consumer

- adjacent server startup calls `GameDataBase::load_live_embedded()` and maps failure to `io::ErrorKind::InvalidData`.
- no inspected production server path uses `GameDataBuilder`; builder callers observed are core/server tests.
- this positive reachability reduces current live incidence but does not type-restrict the public alternate constructor,
  so C07 remains FAIL on RUNTIME-AUDIT-007.
