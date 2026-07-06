# Embedded Data Loader Ownership Contract Notes

## Initial Context

`GameDataBase::load_live_embedded()` is already the official embedded live RON loader for the main gameplay data bundle.

The remaining question is broader ownership:

```text
Should every embedded RON reader be folded into GameDataBase,
or should some static/domain policy data remain domain-owned?
```

This should be answered by code structure and dependency direction, not by a blanket rule that every `include_str!` is bad.

## Initial Classification Hypothesis

Likely `GameDataBase live domain`:

- abnormalities;
- enemies/corroded profiles and wave presets;
- employees;
- equipment;
- artifacts;
- consumables;
- buffs;
- skills;
- skill fragments;
- PVE encounters;
- rewards/shops/events;
- boss omen chains;
- run policy if it is treated as game-rule data.

Likely `domain-owned builtin policy/catalog` pending verification:

- map template database;
- map generation policy;
- combat preview battlefield generation policy;
- engine-level builtin buff data, if not part of live balance data.

Likely `test-only audit fixture`:

- live skill catalog audit manifest.

## Loader Inventory And Classification

Inventory command:

```text
rg -n "include_str!|load_live_embedded|from_ron_str" src tests ../game_server -S
```

Current classification before implementation:

| Loader / call site | Data file(s) | Classification | Decision |
| --- | --- | --- | --- |
| `GameDataBase::load_live_embedded()` | shops, rewards, events, boss omen chains, abnormalities, corroded employees/waves, employees, equipment, artifacts, consumables, buffs, skills, skill fragments, PVE encounters, run policy | `GameDataBase live domain` | Official production live bundle loader. Server and live RON tests must delegate here. |
| `../game_server/src/main.rs` `GameDataBase::load_live_embedded()` | main live bundle | `GameDataBase live domain` consumer | Correct production startup path. |
| `tests/common.rs` / `src/game/world/tests/mod.rs` `GameDataBase::load_live_embedded()` | main live bundle | test consumer of official loader | Correct live test path. |
| `MapNodeDefinitionDatabase::builtin()` | `map/node_definitions.ron` | `domain-owned builtin policy/catalog` | Keep outside `GameDataBase`; map generator owns node definition catalog. Validate with map/RON tests. |
| `MapGenerationPolicyData::builtin()` | `map/generation_policy.ron` | `domain-owned builtin policy/catalog` | Keep outside `GameDataBase`; map generator owns generation policy. Validate with map/RON tests. |
| `BattlefieldGenerationPolicyDatabase::builtin()` | `map/battlefield_archetypes.ron` | `domain-owned builtin policy/catalog` | Keep outside `GameDataBase`; combat preview owns battlefield archetype/size selection policy. Validate during preview generation and RON tests. |
| `BattlefieldTemplateDatabase::builtin()` | `map/battlefield_templates.ron` | `domain-owned builtin policy/catalog` | Keep outside `GameDataBase`; combat preview owns template selection and validation. |
| `RunPolicyData::builtin()` | `run/policy.ron` | domain-owned game-rule policy used by `GameDataBase` and minimal tests | Retain for now as an explicit policy-domain loader because many unit tests need policy without a full live bundle. Production still enters through `GameDataBase::load_live_embedded()`. |
| `BuffDatabase::live_default()` | `buffs/base.ron` | duplicate live domain loader | Remove/replace as public runtime loader. Buff live data should be owned by `GameDataBase`/test fixture helper, not by battle buff runtime. |
| `GameDataBuilder::live_defaults()` | currently live buffs + builtin run policy | test/minimal fixture helper | Retain but document as a fixture builder, not production live bundle loading. |
| `EventLogValidator::with_live_buff_data()` | currently live buffs | duplicate live domain loader | Remove if unused, or route through official game data if needed. |
| `tests/live_skill_catalog_audit.rs` manifest loader | `docs/audit/live_skill_catalog_manifest.ron` | `test-only audit fixture` | Retain; not runtime live game data. |
| `CorrodedWavePresetDatabase` test include | `enemies/corroded_wave_presets.ron` | `test-only direct schema assertion` | Retain as test-only coverage; production load remains `GameDataBase::load_live_embedded()`. |

Files under `../game_resources/data` that are not referenced by embedded loaders, such as legacy wrapper files, are outside this goal unless a runtime loader still consumes them.

## Policy Notes

- Embedded loading remains intentional.
- This goal should not introduce hot reload, mods, or filesystem data packs.
- If a retained domain-owned loader is production data, it must have explicit validation coverage.

## Follow-Up Candidates

- A later mod/data-pack goal may revisit this ownership model.
- If map templates become player-authored or mode-specific live content, they may need to move into the main live data ownership path.
