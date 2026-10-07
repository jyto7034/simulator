# Core Enhanced Refactoring Audit — Phase 00 Surface Inventory

Inventory Date: 2026-08-07
Primary Scope: current working tree의 `core_enhanced`
Inventory Status: complete; Phase completion audit pending
Production Mutation: none

## Inventory Method And Limits

- `rg --files`의 visible file set과 `rg --files --hidden`의 tracked tooling file set을 각각 세고
  source/test/doc/manifest/tooling으로 분류했다.
- `target/**`는 `engineering_workflow.md`가 재생성 가능한 build/debug artifact로 분류하므로 제외했다.
- Runtime boundary는 module export, public entry type, authoritative state container, production loader,
  direct server consumer를 읽어 정했다.
- 검색 hit, 파일 길이, 작성 도구와 문체는 candidate evidence로 사용하지 않았다.
- Unity는 승인된 비범위라 읽지 않았다. README가 외부 consumer 존재를 명시한다는 사실만 기록한다.
- 이 inventory는 후보를 확정하거나 점수화하지 않는다. 아래 `unverified signal`은 후속 Phase가 실제
  runtime/consumer evidence로 재검증해야 한다.

## File-Set Reconciliation

Phase 필수 문서 3개를 만든 직후의 inventory snapshot은 다음과 같다.

| Surface | Files | Evidence / disposition |
| --- | ---: | --- |
| Visible total | 873 | `rg --files core_enhanced` |
| Rust source under `src/**` | 176 | 10개 제안 boundary와 shared seam으로 모두 분류 |
| Rust integration target/support under `tests/**` | 18 | 5 target root + 13 support/module file |
| Docs | 676 | 647 Goal artifact + 29 canonical/governance/history/content file |
| Package manifest | 1 | `Cargo.toml`; workspace root manifest와 server path dependency도 supporting evidence로 확인 |
| Visible config | 1 | `config/game_balance.toml`; workspace reference 미확인, Data Phase의 unverified signal |
| Root tmp history | 1 | `tmp/run_persistence_facility_monolith_reaudit_2026-07-28.md`; non-authoritative history |
| Hidden tracked/tooling files | 13 | visible 873과 합친 non-target total 886 |
| Non-target total | 886 | `rg --files --hidden ... -g '!target/**'` |

Visible extension reconciliation은 Markdown 676, Rust 194, RON 1, TOML 2로 합계 873이다. Hidden 13개는
build/CI/Git/IDE/Cate tooling이며 아래에서 별도로 disposition한다.

## Rust Source Reconciliation

아래 그룹은 겹치지 않으며 합계 176개다.

| Source group | Files | Primary disposition |
| --- | ---: | --- |
| `src/lib.rs`, `src/main.rs` | 2 | Phase 10 package/command host seam |
| `src/bin/*.rs` | 5 | Phase 10 generated contract fixture seam; 실제 consumer는 unverified |
| `src/game/mod.rs` | 1 | shared module registry; Final coverage seam |
| `src/game/behavior.rs` | 1 | Phase 10 command/result/DTO authority seam |
| `src/game/world.rs`, `src/game/world/**` | 33 | Phase 2~10 shared host/mutation/projection seam; submodule별 아래 disposition 적용 |
| `src/game/data/**` | 19 | Phase 1 loader/schema/catalog authority; domain consumer Phase에도 supporting |
| `src/game/battle/**` | 63 | Phase 6~9; lifecycle/spatial/effect/targeting을 독립 분리 |
| `src/game/map/**` | 7 | Phase 2 run/map authority |
| `src/game/resources/**` | 7 | Phase 3 roster/inventory/economy, `GameState`는 Phase 2/10 shared |
| `src/game/combat_preview/**` | 5 | Phase 5 preview authority |
| `src/game/combat_setup/**` | 9 | Phase 5 battle-start plan/handoff |
| Other root domain modules | 20 | Phase 2~9 domain별 disposition |
| `src/game/events/**` | 2 | Phase 6/8 combat event seam |
| `src/game/managers/**` | 2 | Phase 2/6 identity/determinism seam |
| **Total** | **176** | reconciled |

### Root domain module disposition

| Module | Boundary disposition | Authority/seam evidence |
| --- | --- | --- |
| `ability.rs` | Phase 9 | skill definition, cast/step/delivery typed model |
| `abnormality_research.rs` | Phase 2 / Phase 3 seam | run-scoped research state, reward/delivery consumer |
| `boss_omen.rs` | Phase 2 | run/floor map progression authority |
| `combat_power.rs` | Phase 2 / Phase 5 seam | run power authority and combat-entry planning |
| `determinism.rs` | Phase 6 shared foundation | battle/run deterministic identity/RNG helper |
| `employee.rs` | Phase 3 | roster, condition, eligibility and persistent employee state |
| `employee_psychology.rs` | Phase 3 / Phase 4 seam | employee-owned state advanced by node lifecycle |
| `encounter_compendium.rs` | Phase 2 | local profile state; server persistence is supporting evidence |
| `enums.rs` | shared | cross-domain public enum definitions; owner checked by consuming Phase |
| `growth.rs` | Phase 3 | employee growth state |
| `monolith.rs` | Phase 2 | run/floor monolith state and conversion lifecycle |
| `pve_bonus_objectives.rs` | Phase 5 / Phase 8 seam | preview/result input and outcome calculation |
| `range_preview.rs` | Phase 7 / Phase 9 seam | final spatial/targeting projection |
| `recruitment.rs` | Phase 2 / Phase 3 seam | run queue/session and roster commit |
| `reward.rs`, `reward_policy.rs` | Phase 3 / Phase 5 seam | grant authority and combat reward planning |
| `skill_fragment.rs` | Phase 3 / Phase 9 seam | inventory/composition state and battle skill consumer |
| `stats.rs` | Phase 8 | persistent/runtime stat and triggered effect model |
| `visual_novel.rs` | Phase 4 | VN session/trigger runtime state |
| `wave_resolution.rs` | Phase 5 | authored PvE wave resolution into preview/setup |

### World host disposition

`GameCore` in `world.rs` is the public command entry and owns private `GameCoreState` plus shared
`Arc<GameDataBase>`. `execute_with_transport_context()` validates battle authority, dispatches every
`PlayerBehavior` by domain and returns `BehaviorResult`. This file is a shared seam, not an independent gameplay
authority to merge into one candidate.

| World surface | Boundary disposition |
| --- | --- |
| `world.rs` | Phase 10 entry/dispatch; all domain Phases verify their match arms |
| `state.rs` | Phase 2 run/checkpoint + Phase 6 active battle + Phase 7 tactical + Phase 10 projection shared seam |
| `active_run.rs`, `node_flow.rs`, `map_content.rs`, `map_encounters.rs` | Phase 2 |
| `headquarters.rs`, `helpers.rs`, `maintenance.rs`, `reward.rs`, `shop.rs` | Phase 3 |
| `event_node.rs`, `facility.rs`, `visual_novel_flow.rs`, `visual_novel_triggers.rs` | Phase 4 |
| `combat.rs` | Phase 5~9 shared battle handoff/live/result seam; each Phase audits only its authority slice |
| `combat_trace.rs` | Phase 6 debug/non-authoritative result seam |
| `action_capability.rs`, `battle_capability.rs`, `snapshot.rs` | Phase 10 derived projection/admission seam |
| `admin.rs`, `admin/**` | Phase 10 debug command/fixture seam |
| `tests/**` | behavior safety-net owner mapped to the same domain Phase |

### Battle surface disposition

`BattleCore` owns the event queue, scenario runtime, unit/item/buff/cast/projectile/area/movement/field-work
runtime maps, battlefield, live signals, `Arc<GameDataBase>`, event log and deterministic sequence counters.
`BattleExecutionState` owns simulation time and finish/finalize lifecycle. These shared containers are reviewed
by multiple boundary Phases without treating their physical file as one indivisible candidate.

| Battle surface | Files | Boundary disposition |
| --- | ---: | --- |
| `battle/battlefield/**` | 2 | Phase 7 |
| Top-level battle modules | 14 | Phase 6 base/result/event log; effect/spatial modules routed below |
| `battle/core/**` | 33 | Phase 6~9 by runtime responsibility |
| `battle/validation/**` | 14 | Phase 6 validation harness plus Phase 7~9 domain checks |

Detailed split:

- Phase 6: `core/{mod,build,commands,ids,sim,types}.rs`, top-level
  `{enums,event_log,ids,mod,recording,result_stats,scenario,types}.rs`, validation harness and world combat
  lifecycle/result paths.
- Phase 7: battlefield, `tile_range.rs`, `core/movement/**`, `core/spatial.rs`,
  `core/{field_work,tactical_relocation,tactical_stance,tactical_swap}.rs` and world live deployment/tactical
  state seam.
- Phase 8: top-level `{buffs,cooldown,damage,death,stat_pipeline}.rs`,
  `core/{damage_runtime,death_runtime,resonance_runtime,triggers}.rs` and relevant validation/event paths.
- Phase 9: `core/{basic_attack,basic_attack_projectile,projectile_math,target_usefulness,targeting}.rs`,
  `core/skill_runtime/**`, `ability.rs`, `range_preview.rs` and target/delivery validation.

## Runtime And Authority Flow

```text
game_server PlayerGameClientMessage::Command
  -> typed PlayerBehavior + optional battle authority
  -> PlayerGameActor::ExecutePlayerBehavior
  -> GameCore::execute_with_transport_context
  -> GameCoreState / RunState / ActiveBattleSession / BattleCore mutation
  -> BehaviorResult / command-result contract / snapshot DTO
  -> game_server PlayerGameServerMessage mapping
```

- `GameCoreState` owns `GameState`, run/node/session state, economy/inventory/roster, VN, compendium,
  `ActiveBattleSession`, checkpoint and run-power authority.
- `RunState` owns map/progression/facility/research/monolith/boss-omen/session/preview/recruitment state.
- `ActiveBattleSession` owns `BattleCore`, `BattleExecutionState`, revision, entry snapshot, employee attempt,
  tactical resource, live deployment, playback and tactical focus.
- `BattleCore` owns simulation-internal runtime; `ActiveBattleSession` adapts it to run/transport lifecycle.
- `behavior.rs` owns the public command/result/snapshot DTO family. Server mapping consumes these exact Core types,
  but `PlayerGameServerMessage` also flattens selected battle DTOs and therefore remains a direct consumer seam.

## Production Live Data Inventory

`GameDataBase::load_live_embedded()` is the server startup entrypoint. It parses
`GameplayRonSourceSet::embedded()`, performs domain/cross-reference/generated-preview validation and publishes an
`Arc<GameDataBase>` only after success.

| Data surface | Count / evidence | Disposition |
| --- | --- | --- |
| `game_resources/data/bundle.ron` | 1 manifest, schema version 1 | Phase 1 supporting production data |
| Non-manifest embedded gameplay sources | 22 fields and constant `LIVE_GAMEPLAY_RON_SOURCE_COUNT = 22` | Phase 1 |
| Total production files under `game_resources/data/**` | 23 | all 23 appear in Core `include_str!` references |
| Typed catalog modules | 16 domain data files plus `mod.rs`, `live_ron.rs`, `validation.rs` | Phase 1 primary, domain Phases supporting |
| Server startup | `game_server/src/main.rs::load_game_data_from_ron()` delegates to official loader | confirmed supporting consumer |

The current `data_loading_contract.md` says “manifest 외 21개” while code, a named constant, a focused test and
the actual source set say 22. Phase 0 records this as a confirmed code/canonical count drift that Phase 1 must
classify; it is not assigned a Candidate ID here.

`config/game_balance.toml` has no reference in current Core tests/source or server source found by workspace
search. Because an external tool could still consume it, Phase 1 receives it as `unverified`, not as confirmed
obsolete data.

## Test And Validation Safety-Net Inventory

| Test surface | Evidence | Boundary use |
| --- | --- | --- |
| Source/unit test attributes | 862 | distributed across all domain Phases |
| Integration test attributes | 48 | five Cargo integration roots and their modules |
| Total Rust test attributes | 910 | inventory count only; no runner executed |
| `live_item_skill_activation.rs` | 3 | Phase 8/9 effect and skill delivery |
| `live_skill_catalog_audit.rs` | 4 | Phase 1 membership and Phase 9 evidence-governance seam |
| `ron_loading.rs` | 24 | Phase 1 primary plus domain data assertions |
| `skill_runtime_contract.rs` | 13 | Phase 9 primary |
| `skill_test_suite.rs` + `skill_test/**` | 4 | Phase 9 behavior scenarios |

High-density focused surfaces already visible include battle core (97 tests in `core/mod.rs`), battle commands
(28), movement engine/planner/backend (19/25/15), world combat (74), world map flow (40), equipment (28),
node sessions (18), snapshots/start (23), data live RON (54), combat preview (27), damage (22), ability (15),
skill data (13), run policy data (17), employee (16) and admin (16). Counts identify available safety-net
locations only and do not prove contract coverage.

`tests/test_actor/mod.rs` is not imported by any current integration root found in the module-edge search. Phase
10 receives it as an `unverified` test-harness cleanup signal; no non-use verdict is made in Phase 0.

No `cargo test`, `cargo check` or probe was run because this Phase claims surface/authority coverage, not runtime
behavior correctness. The future Phase that asserts a candidate must select and execute the relevant tests.

## Documentation Inventory And Ownership

The initial 676 docs consist of 647 Goal artifacts and 29 non-Goal files.

| Class | Files | Authority disposition |
| --- | ---: | --- |
| `docs/README.md` | 1 | document index/governance |
| Durable runtime/gameplay/data/targeting canonicals | 4 | boundary-specific current contract evidence |
| Workflow/Goal/completion/probe guides | 4 | method/validation evidence, not gameplay authority |
| Game overview | 1 | product context, not detailed runtime contract |
| `docs/audit/**` | 3 | one evidence canonical/manifest plus supporting audit note |
| `docs/skills/**` | 4 | content policy/index; live implementation still code/data-first |
| `docs/decisions/**` | 9 | approved-history input, not current runtime source of truth |
| `docs/archive/**` | 2 | historical snapshot only |
| `docs/tmp/**` | 1 | non-authoritative temporary history |
| `docs/goals/**` | 647 | current Master/Phase artifacts or historical supporting evidence |

Goal shape inventory found 181 `PLAN.md`, 180 `EXPERIMENTS.md` and 180 `EXPERIMENT_NOTES.md`, plus review/audit
artifacts. The sole PLAN without the current three-file shape is historical
`stacked_code_decomposition_audit/PLAN.md`; Phase 0 does not treat it as an active dependency or repair it.

Canonical routing:

- Phase 1: `data_loading_contract.md`, audit evidence contract/manifest.
- Phase 2~5: `game_rulebook.md` plus relevant `core_runtime_contract.md` sections.
- Phase 6~8: `core_runtime_contract.md` and gameplay battle sections.
- Phase 9: `skill_target_contract.md`, relevant runtime/data/gameplay sections.
- Phase 10: `core_runtime_contract.md` projection sections and README-declared external Unity contract seam;
  Unity remains unverified/out-of-scope.
- Final: `README.md`, `engineering_workflow.md`, Goal/completion guides.

## Manifest, Build, CI And Tooling Inventory

| Surface | Disposition |
| --- | --- |
| root `Cargo.toml` + `core_enhanced/Cargo.toml` | workspace/package and direct dependency surface; Phase 10/Final |
| `game_server/Cargo.toml` path dependency | confirmed direct consumer seam |
| `.cargo/config.toml` | build config (`tokio_unstable`); Phase 10/Final validation environment |
| `.github/workflows/main.yml` | coverage workflow; Phase 10 test-safety surface |
| `.gitignore` | file-safety/generated classification surface |
| `.cate/**`, `.idea/**`, `.vscode/**`, `.codex` | IDE/agent workspace tooling; runtime candidate audit out-of-scope |
| `target/**`, timeline/debug exports | generated artifact, excluded by workflow policy |

The VS Code launch configuration still names package/binary `card_game`, while the manifest package is
`game_core`. This is a tooling-only drift outside the approved runtime/data/test/canonical candidate catalog and
is recorded as an out-of-scope follow-up signal, not a Core refactor candidate.

## Supporting Consumer Classification

| Consumer/data boundary | Classification | Evidence |
| --- | --- | --- |
| `game_resources/data/**` | confirmed supporting production data | exact 23-file loader reference match |
| `game_server` | confirmed direct workspace consumer | path dependency plus 14 source/example files importing Core types |
| Server persistence | confirmed supporting consumer for Phase 2 | active-run and compendium stores consume Core payload/profile |
| Server command/transport | confirmed supporting consumer for Phase 10 | actor messages/state/handlers map typed behaviors/results/DTOs |
| Unity | explicitly out-of-scope / unverified | README declares external canonical and user scope excludes access |
| Other external Rust consumers | unknown | public crate surface exists; deployment/usage not established |

## Inventory Signals Requiring Later Evidence

These are not candidates and have no score.

| Signal | Evidence now | Required owner Phase |
| --- | --- | --- |
| Data canonical source-count drift | code/test/source set 22 vs canonical 21 | Phase 1 classification and contract check |
| `config/game_balance.toml` apparent non-use | no workspace symbol/path reference | Phase 1 runtime/external consumer audit |
| `tests/test_actor/mod.rs` apparent non-use | no integration module edge | Phase 10 test-target/reachability audit |
| five Core fixture binaries | Cargo autobin files, no non-history workspace references found | Phase 10 consumer/artifact lifecycle audit |
| empty `src/main.rs` and unused public logger entry in workspace | library is actual server dependency; external consumer unknown | Phase 10 package surface audit |
| stale `.vscode` package name | manifest is `game_core`, launch config says `card_game` | out-of-scope tooling follow-up only |
| historical Goal shape inconsistency | one PLAN-only historical audit directory | out-of-scope document-governance follow-up only |

## Final Surface Disposition

All 176 source files, 18 test files, 676 visible docs, manifest/config/tmp entries and 13 hidden non-target files
have one of these explicit dispositions:

- Phase 1~10 primary or shared supporting audit surface;
- Final cross-boundary/catalog consolidation surface;
- confirmed supporting data/server consumer;
- explicitly out-of-scope Unity/tooling/generated artifact;
- unverified signal assigned to a named Phase.

No production refactor candidate is confirmed by this inventory alone.
