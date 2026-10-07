# Phase 01 Current Boundary Audit

## Current Publication Trace

1. `game_server::main` calls `GameDataBase::load_live_embedded()` before server bind.
2. `GameplayRonSourceSet::embedded()` includes manifest plus 23 macro-declared gameplay sources at compile time.
3. `parse_gameplay_ron_sources()` first validates non-empty inventory, exact manifest, then parses typed catalogs
   with source/domain provenance.
4. `ParsedGameplayRon::validate_domain_catalogs()` runs local and cross-catalog validation. Boss raw graph is
   compiled there so PVE references can validate against indexed definitions.
5. `GameDataBase::load_live_sources()` destructures the parsed aggregate, compiles Boss raw graph again, builds a
   private `GameDataBase` candidate through `GameDataBaseParts`/`from_validated_parts`, then runs generated preview,
   warning and resolved full setup feasibility for baseline plus every manifested difficulty.
6. Only successful `Arc<GameDataBase>` returns to server/AppState/actors/Core. No partial/live fallback publication
   path was found.

## Exact Current Source Inventory

Reproduction:

```text
comm -3 \
  <(extract every game_resources `.ron` include from live_ron.rs, normalize and sort) \
  <(rg --files game_resources/data -g '*.ron', normalize and sort)
```

Result:

- embedded/direct include paths: 24
- physical production RON files: 24
- symmetric difference: no output
- manifest: 1
- macro gameplay sources: 23
- current canonical count: 23 non-manifest sources

Boss Runtime is the only net source addition from the historical P01 inventory. It participates in the same
macro/exact-set test and is not a parallel file loader.

## Parse/Validation/Public Mapping Result

| Layer | Current authority/result | Candidate impact |
| --- | --- | --- |
| source paths/count | single macro + exact physical reconciliation | C01 full target present |
| manifest | strict v1 direct include | unchanged |
| raw schema/local validation | typed catalogs, deny-unknown/custom raw, domain validators | unchanged plus strict Boss raw compiler |
| cross references | final parsed validator plus indexed Boss/PVE references | Boss strengthens current graph |
| manifested generation | all profiles/bands/roles/stages × every encounter × 5 seeds | C02 full target present |
| full setup | same manifested preview into `CombatExecutor` feasibility | C02/P05 prerequisite complete |
| private conversion | parsed destructure→Boss recompile→parts→private destructure→database | C03 remains, with new duplicate Boss compile |
| fixture/programmatic | public parts/builder panic path | deliberately separate; public consumer uncertainty retained |

## Canonical Result

- `data_loading_contract.md` now matches 23 non-manifest sources, macro-derived inventory and exact physical test.
- It accurately states baseline plus every manifested difficulty and full setup feasibility before publication.
- It accurately separates production fallible loader from fixture builder/panic path.
- Boss source/strict compile/PVE binding and one live reference are described consistently with current data.
- No current canonical claims parsed-to-published mapping has been consolidated; C03 remains an internal backlog.

## Boss Historical Claim Reaudit

| Claim | Full original invariant check | Verdict |
| --- | --- | --- |
| P01-C01 coimplemented | all 24 source paths, derived count, physical exact set, parse/error/embedded semantics checked | confirmed, `fully_implemented` |
| P01-C02 prerequisite completed | every encounter and all live difficulty cases/seeds, warnings, same-preview full setup and atomic return checked | confirmed, `fully_implemented` |
| P01-C03 not Boss-linked | current mapping checked; Boss added field and compile duplication without consolidation | current impact confirmed, `still_valid` |

## Cross-Phase Handoffs

### P02

May rely on one immutable database after all current gates, but must independently audit run/checkpoint/persistence
authority. Boss session-private/no-save claim is only supporting evidence.

### P05

Receives current proof that publication calls resolved setup feasibility for every manifested preview and that one
Boss binding reaches typed setup. P05 must inspect full preview/setup authority and C05 test cleanup itself.

### P08/P09

Receive strict published Boss/effect/skill/delivery references. P01 does not decide runtime effect, trigger,
targeting or delivery semantics.

### P10

Receives public fixture API uncertainty and private Boss publication data. P10 owns actual DTO/server/Unity/public
consumer inventory; negative P01 search is not absence proof.

## External/Consumer Classification

| Consumer | Current classification |
| --- | --- |
| server startup/AppState/Core | `workspace_confirmed` official Arc consumer |
| Core domain systems | `workspace_confirmed`, semantic audit deferred by domain |
| builder fixture/server tests/examples | `workspace_confirmed` non-production consumers |
| direct public `GameDataBaseParts` consumer | none found in workspace; unknown external Rust consumer |
| `config/game_balance.toml` | no workspace/accessible Unity hit; `inaccessible_known_or_unknown_tooling` |
| Unity live-data reader | not part of current Core loader; external gameplay consumer boundary remains P10/unverified |

## New Candidate Audit

The duplicate Boss compile is real current evidence but not a distinct root cause. It enters CRA-P01-C03's private
validated conversion scope. No other new source/publication authority, fallback, compatibility reader, silent
recovery or partial publication was found. New candidate result: none.

## Current Boundary Verdict

- Candidate dispositions: 2 fully implemented, 1 still valid.
- Boss historical closure: P01-C01/P01-C02 confirmed over full scope.
- New candidates: 0.
- Policy decisions: none.
- Required external gate: only eventual public API removal; it does not block private C03 readiness.
- P02 handoff: ready after P01 completion audit.

