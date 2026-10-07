# Core Enhanced Refactoring Audit Phase 01 — Boundary Audit

Audit Date: 2026-08-07
Status: investigation complete; phase-exit audit pending
Execution Mode: investigation_only
Production Mutation: none authorized or performed

## Scope Reconciliation

| Surface | Current evidence | Phase disposition |
| --- | --- | --- |
| Core data modules | `src/game/data/**`: 19 files | primary, fully inventoried |
| Domain-owned typed catalogs | `battle/buffs.rs`, map catalog types in `map/{facility,types}.rs`, battlefield catalogs in `combat_preview/mod.rs` | shared primary loading/validation seam |
| Production source set | `GameplayRonSourceSet::embedded()`: 1 manifest + 22 non-manifest fields | primary |
| Production live files | `game_resources/data/**`: 23 RON files | confirmed supporting data |
| Startup consumer | `game_server/src/main.rs::load_game_data_from_ron()` | confirmed supporting consumer |
| Published owner | server `AppState` → actor message → `GameCore::game_data: Arc<GameDataBase>` | confirmed direct publication/consumer seam |
| Focused integration tests | `tests/ron_loading.rs` (24), `tests/live_skill_catalog_audit.rs` (4), `tests/common/mod.rs` loader helper | primary safety net |
| Focused unit tests | `game::data::*` (162 selected tests) | primary parse/validation safety net |
| Preview behavior tests | `game::combat_preview::*` (27 selected tests) | shared safety net for generated-publication gate |
| Canonical | `docs/README.md`, `docs/data_loading_contract.md` | current durable routing/data contract |
| `config/game_balance.toml` | file exists; no workspace path/symbol/generic Core loader reference found | unverified external-consumer signal |
| Domain gameplay consumption | run/economy/event/combat calculations after publication | assigned to Phases 2–9; not over-audited here |
| Unity and other external consumers | not read; Master-approved exclusion | out-of-scope/unverified |

The 19 Core data files are the 16 domain catalog modules plus `mod.rs`, `live_ron.rs`, and
`validation.rs`. Buff and five map/battlefield sources are typed by their domain modules but are parsed through
the same Phase 1 source set. Physical type location therefore does not create a second production loader.

## Exact Embedded/Live File Reconciliation

The exact comparison extracted every `game_resources/data/**` path from the body of
`GameplayRonSourceSet::embedded()` and compared it with `rg --files game_resources/data`.

```text
embedded referenced files: 23
production live files:     23
symmetric difference:      empty
```

The one manifest is `data/bundle.ron`; the remaining 22 files are gameplay catalog sources. Current runtime
inclusion is therefore complete. This current equality does not prove that future newly authored files cannot be
left unreferenced: no checked-in test compares the two sorted sets, and the manifest itself contains only
`schema_version: 1`.

## Startup To Publication Trace

1. `game_server/src/main.rs::main()` loads settings and logger/retry state, then calls
   `load_game_data_from_ron()` before opening persistence stores, constructing `AppState`, binding HTTP, or
   starting WebSocket service.
2. The server wrapper delegates only to `GameDataBase::load_live_embedded()` and maps `LiveRonError` to
   `std::io::ErrorKind::InvalidData`.
3. `GameplayRonSourceSet::embedded()` supplies compile-time `include_str!` values. Runtime file-system hot load,
   alternate data-pack load, manifest fallback, and schema alias are absent.
4. `parse_gameplay_ron_sources()` validates the exact manifest version, parses each typed catalog with
   `serde_path_to_error`, and returns private `ParsedGameplayRon`.
5. `ParsedGameplayRon::validate_domain_catalogs()` runs domain contracts, top-level ID/UUID/pool uniqueness,
   cross-item UUID, cross-catalog references, map catalogs/payloads, RunStart policy, difficulty extra waves, and
   reward/event/shop/employee/PvE relationships.
6. `GameDataBase::load_live_embedded()` converts those validated values into a private candidate
   `Arc<GameDataBase>` through `from_validated_parts()`.
7. The candidate runs generated combat-preview/briefing validation. Only after that returns `Ok` does the loader
   return the `Arc` to the server.
8. `AppState`, actor creation, and `GameCore` clone the same immutable `Arc<GameDataBase>`; no second server-side
   data calculation or publication was found in this seam.

There is no partial bundle publication. Validation is fail-fast rather than error-aggregating; this is an explicit
current failure semantic, not silent recovery.

## Parse And Validation Coverage

| Layer | Authority and behavior | Evidence/result |
| --- | --- | --- |
| Manifest | `LiveRonBundleManifest`, exact version 1, `deny_unknown_fields` | missing/unknown/wrong version unit tests pass |
| Source-aware parse | `parse_live_ron<T>()`, `serde_path_to_error`, trailing input check | source/domain/nested field-path unit test passes |
| Top-level schemas | direct strict database type or custom strict raw type | all 22 sources parse in embedded source-set test |
| Domain validation | catalog `validate_contract()`/custom deserialize validation | data-module inventory found validators and focused tests across every catalog family |
| Key uniqueness | explicit typed source-aware checks plus index construction | duplicate IDs, UUIDs, pool IDs, and cross-item UUID tests pass |
| Cross-catalog references | `data/validation.rs` and `ParsedGameplayRon` orchestration | typed negative tests cover skill/buff, fragment, abnormality, PVE, event, shop, reward, map, trigger, omen, and starter seams |
| Map catalogs | parse once into `GameplayMapCatalogs`, validate before publication | five live files and typed contract validators confirmed |
| Generated preview | post-assembly candidate validation | baseline tuning, floor 0, five seeds; stage-coverage mismatch is Candidate C02 |
| Publication | private candidate then returned `Arc` | server binds only after successful result |

The strictness scan found `deny_unknown_fields` on each direct top-level type or its custom raw deserialization
type. `serde(default)` is present inside several domain schemas, but the current canonical names optional/neutral
defaults and the 162 focused data tests include removed/unknown/missing-field rejection. No legacy alias,
dual-read, compatibility schema, catch-all, or fallback bundle was confirmed.

## Typed Catalog And Publication Ownership

The parsed aggregate owns 17 non-map catalogs plus one `GameplayMapCatalogs` aggregate. The published database
owns the same 17 domain catalogs plus the map aggregate and one derived `ItemRegistry`.

| Representation | Role | Authority classification |
| --- | --- | --- |
| `GameplayRonSourceSet` | compile-time source contents | production source inclusion authority |
| `ParsedGameplayRon` | parsed, not yet fully validated/published values | private candidate representation |
| `GameplayMapCatalogs` | five typed map/battlefield catalogs | one validated publication aggregate |
| `GameDataBaseParts` | public fixture/programmatic construction input | fixture/external API seam, not production source authority |
| `GameDataBase` | immutable published typed database | runtime consumer authority |
| `GameDataBuilder` | minimal test/fixture construction | explicit non-production seam |

The production conversion repeats the same catalog field mapping across `ParsedGameplayRon`, the loader
destructure, `GameDataBaseParts`, and `GameDataBase::from_validated_parts()`. This is not a second content
authority, but it is a measurable change-amplification/omission seam recorded as Candidate C03.

## Default, Fallback, Fixture, And Compatibility Audit

- `GameDataBuilder::empty()`, `GameDataBuilder::live_defaults()`, `RunPolicyData::builtin()`, and cached embedded
  map catalogs are documented and tested fixture lifecycles. They use current typed schemas and validators and do
  not feed server startup.
- `GameDataBase::new()` is a panic-oriented programmatic/fixture validation wrapper. Production intentionally
  uses the fallible source-aware `ParsedGameplayRon` path and private `from_validated_parts()` instead.
- Direct live-file `include_str!` occurrences outside `GameplayRonSourceSet` are fixture/unit-test helpers for
  buffs, run policy, corroded waves, or named tests. No competing production reader was found.
- Optional/default fields are not evidence of fallback by themselves. Current canonical explicitly permits only
  neutral/optional convenience, and focused rejection tests cover gameplay-meaningful required fields.
- `LiveRonError` returns the first parse/validation failure with source/domain/object/path/cause. No contract
  requires aggregated errors, and changing fail-fast semantics is not classified as a refactor candidate.
- No ignored data-loading tests were found. `#[should_panic]` tests deliberately protect fixture/programmatic
  rejection behavior and passed.

## Canonical Alignment Findings

Two material differences were confirmed.

1. `data_loading_contract.md` says the official loader consumes 21 non-manifest sources. Runtime source fields,
   `LIVE_GAMEPLAY_RON_SOURCE_COUNT`, its unit test, and the live directory all prove 22. This is part of C01's
   duplicated source-inventory authority.
2. The canonical says generated preview validation covers every run-policy scaling stage's first floor and five
   seeds. The implementation calls `try_generate_for_node_at_floor(..., floor_index: 0)` with no manifested
   difficulty, and its errors explicitly say `baseline difficulty`. Runtime preview paths pass a stored
   `MapNodeCombatDifficulty`, which can select different multipliers and `extra_waves`. This is C02.

The main bundle table lists all 17 domain groups and separately lists all five map/battlefield catalogs, matching
the actual 22 non-manifest sources despite the stale prose count.

## Consumer Classification

| Consumer | Classification | Evidence |
| --- | --- | --- |
| `game_server` startup | confirmed workspace consumer | direct official loader call before bind |
| `AppState`/actors/`GameCore` | confirmed workspace consumer | shared `Arc<GameDataBase>` fields and constructor |
| Core domain systems | confirmed after-publication consumers | direct typed catalog fields; detailed policy belongs to later Phases |
| Core fixture helpers | confirmed test-only consumers | builder/builtin/cache calls and focused tests |
| `config/game_balance.toml` external tool | unknown | no workspace path, symbol, generic TOML loader, or Cargo include evidence |
| Unity | explicitly excluded/unverified | Master scope |
| other public Rust crate consumers | unknown | public `GameDataBaseParts`/builder API; no deployment inventory |

## Validation Results

All commands ran in the current dirty working tree.

- exact sorted embedded/live comparison: 23 vs 23, symmetric difference empty.
- `cargo test -p game_core --test ron_loading -- --nocapture`: 24 passed, 0 failed, 0 ignored,
  0 measured, 0 filtered out.
- `cargo test -p game_core --test live_skill_catalog_audit -- --nocapture`: 4 passed, 0 failed,
  0 ignored, 0 measured, 0 filtered out.
- `cargo test -p game_core --lib game::data:: -- --nocapture`: 162 passed, 0 failed, 0 ignored,
  0 measured, 700 filtered out.
- `cargo test -p game_core --lib game::combat_preview:: -- --nocapture`: 27 passed, 0 failed,
  0 ignored, 0 measured, 835 filtered out.

The filtered counts are runner-reported nonmatching tests, not skips. Expected panic output came only from passing
`#[should_panic]` tests.

