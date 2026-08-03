# Documentation–Code Alignment Reaudit Notes

Status: complete

Document class: non-canonical working notes

Started: 2026-07-18

This file is the resumable working notebook for the audit. It is not gameplay, runtime, data, or
wire authority. Current behavior must be established from runtime code, live data, producers,
consumers, and executable evidence; intended policy must be interpreted through the current durable
canonical documents and explicit user decisions.

Do not move conclusions from this notebook into canonical documents during the audit. Material
findings are summarized in `AUDIT_REPORT.md` at completion, and approved corrections are handled by
separate goals.

## Working Rules

- Give each claim and finding a stable ID; do not renumber previously recorded items.
- Separate observed facts from inference and unresolved questions.
- Cite exact file paths, symbols, test names, and commands rather than relying on recollection.
- A green test proves only its assertions, not the surrounding feature or catalog.
- Mark the last fully traced point and next check whenever a path is incomplete.
- Do not attribute pre-existing dirty-worktree changes to this audit.
- Do not edit production code, live data, tests, durable canonical documents, or external Unity
  artifacts under this goal.

## Resume Checkpoint

Last updated: 2026-07-18

Current pass: complete

Last completed claim: all 48 claims classified; final report published

Next action:

- None. Corrections require separately approved follow-up goals.

Known environmental limitations:

- None recorded yet.

## Coverage Ledger

Add one row for each substantive canonical section or independently testable claim. Split rows when
one section contains multiple producers, state transitions, or externally visible outcomes.

| Claim ID | Domain | Document claim | Producer/state owner | Data | Server/Unity consumer | Test/probe | Verdict | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| DCA2-DOC-CLAIM-001 | governance | README source-of-truth order, document ownership, archive/goal interpretation | runtime/data/document tree | live bundle | external canonical locations | path/ownership audit | Document drift | evidence_complete |
| DCA2-GAME-CLAIM-001 | modes | game identity, `DefenseRoute`, Normal/Endless mode and termination rules | world/map progression | run policy | root snapshot/commands | world/map tests | Aligned | evidence_complete |
| DCA2-GAME-CLAIM-002 | run start | starter selection, unified employee definition pool, recruitment queue identity/ownership | world/recruitment/employee | employee definitions | starter/roster/recruitment DTO | start/recruitment tests | Aligned | evidence_complete |
| DCA2-GAME-CLAIM-003 | map | Floor generation, selectable nodes, Gate progression, safe-zone semantics | map/world node flow | generation/node policy | node-map/capability projection | map-flow tests | Aligned | evidence_complete |
| DCA2-GAME-CLAIM-004 | event/inventory | Event choice, ordinary inventory, capacity, quantity, consumable timing | world event/resources | event/consumable data | event/inventory DTO and commands | node/inventory tests | Aligned | evidence_complete |
| DCA2-GAME-CLAIM-005 | shop/briefing | shop session/quote/transaction and pre-node briefing | world shop/combat preview | shop/PvE data | shop/preview DTO | shop/preview tests | Aligned | evidence_complete |
| DCA2-GAME-CLAIM-006 | battle entry | official combat mode, setup boundary, battlefield/deployment and wave rules | world combat/combat setup | battlefield/PvE/waves | setup snapshot | combat/setup tests | Aligned | evidence_complete |
| DCA2-GAME-CLAIM-007 | combat seam | player-visible targeting and damage feedback rules | battle runtime/range preview | equipment/skill data | update/event/HUD | battle/skill tests | Aligned | evidence_complete |
| DCA2-GAME-CLAIM-008 | growth/research | rewards, employee growth, research delivery and permanent compendium | reward/growth/research/compendium | reward/PvE data | result/research/compendium DTO | reward/research tests | Aligned | evidence_complete |
| DCA2-GAME-CLAIM-009 | equipment/fragments | equipment slots/loadout and skill-fragment acquisition/upgrade/activation policy | resources/skill fragment | equipment/fragment/skill data | inventory/loadout/maintenance DTO | equipment/fragment tests | Policy decision needed | blocked |
| DCA2-GAME-CLAIM-010 | non-combat | Support, Maintenance and HeadquartersContact node behavior | world support/maintenance/headquarters | node/shop/reward data | selected-event/options/results | node tests/admin probe | Aligned | evidence_complete |
| DCA2-GAME-CLAIM-011 | resolution | node completion, retreat restoration, attempts, employee incapacitation and run failure | world node flow/combat/state | run policy | result/root capability | combat/map-flow tests | Aligned | evidence_complete |
| DCA2-GAME-CLAIM-012 | psychology | personality, trust, trauma and psychology lifecycle | employee psychology/world | employee definitions | roster/employee DTO | psychology tests | Aligned | evidence_complete |
| DCA2-GAME-CLAIM-013 | boss/legacy | BossOmen chain/forced boss/exit Gate and removed legacy policy | boss omen/map/world | boss omen data | map/result/recovery DTO | boss/map tests | Aligned | evidence_complete |
| DCA2-GAME-CLAIM-014 | maintenance content | generic Maintenance verbs and current executable authored coverage | maintenance/resources | equipment/fragment catalogs | options/result DTO | equipment/support tests | Policy decision needed | blocked |
| DCA2-BATTLE-CLAIM-001 | targeting basis | weapon profiles, authored basic range, enemy ranged-route policy, skill reach separation | tile range/targeting/build | equipment/enemy/skill data | range preview | targeting/range tests | Policy decision needed | blocked |
| DCA2-BATTLE-CLAIM-002 | auto target | automatic hostile target usefulness and immunity/effect rules | target usefulness/targeting | skill/buff data | events/preview seam | focused battle tests | Aligned | evidence_complete |
| DCA2-BATTLE-CLAIM-003 | skill target | `SkillTarget`, cast selection, step targeting and random identity | skill runtime/targeting/determinism | skill data | command/events | skill runtime tests | Aligned | evidence_complete |
| DCA2-BATTLE-CLAIM-004 | cast/projectile | interrupt, epoch, projectile target lifecycle and collision | cast/projectile runtime | skill data | battle events | skill/projectile tests | Aligned | evidence_complete |
| DCA2-BATTLE-CLAIM-005 | tile area | `TileArea` geometry, valid-tile filtering, primary target and repeat delivery | area/spatial/tile range | skill/battlefield data | range/event projection | area tests | Aligned | evidence_complete |
| DCA2-BATTLE-CLAIM-006 | targeting projection | final range cells and event/Unity ownership; removed continuous area legacy | range preview/event log | live skill data | setup/update/Unity overlay | DTO/Unity tests | Implementation drift | evidence_complete |
| DCA2-RUNTIME-CLAIM-001 | determinism | gameplay RNG identity is independent from event-log sequence | determinism/battle RNG callers | generated/live inputs | observable events only | deterministic tests | Aligned | evidence_complete |
| DCA2-RUNTIME-CLAIM-002 | event/checkpoint | append-only event delta, checkpoint reconcile authority and sequence semantics | battle event log/snapshot | — | server/Unity battle store | event/transport tests | Aligned | evidence_complete |
| DCA2-RUNTIME-CLAIM-003 | movement/route | setup route cells, movement events, active checkpoint units and battlefield collision projection | combat setup/movement/snapshot | battlefield data | field/actor consumers | movement/setup tests | Aligned | evidence_complete |
| DCA2-RUNTIME-CLAIM-004 | result/diagnostics | no gameplay battle history, debug trace boundary, result stats and inactive-unit resolution | world combat/result stats/trace | — | combat result DTO | result tests | Aligned | evidence_complete |
| DCA2-RUNTIME-CLAIM-005 | atomic commit/recovery | staged result commit, retreat, run-failure precedence, checkpoint/setup recovery | world combat/state/node flow | run policy | root capabilities/results | world combat tests | Aligned | evidence_complete |
| DCA2-RUNTIME-CLAIM-006 | research/compendium | safe-zone delivery and global encounter compendium authority | research/compendium/world | abnormality/PvE data | snapshot/notification | focused tests | Aligned | evidence_complete |
| DCA2-RUNTIME-CLAIM-007 | inventory/shop | inventory identity/quantity/capacity and shop quote/session transaction boundary | resources/world shop | item/shop data | inventory/shop DTO | focused tests | Aligned | evidence_complete |
| DCA2-RUNTIME-CLAIM-008 | reward/growth | reward candidates/grants and employee growth atomicity | reward/growth/world | reward data | reward/result DTO | focused tests | Aligned | evidence_complete |
| DCA2-RUNTIME-CLAIM-009 | visual novel | top-level/pre-run/local presentation and trigger runtime ownership | visual novel/world trigger flow | VN/trigger data | snapshot/Unity local catalog | VN tests | Aligned | evidence_complete |
| DCA2-RUNTIME-CLAIM-010 | projections | actor identity/HUD, range preview and combat preview ownership | snapshot/range/combat preview | multiple | server/Unity consumer | shape/consumer tests | Aligned | evidence_complete |
| DCA2-DATA-CLAIM-001 | loader | embedded `load_live_embedded`, exact bundle version, source count, all-or-nothing publication and error boundary | data mod/live_ron/validation | bundle + 21 sources | server startup | ron_loading/check | Document drift | evidence_complete |
| DCA2-DATA-CLAIM-002 | authoring | strict unknown-field rejection, explicit gameplay fields and no legacy aliases/fallback | serde data modules | all live RON | — | negative parser tests | Implementation drift | evidence_complete |
| DCA2-DATA-CLAIM-003 | catalogs | canonical live-file index, domain loader ownership and cross-catalog references | data modules/validation | bundle catalogs | admin/catalog projections | loading/catalog tests | Aligned | evidence_complete |
| DCA2-DATA-CLAIM-004 | maps | map/battlefield catalog authoring and deterministic generated route/preview validation | map/combat preview validation | map/PvE data | setup/briefing | loading/preview tests | Aligned | evidence_complete |
| DCA2-DATA-CLAIM-005 | fixtures/gates | fixture builders are non-production; parse/local/cross/generated validation precedes publication | builder/loader/validation | embedded live data | server startup | ron_loading | Aligned | evidence_complete |
| DCA2-EVID-CLAIM-001 | skill evidence | manifest proves exact membership/delivery/reference ownership but not behavior | audit test | manifest/live catalogs | content index | live_skill_catalog_audit | Aligned | evidence_complete |
| DCA2-CONTENT-CLAIM-001 | roster | confirmed abnormality/tool roster and design-only versus implemented distinction | runtime catalogs | abnormality/equipment/fragments | admin/content views | loading/catalog audit | Aligned | evidence_complete |
| DCA2-CONTENT-CLAIM-002 | fragments | boss-linked fragment lifecycle and WhiteNight risk/effect policy | boss omen/skill fragment/runtime | boss/fragment/skill data | result/loadout/events | focused skill tests | Policy decision needed | blocked |
| DCA2-CONTENT-CLAIM-003 | live index | current E.G.O/fragment counts, IDs, rarity and placeholder effect summary | data loader/catalogs | equipment/fragment RON | admin/inventory views | catalog audit | Aligned | evidence_complete |
| DCA2-WIRE-CLAIM-001 | battle transport | setup/update/result order, revision, command acceptance, resync and reconnect recovery | Core live DTO | — | game_server/Unity battle store | server + Unity suites | Aligned | evidence_complete |
| DCA2-WIRE-CLAIM-002 | battle presentation | required event stream drives telegraphs, attack/cast/projectile/buff and inactive-unit presentation | Core event log | skill/battle data | Unity battlefield presentation | consumer search + Unity suites | Implementation drift | evidence_complete |
| DCA2-WIRE-CLAIM-003 | typed projections | unit identity, HUD, range preview, routes and skill catalog are consumed without local gameplay recalculation | Core setup/checkpoint/catalog | multiple | game_server/Unity field/UI models | consumer tests | Aligned | evidence_complete |
| DCA2-WIRE-CLAIM-004 | external enum inventory | external canonical lists current Core node categories and targeting profiles exactly | Core enums | map/equipment data | Unity parser/labels | exact enum/search audit | Document drift | evidence_complete |
| DCA2-WIRE-CLAIM-005 | setup catalog refs | setup abnormality preload refs contain only non-empty actual abnormality catalog IDs | Core setup projection | preview/wave identity | game_server/Unity setup | live smoke + source/test trace | Implementation drift | evidence_complete |
| DCA2-DOC-CLAIM-002 | workflow | workflow paths, package/test commands, goal lifecycle and debug classification match repository reality | workspace/document tree | — | external probe tree | path/command checks | Aligned | evidence_complete |
| DCA2-DOC-CLAIM-003 | probes | registered probe paths, environment gates and adjacent contracts exist | server env/probe scripts | — | probes | path + live smoke | Aligned | evidence_complete |
| DCA2-DOC-CLAIM-004 | decisions | decision records are correctly marked historical/dispositioned and point to current owners | current canonical/runtime | relevant live data | external contracts where linked | link/status audit | Aligned | evidence_complete |

Status values: `pending`, `tracing`, `evidence_complete`, `blocked`, `reported`.

## Finding Registry

| Finding ID | Claim ID | Severity | Verdict | Summary | Owner boundary | Reported |
| --- | --- | --- | --- | --- | --- | --- |
| DCA2-DATA-001 | DCA2-DATA-CLAIM-001 | P3 | Document drift | Data contract says 23 non-manifest gameplay RON sources; loader and unit test define 21 | durable data doc vs loader | yes |
| DCA2-GAME-001 | DCA2-GAME-CLAIM-009 | P2 | Policy decision needed | Rulebook promises weapon/armor block adjustment but equipment schema/stat pipeline has no block modifier | gameplay policy vs equipment/runtime | yes |
| DCA2-GAME-002 | DCA2-GAME-CLAIM-009 | P2 | Policy decision needed | Rulebook says basic-attack fragment changes are active-skill results; live model has 29 direct `BasicAttackModifier` fragments | gameplay policy vs content/runtime | yes |
| DCA2-TEST-001 | DCA2-GAME-CLAIM-008 | P2 | Aligned + Test gap | Endless all-response-complete RunComplete branch has no direct focused regression assertion | runtime vs executable evidence | yes |
| DCA2-BATTLE-001 | DCA2-BATTLE-CLAIM-001 | P2 | Policy decision needed | Contract requires Boss/Elite/special authored attack range, but all 30 live abnormalities use generic missing-pattern fallback | targeting policy vs abnormality data/runtime | yes |
| DCA2-BATTLE-002 | DCA2-BATTLE-CLAIM-001 | P3 | Policy decision needed | Runtime schema still exposes `GrenadeLauncher`, while targeting/content docs exclude it as current and call grenade future-only | targeting docs vs equipment schema | yes |
| DCA2-DOC-001 | DCA2-DOC-CLAIM-001 | P3 | Document drift | Runtime canonical uses historical goal labels `G15` and `G24` as current domain vocabulary | durable runtime doc vs document governance | yes |
| DCA2-DATA-002 | DCA2-DATA-CLAIM-002 | P2 | Implementation drift | Production nested gameplay RON types accept unknown fields despite the maximally strict canonical contract and completed G12 disposition | data schema vs canonical/accepted PD-09 | yes |
| DCA2-CONTENT-001 | DCA2-CONTENT-CLAIM-002 | P2 | Policy decision needed | WhiteNight content policy describes a 12-mark/apostle chain and boss-clear fragment, while live content is a one-step placeholder whose fragment is response-research gated and only modifies basic attack | content policy vs live/runtime | yes |
| DCA2-WIRE-001 | DCA2-WIRE-CLAIM-002 | P1 | Implementation drift | Unity stores the required battle event stream but does not render `SkillAreaDeclared` tiles, inactive-unit lifecycle positions, or attack/cast/projectile/buff presentation events | external battle contract vs Unity consumer | yes |
| DCA2-WIRE-002 | DCA2-WIRE-CLAIM-004 | P3 | Document drift | External Unity canonical omits three current `MapNodeCategory` variants and still lists removed `SplashClusterFirst` | external canonical vs Core enums/Unity labels | yes |
| DCA2-GAME-003 | DCA2-GAME-CLAIM-014 | P2 | Policy decision needed | Rulebook presents equipment dismantle/enhance as general Maintenance verbs, but live recipes cover only `standard_armor` | gameplay/content policy vs live equipment recipes | yes |
| DCA2-WIRE-003 | DCA2-WIRE-CLAIM-005 | P2 | Implementation drift | Corroded-only live setup emits `catalog_refs.abnormality_ids: [""]` although the external contract permits only actual abnormality catalog IDs | Core setup projection vs external battle contract | yes |
| DCA2-DOC-002 | DCA2-DOC-CLAIM-001 | P3 | Document drift | External Unity project retains a divergent `docs - 복사본` tree with three stale canonical-looking contract copies | document governance/discoverability | yes |

Finding ID prefixes:

- `DCA2-GAME-*`: gameplay/run/node/reward/employee
- `DCA2-BATTLE-*`: battle runtime, targeting, event/checkpoint/result
- `DCA2-DATA-*`: loader, schema, validation, live content
- `DCA2-WIRE-*`: Core/server/Unity transport and consumption
- `DCA2-TEST-*`: missing or misleading executable evidence
- `DCA2-DOC-*`: governance, ownership, terminology, discoverability

## Claim Evidence Template

```md
### <Claim ID> — <Short title>

Document claim:
- `<document path and heading>`: ...

Expected behavior:
- ...

Runtime evidence:
- `<path>::<symbol>`: ...

Data evidence:
- `<path and id/field>`: ...

Server/Unity evidence:
- `<producer, mapping, contract, consumer>`: ...

Test/probe evidence:
- `<test name or command>`: proves ...

Legacy/fallback search:
- Search: ...
- Result: ...

Observed facts:
- ...

Inference:
- ...

Verdict:
- Aligned / Document drift / Implementation drift / Cross-layer drift /
  Policy decision needed / Insufficient evidence / Not applicable

Test gap:
- yes / no

Severity:
- P0 / P1 / P2 / P3 / none

Remaining risk:
- ...

Next check:
- ...
```

### DCA2-DATA-CLAIM-001 — Embedded live loader and publication boundary

Document claim:

- `docs/data_loading_contract.md#source-of-truth` names
  `GameDataBase::load_live_embedded()` as the production entrypoint, exact schema version 1,
  embedded `include_str!` loading, manifest plus 23 gameplay sources, staged validation, private
  candidate publication, contextual `LiveRonError`, and startup failure before bind.

Expected behavior:

- Server startup parses the exact versioned embedded source set, validates it completely, and only
  exposes a `GameDataBase` after all validation succeeds.

Runtime evidence:

- `src/game/data/live_ron.rs::GameplayRonSourceSet::embedded` embeds one manifest and 21 gameplay
  source fields; `LIVE_GAMEPLAY_RON_SOURCE_COUNT` is 21.
- `src/game/data/live_ron.rs::validate_live_ron_manifest` requires exactly schema version 1 and its
  manifest DTO denies unknown fields.
- `src/game/data/live_ron.rs::parse_gameplay_ron_sources` parses the complete typed source set.
- `src/game/data/live_ron.rs::ParsedGameplayRon::validate_domain_catalogs` performs domain,
  uniqueness, cross-reference, run-start, extra-wave, and map validation.
- `src/game/data/mod.rs::GameDataBase::load_live_embedded` builds a private local `Arc`, runs
  generated combat-preview validation, and returns it only after success.
- `game_server/src/main.rs::load_game_data_from_ron` maps loader failure to `InvalidData` before
  `HttpServer::bind`.

Data evidence:

- `game_resources/data/bundle.ron` contains `schema_version: 1`.
- The current `game_resources/data` inventory matches the loader's 21 non-manifest RON paths.

Server/Unity evidence:

- Server startup uses the official loader; this claim has no Unity consumer behavior.

Test/probe evidence:

- `cargo check -p game_core`: pass.
- `ron_loading`: 20 passed.
- `live_ron::tests::embedded_source_set_parses_all_versioned_gameplay_ron` explicitly asserts 21;
  manifest missing/unknown/version and nested source error tests cover strict entry behavior.

Legacy/fallback search:

- Production/server references use `load_live_embedded`; `GameDataBuilder::empty/live_defaults`
  occurrences are tests, fixtures, or examples. The server main path has no fallback bundle.

Observed facts:

- Loader, server startup, live data, and tests agree on the behavior but disagree with the durable
  document's stated source count.

Inference:

- The number 23 is stale documentation; the remaining inspected loader/publication statements are
  aligned.

Verdict:

- Document drift (`DCA2-DATA-001`).

Test gap:

- no for the identified source-count and entry-boundary behavior.

Severity:

- P3; the stale count is misleading but does not alter loader behavior.

Remaining risk:

- Strictness of every nested gameplay type remains under DCA2-DATA-CLAIM-002 rather than being
  inferred from this entrypoint audit.

Next check:

- Audit per-domain authoring strictness and catalog ownership.

### DCA2-DATA-CLAIM-005 — Fixture and validation gate separation

Document claim:

- `GameDataBuilder` is a test fixture surface; production loading performs parse, domain/cross
  validation and generated-preview validation before publication.

Runtime evidence:

- `src/game/data/mod.rs::GameDataBuilder` supplies `empty` and `live_defaults`; production
  `game_server/src/main.rs` uses `load_live_embedded` directly.
- `GameDataBase::load_live_embedded` does not pass parsed production data through
  `GameDataBuilder::build`/`GameDataBase::new`; it uses `from_validated_parts` after parsed-domain
  validation and gates return on generated-preview validation.

Test/probe evidence:

- Source search separates builder usages into tests/fixtures/examples and the live loader into
  server startup/live fixture generators.
- `ron_loading` and Core check pass.

Verdict:

- Aligned.

Test gap:

- no material gap found for entrypoint separation; startup error mapping is also covered by server
  module tests inspected later under the wire pass.

### DCA2-EVID-CLAIM-001 — Live skill manifest evidence scope

Document claim:

- The manifest proves exact delivery classification and reference membership/ownership, checks the
  abnormality-derived fragment discoverability list, and explicitly does not prove skill behavior.

Runtime/data/test evidence:

- `tests/live_skill_catalog_audit.rs` strictly parses manifest version 2, rejects duplicate list
  entries through set-length checks, derives spatial versus all-Instant membership from actual
  steps, derives abnormality/equipment/active-fragment references, computes the unreferenced
  complement, and compares abnormality-origin fragment IDs with backtick IDs in
  `skills/live_content_index.md`.
- `docs/audit/live_skill_catalog_manifest.ron` records `unknown_distortion_strike` in the explicit
  unreferenced set.
- The test contains no assertions for damage, target choice, repeat, geometry, timing, or
  presentation, matching the contract's stated limitations.
- `cargo test -p game_core --test live_skill_catalog_audit -- --nocapture`: 4 passed.

Verdict:

- Aligned.

Test gap:

- no for manifest semantics; behavior coverage is intentionally a separate claim family.

### DCA2-GAME-CLAIM-001 — Game modes and run termination

Document claim:

- `game_rulebook.md#게임-모드` defines one official `DefenseRoute` combat axis, Standard/normal as a
  fixed three-Floor run ending in a final Boss, Endless as unbounded Gate progression with BossOmen
  content, and `GameMode` as a required command/runtime source of truth.

Runtime/data evidence:

- `map/progression.rs::GameMode` has only `Standard` and `Endless`;
  `RunProgressionModeState` stores bounded `u8` Standard and unbounded `u32` Endless indices.
- `behavior.rs::PlayerBehavior::StartNewGame` requires `game_mode`; `world.rs` carries it through
  pre-run VN and constructs `RunProgression` after starter selection.
- `game_resources/data/run/policy.ron` sets `standard_floor_count: 3`.
- `RunProgression::terminal_node_category`, `standard_gate_pre_terminal_kind_id`, and
  `world/node_flow.rs` produce Elite-parented Gate terminals before the last Standard Floor, Boss on
  the last Floor, unbounded Endless Gate advances, and `RunComplete` only when Standard cannot
  advance.
- `world/combat.rs::is_final_boss_node` and combat-result completion give final Standard Boss defeat
  immediate `BossDefeated` run failure; ordinary non-final defeat follows attempt resolution.

Test evidence:

- `snapshots_and_start`: 22 passed, including required mode propagation.
- `map_flow`: 32 passed, including Standard Gate/Boss topology, Standard completion, and Endless
  uncapped Floor advancement.
- Existing focused combat tests explicitly cover final-Boss immediate failure, non-final retry, and
  non-consuming retreat; their full module is validated under DCA2-GAME-CLAIM-011.

Verdict:

- Aligned.

### DCA2-GAME-CLAIM-002 — Starter definitions and recruitment ownership

Document claim:

- One strict authored employee-definition catalog supplies starter and later recruitment candidates;
  authored definition identity and run-local employee UUID remain separate; every unselected
  definition belongs to exactly one queue/session/roster location with deterministic order and
  checkpoint preservation.

Runtime/data evidence:

- `data/employee_data.rs::EmployeeDefinitionDatabase` is a single strict catalog and derives starter
  eligibility without separate production catalogs.
- `world.rs::prepare_starter_employee_candidates` uses only `starter_eligible` definitions;
  `handle_select_starter_employees` creates run-local UUID employees and initializes recruitment
  from all definitions excluding the selected IDs.
- `recruitment.rs::RecruitmentState` sorts then seeded-shuffles the remaining complete definition
  set, fixes one active node session, returns unselected candidates to queue tail, and validates
  exact queue/session/roster ownership.
- Headquarters actions and checkpoint load repeat `validate_exact_ownership` before commit/restore.
- Snapshot tests show `employee_definition_id` and run-local `uuid` as separate fields, required
  profile/personality/combat-aptitude data, no specialties, and no legacy candidate identity.

Test evidence:

- `snapshots_and_start`: 22 passed.
- `node_sessions`: 16 passed, including fixed offers, recruit/tail behavior, shop/emergency return,
  checkpoint queue restore, and invalid ownership rejection without partial restore.
- Live `ron_loading` covers unified employee-definition parsing and starter/recruitment filtering.

Verdict:

- Aligned.

### DCA2-GAME-CLAIM-003 — Map navigation and Floor/Gate progression

Document claim:

- Core owns graph reachability/selectability, completed-node transit and edge direction; entry is
  confirmed separately; Gate is an independent terminal node, not a SavePoint, and preserves run
  assets while applying authored trauma recovery.

Runtime/data evidence:

- Generated maps contain a completed/revealed Start current position and typed node categories,
  state, visibility, slot, payload, and typed edge direction.
- `MapProgression::selectable_node_ids` traverses completed nodes through
  `RunMap::bidirectional_neighbors`, which respects outgoing ForwardOnly and both directions for
  Bidirectional edges; concealed nodes are excluded.
- World selection produces `NodeConfirm` for combat/Boss and `ConfirmEnterNode` revalidates
  capability before actual entry.
- `RunProgression` and map generation create separate Gate nodes with Elite parents on non-final
  Standard Floors and a final Boss on Floor 3. `world/support.rs` applies the authored 10% living
  trauma recovery on confirmed Gate transition without making a checkpoint.
- `RunState::for_next_floor_from` carries roster condition, recruitment, inventory/resources, skill
  fragments, research/history, and BossOmen state into the next Floor.

Test evidence:

- `map_flow`: 32 passed, including Core selectability, Gate preview/confirm separation, trauma
  recovery, Standard and Endless advancement, and Event presence/flow.
- `node_sessions`: 16 passed for interactive non-combat session ownership adjacent to map flow.

Verdict:

- Aligned.

### DCA2-GAME-CLAIM-004 — Event, ordinary inventory, and consumables

Document claim:

- Event owns typed ordered scenes and one committed pure d20 check; ordinary inventory has no
  admission cap and uses equipment instances plus definition consumable stacks; consumables are
  usable only in map/confirm planning contexts and their battle-duration modifiers decrement only on
  committed combat-node resolution, not retreat or non-combat completion.

Runtime/data evidence:

- `world/event_node.rs` validates typed session/scene/choice/employee identities, generates exactly
  `1..=20`, classifies natural 1/20 separately, plans grants/combat before committing one stored
  check, and reuses committed Event combat state after re-entry.
- `resources/inventory.rs` counts each equipment instance and definition stack as one slot, grows
  display reserve from 50 in increments of 10 without any capacity rejection, and never shrinks the
  reserve on removal.
- `world/maintenance.rs::handle_use_consumable_item` clones inventory/roster, consumes exactly one
  definition-stack quantity, validates the static definition UUID selector, and commits both states
  together.
- `world/action_capability.rs` admits consumable use only through the map/NodeConfirm planning
  context. `world/combat.rs` decrements modifiers in staged committed result paths; retreat restores
  the captured state.

Test evidence:

- `map_flow`: Event DTO, natural result persistence, atomic grant, stale identity rejection, Event
  combat/retreat, and terminal BossOmen Event tests pass.
- `equipment`: 26 passed, including quantity, selector, replacement/no-refund, treatment, and target
  eligibility behavior.
- `support`: 12 passed, including reserve high-water checkpoint restore.
- Focused `consumable_modifier_survives_abandonment_and_expires_on_committed_result`: pass.

Verdict:

- Aligned.

### DCA2-GAME-CLAIM-005 — Shop transaction and combat briefing

Runtime/data evidence:

- `world/shop.rs` owns deterministic session pages, stable offer UUIDs, checked ceil buy/floor sell
  quotes, typed quantities, no automatic refill, purchased non-repeatable exclusion, full-page
  refresh prevalidation, and cloned-state purchase/sale commit.
- Cross-shop minimum-buy comparison and checked arithmetic reject arbitrage/overflow; live shop data
  passes loader validation.
- `combat_preview` emits node/mission, template/valid/deployment tiles, routes, risk/reward,
  `enemy_briefing`, warnings/confidence and explicit `spawn_waves.enemy_entries` as separate fields.
  `combat_setup/enemy_spawns.rs` consumes `enemy_entries`, not `EnemyBriefing`.
- Generated-preview validation runs before live bundle publication.

Test evidence:

- `world::shop::tests`: 6 passed for deterministic identity/quotes, purchase, partial sale, refresh,
  non-repeatable exclusion, and rounding.
- `node_sessions`: shop session persistence and node exit tests pass.
- `map_flow::combat_node_preview_exposes_basic_briefing` and `ron_loading` generated warning tests
  pass.

Verdict:

- Aligned.

### DCA2-GAME-CLAIM-010 — Support, Maintenance, and HeadquartersContact

Runtime/data evidence:

- Support has only typed SavePoint/Rest effects; completion stages trauma effects, SavePoint
  checkpoint creation, and node completion together. Current policy values are 15% and 10 trauma.
- Maintenance is a distinct active node content/session; enhance/dismantle/fragment upgrade/awaken
  capabilities are absent in ordinary Safezone and repeatable before explicit completion.
- HeadquartersContact creates one fixed recruitment session and exposes authored recruitment,
  emergency Enkephalin, and headquarters Shop options. Each action resolves the node; pending
  research delivery occurs on the shared planning return, not on entry.

Test evidence:

- `support`: 12 passed, including SavePoint/load limit, Rest, choice modes, and Maintenance
  separation.
- `equipment`: 26 passed, including Maintenance-only equipment/fragment operations and atomic
  auto-unequip behavior.
- `node_sessions`: 16 passed, including all Headquarters paths and session persistence.

Verdict:

- Aligned.

### DCA2-GAME-CLAIM-006 — Official battle entry, deployment, and waves

Runtime/data evidence:

- Live combat entry rejects any non-Defense mission pair and constructs one live DefenseRoute battle
  model. Removed `DefendPoint`, `Encirclement`, `DefendAndEscape`, and formation policy names do not
  exist as combat variants; `Recovery` matches only a command scope, not a battle mode.
- `survive_timer_ms` is a Defense encounter option and is projected in setup; required wave groups,
  defense-object loss, deployment cost, 30s withdrawal/90s defeat redeploy locks, new instance IDs,
  and current-HP policies are Core-owned.
- SplitRoom remains a typed battlefield archetype/template but live generation weight is disabled
  and fallback selection rejects it, matching the rulebook's “template classification, not current
  live assignment” boundary.
- Spawn waves contain explicit enemy entries and route IDs fixed in preview/setup; current live
  authored timing and optional non-required waves match the RON loading assertions.

Test evidence:

- `game::world::tests::combat`: 59 passed, including live RON playable path, setup timer, authored
  route, deploy/withdraw/redeploy, pause/speed, required-wave result flow, and setup recovery.
- `ron_loading`: current wave source/timing/required flags and preview contracts pass.

Verdict:

- Aligned.

### DCA2-GAME-CLAIM-008 — Reward, growth, research, and compendium

Runtime/data evidence:

- Live encounter validation requires `ClaimAll`; result completion rejects `ChooseOne`. Reward node
  sessions use one seeded candidate and exclude XP according to their policy path.
- `apply_post_battle_resolution_to_state` grants authored survival XP once per surviving employee row
  after multi-instance result aggregation; `GrantExecutor::grant_experience` sorts/deduplicates
  eligible employees and distributes total-pool remainder deterministically.
- Planning-return research delivery is staged only for NodeCompleted/FloorAdvanced paths; retreat,
  cancel, checkpoint load, RunComplete and RunFailed do not synthesize delivery.
- Encounter compendium derives only actual opponent Abnormality/CorrodedEmployee spawn events,
  validates IDs, unions into game-global profile state only on consumed results, and resolves display
  metadata from current catalogs.
- Endless response research is run-local, victory-only, capped by policy, grants the unique fragment
  once then threat-class dust, and changes to `RunComplete` when every initialized abnormality entry
  is complete.

Test evidence:

- Combat suite assertions cover atomic reward failure, survival XP, pending delivery boundaries,
  compendium commit/rollback/global persistence, research completion, duplicate-fragment dust, and
  bonus objectives.
- Abnormality research tests: 2 passed; reward tests: 2 passed.

Verdict:

- Aligned, with `DCA2-TEST-001`: no focused test directly drives the final remaining Endless
  response entry to completion and asserts the `RunComplete` result/absence of planning delivery.

### DCA2-GAME-CLAIM-009 — Equipment and skill-fragment gameplay meaning

Aligned portions:

- Weapon/Armor/Accessory slots, owned-instance enhancement, equipment versus fragment dust,
  Maintenance-only upgrade/awaken/dismantle, active-fragment slot, weapon-profile compatibility and
  Core validation are implemented and covered by the 26-test equipment module.
- Equipment Permanent effects and enhancement modifiers are snapshotted into the battle draft and
  applied in the documented final-stat pipeline order.

Mismatch 1 (`DCA2-GAME-001`):

- `game_rulebook.md#장비` says weapons provide block adjustment and armor provides survivability and
  blocking. `EquipmentMetadata` has triggered stat effects, activations and optional weapon profile;
  `StatId` has no BlockCapacity modifier and the final-stat pipeline cannot change block capacity
  from equipment.
- The previous cross-layer master explicitly leaves this as user-deferred G22/PD-14 and forbids
  inventing block values/schema before Equipment/Maintenance content policy is resumed.

Mismatch 2 (`DCA2-GAME-002`):

- `game_rulebook.md#스킬-파편` says basic-attack enhancement/stat/buff/resonance changes are expressed
  as active-skill results. Current strict schema intentionally has
  `SkillFragmentEffectDef::BasicAttackModifier`; `SkillFragmentLoadout::apply_to_profile` applies it
  directly before battle without an active skill.
- Live data has 29 BasicAttackModifier fragments (28 abnormality-derived plus the starter) and two
  abnormality-derived ActiveSkill fragments. The specialized content catalog explicitly lists both
  modes, so current durable documents also disagree with each other.

Verdict:

- Policy decision needed. The audit cannot choose between narrowing the rulebook to the current
  model and migrating live content/runtime to the stronger active-only/block-adjusting policy.

### DCA2-GAME-CLAIM-011 — Result commit, retreat, condition, and failure

Runtime/test evidence:

- Combat result completion plans cloned roster/inventory/fragments/currency/UUID state, condition
  asset recovery, duration, reward, research and node completion before one commit. Failure tests
  prove no partial rewards/state mutation.
- Attempt state records at most three committed results; retreat restores the complete pre-entry
  snapshot including RNG/UUID continuation and consumes no attempt.
- Employee condition is only physical damage 0..6 plus trauma 0..100; runtime HP is not written back.
  Per-instance incapacity is idempotent, newly dead employees retain roster identity while equipment
  and shared fragment return atomically, and death-prevent avoids recovery.
- No-living precedence, typed immediate Headquarters recruitment recovery for no-deployable state,
  one-node recheck, final-Boss immediate failure, and forced-Boss checkpoint exception all have
  direct tests in the 59-test combat suite.

Verdict:

- Aligned.

### DCA2-GAME-CLAIM-012 — Employee Psychology

Runtime/test evidence:

- Authored immutable personality remains in `EmployeeProfile`; `EmployeePsychology` separately owns
  clamped trust, exact band derivation, merged semantic memories, bounded eight-entry recent events,
  and source-idempotence.
- The only production `PsychologyEventKind` is Rest completion at +8 for living employees. Node
  completion advances optional memory duration; roster cloning in checkpoint/retreat preserves
  Psychology, and death asset recovery does not clear it.
- Psychology unit tests: 5 passed. Support and equipment tests assert Rest application,
  idempotence-adjacent completion behavior, projection, and preservation on a newly dead employee.

Verdict:

- Aligned.

### DCA2-GAME-CLAIM-013 — BossOmen chain and dedicated exit Gate

Runtime/data/test evidence:

- `BossOmenRunState` owns one provisional/active/placed/forced chain; application is Endless-only,
  requires awakened fragment/minimum Floor/catalog references, overlays exact Event/Combat source
  categories, defers when missing, and returns an ignored first source to skipped candidates.
- Completion consumes source only when the node completes, creates one ForwardOnly forced Boss while
  disabling other available nodes, and allows Safezone loadout through common planning capability.
- Forced defeat commits then fails; only matching forced-Boss RunFailed exposes checkpoint load.
  Forced victory replaces the terminal with one `boss_omen_exit_gate`, clears the chain and advances
  Floor/trauma only when that Gate is confirmed.
- Map-flow and combat suites directly cover overlay/defer/force, Event consumption, defeat/load,
  victory/exit Gate and final-Boss separation.

Verdict:

- Aligned.

### DCA2-BATTLE-CLAIM-001 — Basic attack range and weapon targeting foundations

Aligned portions:

- Runtime basic-attack eligibility uses final tile pattern/WholeFieldValidTiles and valid battlefield
  tiles, never `range_units` or body/world distance. Obstacles remain range cells; void/outside cells
  are clipped.
- Employee weapon profile overrides the default forward pattern. Every live normal corroded profile
  references an explicit range preset, and loader resolution makes its concrete pattern mandatory.
- Enemy ranged target persistence/reposition/blocker priority, profile ordering, release revalidation,
  and locked basic-projectile lifecycle are implemented in movement/targeting/projectile paths.
- Skill cast acquisition uses explicit reach/target range independently of basic attack. Global
  reach does not determine `TileArea` footprint, and normal corroded profiles cannot use
  WholeFieldValidTiles.

Mismatch `DCA2-BATTLE-001`:

- The contract says Boss/Elite/special units must have their own authored range/skill instead of the
  generic fallback. All 30 live abnormality `basic_attack.defense_tile_range` values are `None`, so
  `effective_basic_attack_tile_range` and range preview use the generic role fallback.
- Historical DCA-M-21/G05 explicitly leaves per-abnormality range authoring user-deferred until
  detailed combat designs and approved values exist. The audit therefore cannot invent patterns or
  weaken the canonical requirement.

Mismatch `DCA2-BATTLE-002`:

- `WeaponArchetype` includes `GrenadeLauncher`, but the targeting contract's current list omits it
  and the content catalog calls grenade a future candidate. No live equipment uses the variant.
- It is unclear whether the enum is an intentional reserved schema value or stale future vocabulary
  that should be removed until activation.

Test evidence:

- Core targeting tests: 11 passed; range preview tests: 7 passed; Core battle tests: 75 passed.

Verdict:

- Policy decision needed for the two mismatches; implemented generic targeting behavior otherwise
  matches the document.

### DCA2-BATTLE-CLAIM-002 — Automatic hostile target usefulness

Runtime/test evidence:

- `target_usefulness.rs` computes deterministic final damage through current defense/resistance,
  immunity, fixed damage and modifiers; random crit/miss is not a selection input.
- Zero-damage targets require target-applied hostile effects; heal/ally benefit/presentation/resource
  changes do not qualify. Interrupt-only selection requires a pending cast. Area/autocast gates on
  at least one useful target without adding damage-versus-effect priority.
- Core tests directly cover zero damage exclusion, hostile effect acceptance, enemy heal exclusion,
  area autocast waiting/acceptance and interrupt-only eligibility.

Verdict:

- Aligned.

### DCA2-BATTLE-CLAIM-003 — Cast target and step retarget lifecycle

Runtime/data/test evidence:

- `SkillTarget` contains exactly SelfUnit, required-rule EnemySingle and CastTarget;
  `StepTargetingMode` contains ReuseCastTarget/RetargetOnStep. Strict RON has no unknown
  weapon-profile inheritance field.
- Runtime captures cast target/tile, does not reapply acquisition range for reused direct targets,
  preserves area anchor tile after target disappearance, and retargets with the same cast-level
  reach/range/air rules only for `RetargetOnStep`.
- Live RON contains exactly the four RetargetOnStep occurrences listed by the contract.
- Core battle tests and `skill_runtime_contract` cover reuse, target disappearance, air eligibility,
  retarget execution time, mixed targets, conditional follow-up and cast/step separation.

Verdict:

- Aligned.

### DCA2-BATTLE-CLAIM-004 — Interrupt and projectile target lifecycle

Runtime/test evidence:

- Pending cast stores its start-log sequence; interrupt clears only a matching pending cast and
  records `SkillCastInterrupted`, while stale end events cannot invoke. Silence gates new casts but
  does not clear an existing cast. Withdraw records `SkillCastCancelled` and removes caster-owned
  active runtime.
- Basic projectiles use one locked target and launch source/side snapshot, sweep only against that
  target, do not recheck range at arrival, and record impacted `hit: false` for death/withdrawal.
- Skill projectile impacted events use nullable first-hit identity and targetless directional
  collision/pierce paths follow their explicit collision/stop policy.
- Core and integration tests directly cover manual/auto interrupt, no pending target, Silence,
  withdraw, basic projectile death/withdraw/range/source behavior, and skill projectile misses.

Verdict:

- Aligned.

### DCA2-BATTLE-CLAIM-005 — TileArea geometry and delivery

Runtime/data/test evidence:

- `DeliveryDef` has only Instant, Projectile and TileArea; no continuous Area shape types remain.
- TileArea resolves one concrete footprint over caster/anchor, intersects authored valid tiles,
  preserves obstacle cells, filters typed hit targets/include-caster and owns persistent tracking,
  tick/duration policy. ImpactContext anchors require prior spatial context through validation.
- Global unit reach is independent from footprint; captured target tile survives unit disappearance.
- Core tests cover clipped affected tiles, anchor origin, air targets, global captured tile,
  pre-step impact-context rejection, persistent area identity determinism and conditional delivery.

Verdict:

- Aligned for Core TileArea semantics. Unity consumption was closed separately as implementation
  drift under DCA2-BATTLE-CLAIM-006 / DCA2-WIRE-CLAIM-002.

### DCA2-RUNTIME-CLAIM-001 — Deterministic gameplay identity

Runtime/test evidence:

- `game/determinism.rs` derives stable seeds from run seed, namespace, full UUID, unit identity and
  explicit counters; `repath_jitter_ms` is bounded and deterministic.
- Battle crit/proc code has direct regressions proving that inserting unrelated event-log entries
  does not change the roll result. A source search found no RNG/seed expression coupled to
  `event_log_seq` or `next_event_seq`.
- `game::determinism::tests`: 3 passed.
- `independent_from_event_log_sequence`: 2 passed.

Verdict:

- Aligned. The audit does not infer deterministic equivalence for future random callers that do not
  reuse these identity helpers; new random paths still require their own regression.

### DCA2-RUNTIME-CLAIM-002 — Event log and checkpoint sequence boundary

Core evidence:

- `BattleEventLog` assigns immutable monotonically increasing sequence values and exposes entries
  after a client-provided sequence; `BattleUpdate` carries `events_delta`, authoritative
  `checkpoint`, and checkpoint `at_seq` from the same active simulation boundary.
- Event DTO tests prove explicit snake-case feedback tags and movement-stop time serialization.
- `game::battle::event_log::tests`: 2 passed; the previously completed world combat suite exercises
  setup/update/checkpoint production and revisioned setup recovery.

Verdict:

- Aligned after the Pass 4 server/Unity trace. Server pass-through, Unity delta/revision/gap checks,
  same-revision explicit resync and presentation-gated checkpoint reconciliation agree; see
  DCA2-WIRE-CLAIM-001.

### DCA2-RUNTIME-CLAIM-003 — Route, movement, collision and active checkpoint units

Core/data/test evidence:

- Official checkpoint projection filters to `RuntimeUnit::is_active`; retained inactive units are
  available only through the explicitly read-only `debug_inactive_units` surface.
- Setup snapshot route cells, runtime enemy movement plans and movement event segments derive from
  the resolved battlefield route. Generated/authored route validation enforces valid cardinally
  adjacent cells and rejects obstacle/void traversal.
- Movement projection records immutable segments only when the body actually moves; movement input
  includes authored static obstacles and non-rectangular void tiles.
- World combat tests directly cover an authored route matching setup/runtime, adjacent immutable
  movement segments, delayed-wave checkpoint membership, and inactive-unit omission. Core battle
  movement tests were included in the 75-test battle pass.

Verdict:

- Aligned after the Pass 4 consumer trace. Unity requires canonical route cells, projects movement
  events and reconciles Active checkpoint actors without rebuilding routes from endpoints; see
  DCA2-WIRE-CLAIM-003.

### DCA2-RUNTIME-CLAIM-004 — Result and diagnostic ownership

Runtime/test evidence:

- Completed battle state owns its event log, participant results and computed result stats only
  through the current CombatResult lifecycle; strict `EncounterCompendiumProfileState` explicitly
  rejects the removed `battle_records` field.
- `world/combat_trace.rs` writes optional developer JSON to
  `target/combat_traces/run_<seed>/<battle_uuid>.json`. Transition code catches every export error,
  logs a warning and continues to CombatResult.
- Result aggregation retains runtime-instance metrics but emits one chronological employee row,
  excludes player defense objects and keeps incapacitated employees MVP-eligible.
- `game::battle::result_stats::tests`: 4 passed; world combat tests include trace success/failure and
  result-transition assertions; compendium strictness tests: 2 passed.

Verdict:

- Aligned. The remaining `battle_records` occurrences are a negative parser test and historical
  audit notes, not live state or a second producer.

### DCA2-RUNTIME-CLAIM-005 — Atomic result commit and recovery

Runtime/test evidence:

- The same staged post-battle path clones employee, inventory, fragment, currency and UUID state,
  applies condition/reward/research/node effects, and swaps authoritative state only after every
  validation succeeds.
- Retreat consumes the immutable battle-entry snapshot and restores attempt, RNG/UUID, roster,
  inventory, content, VN trigger history and checkpoint state without creating a result.
- No-living/no-deployable/final-Boss precedence, forced-Boss checkpoint exception and read-only
  setup-loss recovery have direct assertions in the 59-test world combat suite.

Verdict:

- Aligned. This runtime claim reuses the independently traced gameplay evidence under
  DCA2-GAME-CLAIM-011 rather than treating duplicate prose as additional proof.

### DCA2-RUNTIME-CLAIM-006 through 008 — Commit subdomains

Runtime/test evidence:

- Safe-zone research delivery and permanent encounter compendium ownership were traced through
  planning-return staging, actual spawn-log collection, strict catalog validation and monotonic
  profile union under DCA2-GAME-CLAIM-008. Compendium unit tests: 2 passed.
- Inventory identity/quantity/reserve and Shop session/quote/transaction boundaries were traced
  through shared cloned-state commit under DCA2-GAME-CLAIM-004/005. Equipment, Support, Shop and
  node-session focused suites all pass.
- Reward candidates, atomic grants, exact per-employee XP distribution, survival XP and GrowthStack
  boundaries were traced through reward/world code and live policy under DCA2-GAME-CLAIM-008.

Verdict:

- Aligned for claims 006, 007 and 008. No competing runtime owner was found; external presentation
  of selected DTOs is checked separately rather than silently folded into these state claims.

### DCA2-RUNTIME-CLAIM-009 — Top-level Visual Novel and trigger runtime

Runtime/data/test evidence:

- `GameCoreState.visual_novel` is top-level and is not stored inside `RunState`; pre-run briefing
  creates no run/map until starter selection completes.
- Trigger matching is data-driven with priority/id ordering, active-session queueing, trigger-owned
  blocking, separate persist/rewind histories and strict time-threshold crossing. Removed
  definition-level `default_blocking` is rejected by a negative parser test.
- Root and battle projections use the same active runtime state. Battle-trigger evaluation occurs
  after the produced update and a newly blocking overlay pauses the next tick.
- `snapshots_and_start`: 22 passed; `support`: 12 passed; world combat's 59-test pass includes battle
  projection, trigger crossing, blocking pause and action preflight assertions.

Verdict:

- Aligned for Core/runtime ownership. Exact JSON consumption and Unity-local resource behavior
  remain scoped to the external wire/consumer claim, not this state-lifecycle verdict.

### DCA2-RUNTIME-CLAIM-010 — Core projection ownership

Core evidence:

- Checkpoint unit projection owns typed `unit_source`, HUD policy and player-only final
  `range_previews`; combat preview keeps `enemy_stat_scale` internal and omits it from serialized
  DTOs.
- Range-preview Core generation is covered by the 7-test focused pass, and setup/checkpoint actor
  production is exercised by world combat tests.

Verdict:

- Aligned after the Pass 4 server/Unity trace for actor identity, HUD, range and combat preview
  ownership; see DCA2-WIRE-CLAIM-003. Missing unrelated event-presentation consumers are isolated in
  DCA2-WIRE-CLAIM-002 rather than hidden here.

### DCA2-DOC-CLAIM-001 — Durable document vocabulary seam

Observed fact/finding:

- `docs/core_runtime_contract.md` describes current forced-Boss result commit as “G15 결과” and
  Shop purchase/consumable stack behavior as “G24 GrantExecutor/stage”. `G15` and `G24` are historical
  work-package labels, not domain types or symbols present in current runtime code.
- `docs/README.md` and `docs/engineering_workflow.md` say goal records are historical working memory,
  durable canonical documents should preserve current policy, and current behavior must not depend
  on goal history.

Inference:

- The surrounding behavior is traceable and agrees with current code, but the labels make a reader
  consult historical work to understand a current canonical. This is document drift, not an
  implementation mismatch.

Verdict:

- Document drift (`DCA2-DOC-001`), P3. A later documentation-only correction should replace the
  labels with current domain terms without changing policy.

### DCA2-DATA-CLAIM-002 — Transitive live-RON strictness

Document/accepted policy:

- `data_loading_contract.md#Live-RON-Authoring` says unknown/stale/removed gameplay fields are
  rejected without aliases and identifies the generated live bundle as exact-schema authoring.
- Historical accepted PD-09 and completed G12 are stronger: live gameplay RON is “maximally
  strict”, including unknown nested fields, and Core independently validates the generated bundle.

Runtime/probe evidence:

- Many outer database/raw structs correctly use `deny_unknown_fields`, and the data suite has
  useful negative tests for manifest, PvE, Event, Shop, VN, run policy, basic-attack authoring and
  selected removed fields.
- Strictness is not transitive into several live-reachable nested gameplay types:
  `MovementDef`, `ResonanceDef`, `TileRangePattern`, `DeliveryDef`, `SkillEffectDef`,
  `StatModifier`, and `MapNodePayload` do not deny unknown fields. The battlefield template
  database wrapper also lacks the attribute even though its nested template/route entries are
  strict.
- A temporary `/tmp/dca2_schema_probe` linked against the current `game_core` and passed extra
  `legacy_*` fields through actual `AbnormalityDatabase`, `SkillDatabase`, `EquipmentDatabase`, and
  `MapNodeDefinitionDatabase` deserialization. All four production-shaped inputs were accepted;
  direct nested-type probes were accepted as well.
- Example effect: an author can write both `speed_units_per_ms: 1000` and an obsolete
  `legacy_speed: 9999`; Core silently ignores the latter instead of reporting an authoring error.
- `cargo test -p game_core game::data --lib -- --test-threads=1`: 125 passed. The green suite does
  not contradict the probe because it has no negative tests for these nested unknown fields.

Verdict:

- Implementation drift (`DCA2-DATA-002`), P2. Current live RON has no observed offending field and
  loads successfully, so this is not a current state corruption. It is a real authoring contract
  breach and a regression/reopen of the completed G12/PD-09 disposition.

Test gap:

- yes. Each shared nested gameplay schema needs a production-shaped unknown-field regression,
  including at least abnormality/corroded movement and resonance, skill delivery/effects/range,
  equipment stat/activation effects, map payload and battlefield template wrapper.

### DCA2-DATA-CLAIM-003 — Canonical live source index and catalog validation

Runtime/data/test evidence:

- `GameplayRonSourceSet` contains one manifest plus 21 typed gameplay sources. The 17 main groups
  and four map/battlefield groups in `data_loading_contract.md` point to those same files and current
  domain owners; every listed file exists.
- `ParsedGameplayRon::validate_domain_catalogs` runs domain validation, duplicate ID/UUID and pool
  validation, skill/buff/fragment/equipment/reward/Event/VN/BossOmen/employee cross references,
  RunStart/extra-wave/map payload checks and map catalog validation before candidate publication.
- `ron_loading`: 20 passed; `game::data`: 125 passed. Duplicate and broken-reference cases return
  typed failures instead of first/last winner behavior.

Verdict:

- Aligned except for the already isolated source-count typo `DCA2-DATA-001` and the transitive
  nested strictness gap `DCA2-DATA-002`.

### DCA2-DATA-CLAIM-004 — Map and battlefield authoring/validation

Runtime/data/test evidence:

- The four map sources are parsed into the same `GameplayMapCatalogs` stored in `GameDataBase`;
  there is no separate production reader or builtin fallback.
- Map policies and node definitions reject duplicate/invalid weights, missing explicit Event
  references and broken payloads. ASCII battlefield parsing derives non-rectangular valid tiles,
  tile kinds, zones, obstacles and ordered route overlays, then validates route contiguity and
  common Defense endpoints.
- Generated preview validation exercises all live encounters across scaling stages/seeds before
  publication and checks wave routes and briefing warnings.
- `game::map`: 23 passed; `game::combat_preview`: 25 passed; `ron_loading`: 20 passed.

Verdict:

- Aligned for catalog ownership and semantic validation. Unknown-field strictness of the shared
  payload/template wrappers remains captured by `DCA2-DATA-002`, not hidden by this verdict.

### DCA2-CONTENT-CLAIM-001 — Roster policy and implementation-status distinction

Document/data/test evidence:

- `lobotomy_content_catalog.md` defines 3 final-boss, 14 ordinary-boss and 13 elite abnormalities
  (30 total) plus nine tool-only concepts. It explicitly says current IDs/effects/implementation are
  determined by live RON and that tools have no production schema yet.
- `abnormalities/base.ron` contains exactly those 30 combat abnormalities. The 20-test loading suite
  compares the complete live ID set against an explicit expected set.
- No tool ID from the nine-entry design table appears in the live abnormality or ordinary fragment
  catalogs; the content docs clearly classify them as future tool-schema candidates.
- `abnormality_skill_design_notes.ko.md` labels itself a content design note rather than a technical
  implementation contract. Its many encounter mechanics are therefore not treated as falsely
  implemented runtime behavior.

Verdict:

- Aligned. The documents preserve a usable distinction between confirmed roster/design intent and
  current implementation, with the WhiteNight exception below requiring a clearer disposition.

### DCA2-CONTENT-CLAIM-002 — Boss-linked and WhiteNight fragment policy

Document claim:

- `skill_fragment_system.md` says a resolved 12-person potential-apostle chain forces the WhiteNight
  boss, boss clear awards its fragment, repeated fragment activations carry apostle progress across
  battles, and completion of all 12 apostles causes a third-faction sequence and run failure.
- It also acknowledges that `fragment_white_night_trace` is a temporary live placeholder and that
  the final DTO/effect is not yet fixed.

Runtime/live evidence:

- Live BossOmen has `required_steps: 1` and one presentation-only confession Event. Core run state
  has generic provisional/active/placed/forced chain fields but no potential-apostle marks,
  apostleization progress, third faction or WhiteNight fragment-use failure state.
- `fragment_white_night_trace` is `BasicAttackModifier(+6 attack, -60 ms)` and has no active skill.
- The WhiteNight encounter reward UUIDs grant Enkephalin, equipment dust and unrelated high-risk
  research. The trace fragment comes from generic Endless response completion:
  `default_research_required: 100`, `victory_research_gain: 20`, then a queued unique-fragment grant.
  One boss clear therefore does not directly guarantee the documented containment reward.

Inference/verdict:

- This is not safely classifiable as a code regression because the same document calls the live
  implementation a placeholder and says final DTOs are future work. It is nevertheless not an
  aligned current gameplay contract: the acquisition rule and full risk loop materially differ.
- Policy decision needed (`DCA2-CONTENT-001`), P2: either retain the design as explicitly
  future/non-current and correct the boss-clear/live-placeholder wording, or activate a separately
  reviewed content/runtime/data/Unity implementation goal. The audit must not invent the mechanics.

### DCA2-CONTENT-CLAIM-003 — Derived live E.G.O/fragment index

Runtime/data/test evidence:

- Loader-backed probe counts: 30 abnormalities; 86 total equipment entries; 66 `placeholder_*`
  E.G.O entries forming 22 complete Weapon/Armor/Accessory groups; 31 fragments total; 30
  abnormality-origin fragments with 2 ActiveSkill and 28 BasicAttackModifier effects.
- The concept starter is the remaining fragment and is a BasicAttackModifier, matching the summary.
- A set comparison found no difference between all 66 backtick placeholder equipment IDs in the
  index and live equipment IDs. Every group has exactly three slot types and the documented rarity.
- Loader-backed per-fragment output matches every documented rarity and current effect summary;
  `live_skill_catalog_audit` independently proves all 30 abnormality-derived fragment IDs appear
  exactly once in the index.
- `live_item_skill_activation`: 3 passed for referenced live item skills and forced proc/battle-start
  behavior.

Verdict:

- Aligned. The document correctly calls the 66 count E.G.O placeholder equipment rather than total
  equipment and does not claim that placeholder effects implement the future encounter designs.

### DCA2-GAME-CLAIM-007 — Player-visible targeting and damage feedback

Evidence/verdict:

- Core target-usefulness preview applies the documented expected-final-damage or hostile
  target-applied-effect rule, and basic/skill selection uses the documented independent explicit
  authorities. Focused battle/targeting/skill suites and the broad Core suite cover those paths.
- Core records `HpChanged` with `final_damage`, typed breakdown and only the official
  critical/mitigated/fixed_damage/immune tags. Unity reads that event rather than recalculating
  defense, derives typed damage/heal/immune feedback, and schedules rendering by presentation time.
  Focused EditMode and PlayMode assertions are included in the passing full Unity suites.
- Aligned. Missing presentation for other battle event types remains the distinct
  `DCA2-WIRE-001` finding and does not negate the implemented `HpChanged` feedback path.

### DCA2-WIRE-CLAIM-001 — Battle transport, ordering and recovery

Document claim:

- External `docs/core_unity_battle_transport_contract.md` requires typed top-level setup/update,
  command acceptance before the resulting update, monotonic battle revision, explicit catch-up
  resync, setup-loss/reconnect recovery, and the final battle update before the combat-result root
  snapshot.

Producer/server/consumer evidence:

- Core `LiveBattleSetupSnapshotDto` and `LiveBattleUpdateDto` own the serialized setup, event delta,
  checkpoint, capabilities and revision.
- `game_server/src/game/player_game_actor/messages.rs::battle_update` forwards every update field
  without recomputation. `state.rs::behavior_result_to_command_result` emits `CommandAccepted` plus
  the required update/resync and suppresses a trailing root snapshot for battle commands.
- `handlers.rs` sends setup before the first update, root snapshot then setup/update on reconnect,
  error then non-mutating resync on stale authority, and final update before combat-result snapshot.
- Unity `GameSnapshotStore`, `GameWsClient` and `BattleRuntimeState` validate battle UUID, revision,
  delta continuity and capability/eligibility state; same-revision explicit resync remains
  applicable and active-battle commands do not fall back to stale root capabilities.

Test evidence:

- `cargo check -p game_server`: pass.
- `cargo test -p game_server -- --test-threads=1`: 30 passed.
- Connected Unity editor full EditMode: 241 passed, 0 failed/skipped.
- Connected Unity editor full PlayMode: 61 total, 57 passed, 0 failed, 4 explicit live-server tests
  skipped. The skipped tests require a separately running server and are not counted as proof.

Verdict:

- Aligned. The inspected producer, pass-through, message ordering, store validation and executable
  assertions agree. Presentation use of individual events is a separate claim below.

### DCA2-WIRE-CLAIM-002 — Required battle presentation events

Document claim:

- External `core_unity_battle_transport_contract.md#Required-Presentation-Events` makes the event
  stream the presentation source for deployment, movement, basic attacks, casts, projectiles,
  TileArea warnings/effects, HP/stat/resonance, death/withdrawal, buffs and battle end.
- Its `SkillAreaDeclared` section specifically requires Unity to render
  `shape.affected_tiles` rather than recomputing geometry. It also makes `UnitWithdrawn` and
  `UnitDied` event positions the inactive-transition presentation source.
- Core `skill_target_contract.md` and `core_runtime_contract.md` repeat those ownership rules.

Runtime/server evidence:

- `BattleLogEvent` exposes the documented typed events, including final TilePattern affected tiles
  and lifecycle world/tile positions. Core tests verify important event payloads and server forwards
  the event envelopes unchanged.

Unity evidence:

- `BattleRuntimeState.TryPresentationReadyAtMs` recognizes `SkillAreaDeclared` only to delay
  checkpoint reconciliation until `start_time_ms + display_duration_ms`; it never parses
  `shape.affected_tiles`.
- `BattlefieldRenderHandler.Render` projects the field, selected range preview and `HpChanged`
  damage feedback only. `BattlefieldFieldModel.ApplyBattleLiveProjection` consumes `UnitSpawned`
  for identity validation and movement start/stop for actor motion. `BattleDeployFeedbackState`
  consumes `UnitDeployed` only for pending-command confirmation.
- Exhaustive `PresentationEvents` consumer search found no presentation consumer for
  `SkillAreaDeclared`, `UnitDied`, `UnitWithdrawn`, attack/cast/projectile events, buff events,
  stat/resonance events or battle end. The Unity tests contain no `SkillAreaDeclared`,
  `UnitDied`, or `UnitWithdrawn` rendering assertion.

Inference/verdict:

- Unity correctly receives and queues the event stream but does not implement much of the visual
  behavior the canonical contract assigns to it. In particular, area warning/effect tiles and
  death/withdrawal position presentation cannot be reconstructed from the existing consumers.
- Implementation drift (`DCA2-WIRE-001`), P1. This is player-visible battle behavior across many
  events and includes missing telegraph information, not merely an optional animation polish gap.

Test gap:

- yes. The 241/241 EditMode and 57/61 non-explicit PlayMode passes do not exercise these missing
  consumers. A repair goal needs event-by-event projection tests and focused PlayMode rendering for
  TileArea, lifecycle, cast/projectile and buff behavior.

### DCA2-WIRE-CLAIM-003 — Typed actor, HUD, route and preview projections

Evidence/verdict:

- Server forwards Core setup/checkpoint/catalog shapes. Unity strictly consumes canonical route
  cells, checkpoint Active units, typed `unit_source`, employee/abnormality/corroded identities,
  player-only final range previews, screen-space HUD policy and catalog-v2 skill metadata.
- Unity rejects legacy route start/end fallback, opponent range previews, missing/mismatched actor
  identity and legacy skill-range authority. Focused assertions are included in the passing
  241-test EditMode suite, while battlefield assembler/render behavior is represented in the
  passing non-explicit PlayMode suite.
- Aligned for these state/projection surfaces. This verdict does not mask the event-presentation
  gap in `DCA2-WIRE-CLAIM-002`.

### DCA2-WIRE-CLAIM-004 — External current-enum inventories

Observed facts:

- Core `MapNodeCategory` currently has ten variants: Start, Combat, Support, Maintenance,
  HeadquartersContact, Event, Shop, Boss, Gate and Reward. External
  `unity_core_contract.md#MapNodeCategory` lists only seven and omits Maintenance, Event and Gate.
- Unity parses category as a string, so transport does not fail, but
  `NodeMapScreenComposer.MapNodeCategoryDisplayName` also lacks all three and returns the raw enum
  text instead of the localized label used for listed categories.
- Core `TargetingProfile` currently has four variants and no `SplashClusterFirst`. External
  `unity_core_contract.md` still lists `SplashClusterFirst` both in the equipment enum inventory and
  targeting-profile explanation. Repository history records its deliberate removal; searches find
  no current code or live-data use.

Verdict:

- Document drift (`DCA2-WIRE-002`), P3. The stale node-category list also exposes a small Unity
  display-label gap, but categories remain parseable and the current behavior is raw-label display;
  the principal mismatch is the external canonical's false current inventory.

### DCA2-WIRE-CLAIM-005 — Setup abnormality preload references

Document claim:

- External battle transport canonical says `catalog_refs.abnormality_ids` contains actual
  abnormality catalog IDs only. Corroded-employee `profile_id` and the empty
  `SpawnWaveEnemyEntry.abnormality_id` sentinel must not appear.

Runtime/live evidence:

- `BattleCoreState::live_setup_snapshot` correctly filters spawn-wave entries to non-empty
  `EnemyKind::Abnormality`, but also copies every `combat_preview.enemy_briefing.abnormality_id`
  without filtering by kind or emptiness.
- The registered broad live probe selected `normal_black_box_pressure`, whose two enemies were
  CorrodedEmployee profiles. Its successful `battle_setup_snapshot` nevertheless contained
  `catalog_refs.abnormality_ids: [""]`.
- The adjacent Core test only proves the corroded `profile_id` itself is absent from catalog refs;
  it does not assert that the collection contains only non-empty, catalog-resolved IDs.
- No current Unity production consumer of `catalog_refs.abnormality_ids` was found, so the empty ID
  does not currently crash the inspected client. It still violates the typed cross-layer meaning
  and would poison the documented preload input when that consumer is activated.

Verdict:

- Implementation drift (`DCA2-WIRE-003`), P2. The external policy is explicit and the repair is an
  unambiguous Core projection/filter plus regression assertion, but it is outside this audit goal.

### DCA2-GAME-CLAIM-014 — Maintenance authored coverage

Observed facts:

- The rulebook presents equipment dismantling/enhancement and fragment dismantling/upgrade/bloom
  as Maintenance verbs while correctly saying equipment crafting is not a current action.
- Runtime exposes per-target enabled/disabled operation previews. Fragment operations have generic
  runtime coverage, but the live equipment catalog authors exactly one dismantle recipe and one
  enhancement recipe, both for `standard_armor`; the other current equipment entries cannot execute
  those equipment operations.
- Equipment and Support focused suites prove the operation mechanics and node boundary, not broad
  live recipe coverage.

Verdict:

- Policy decision needed (`DCA2-GAME-003`), P2. This is the still-valid user-deferred DCA-M-28/G22
  content-scope question: either explicitly document the narrow current recipe coverage or resume
  approved equipment/Maintenance content authoring. The audit must not invent recipes or balance.

### DCA2-DOC-CLAIM-001 — External duplicate canonical-looking tree

Observed facts/verdict:

- Core README/workflow correctly point only to `/mnt/f/unity projects/ark/docs` and Core contains no
  forbidden local copies. However, the external project also has `/mnt/f/unity projects/ark/docs -
  복사본` containing `unity_core_contract.md`, `core_unity_battle_transport_contract.md`, and
  `unity_client_implementation_goal.md`.
- SHA-256 comparison shows every copy diverges from the canonical file. The copy tree is not linked
  by current docs/code, but its canonical names and adjacency make it a misleading live-looking
  source and it retains stale terms such as `SplashClusterFirst`.
- Document drift/discoverability (`DCA2-DOC-002`), P3. Cleanup requires separate authorization
  because the external duplicate may be user-owned backup material.

### DCA2-DOC-CLAIM-002 — Engineering workflow executability

Evidence/verdict:

- Current workspace packages are `game_core` and `game_server`; every selected workflow command
  used by this audit resolves and passes. Focused-to-broad ordering, goal status/lifecycle,
  dirty-worktree safety and debug-output non-authority match the repository structure and current
  master/status records.
- No `tests_bak` or `src/old` live source was found, no ignored official Rust/Unity tests were found,
  and negative tests mentioning old names explicitly reject them instead of providing runtime
  compatibility.
- Aligned. The two historical goal labels inside the runtime canonical and the external duplicate
  tree remain isolated governance findings under `DCA2-DOC-001/002` rather than invalidating the
  workflow itself.

### DCA2-DOC-CLAIM-003 — Probe registry

Evidence/verdict:

- All three registered scripts and all three adjacent contracts exist at the exact external paths.
  Script environment names/defaults match the registry, and server source/tests enforce the stated
  enable/token/production admin gate.
- The documented normal-server command and broad smoke command ran successfully through live
  battle setup/update/resync/deploy. The result is recorded in EXP-009; combat-result waiting was not
  enabled and is not overclaimed.
- Aligned. The live run also found `DCA2-WIRE-003`, demonstrating why probe success and individual
  payload assertions remain distinct evidence.

### DCA2-DOC-CLAIM-004 — Decision-record disposition

Evidence/verdict:

- Both current decision records explicitly identify themselves as historical inputs, link existing
  completion evidence and current canonical owners, and warn readers not to treat their initial
  recommendations/open questions as current policy.
- Current employee definition/recruitment/condition/Psychology and top-level pre-run VN claims were
  independently traced in Pass 1/2 and exercised by the focused and broad suites. The intentionally
  retained before/after descriptions in the records are therefore history, not competing current
  schemas.
- Aligned. All linked goal/canonical paths inspected here exist.

## Cross-Component Findings

Record combined-path issues here rather than hiding them inside one claim:

- Competing state or capability producers: none recorded yet.
- Core/server transport/order conflict: none found; presentation semantics diverge because Unity
  does not consume most required presentation events (`DCA2-WIRE-001`).
- Setup catalog preload meaning diverges because Core inserts the corroded empty sentinel into an
  actual-abnormality-only collection (`DCA2-WIRE-003`).
- Live data versus generated/runtime validation conflicts: current live data validates, but shared
  nested gameplay types still accept unknown fields contrary to PD-09 (`DCA2-DATA-002`).
- Debug/history artifacts acting as accidental authority: external current canonical still carries
  removed `SplashClusterFirst` despite the completed removal record (`DCA2-WIRE-002`), and a
  divergent `docs - 복사본` tree remains discoverable beside the canonical (`DCA2-DOC-002`).

## Historical Regression Checklist

The 2026-07-10 audit and subsequent cross-layer master are historical inputs only. Build a mapping
from each historical finding/disposition to current evidence before claiming that it remains fixed or
has regressed.

| Historical source/id | Current claim/finding | Current evidence | Disposition |
| --- | --- | --- | --- |
| DCA-H-04 / PD-09 / G12 | DCA2-DATA-CLAIM-002 / DCA2-DATA-002 | Production-shaped probe accepts nested unknown fields in four catalogs | regressed/reopen |
| DCA-M-08 / G22 | DCA2-GAME-CLAIM-009 / DCA2-GAME-001 | block promise still lacks schema/stat-pipeline support | remains user-deferred |
| DCA-M-21 / G05 | DCA2-BATTLE-CLAIM-001 / DCA2-BATTLE-001 | all 30 live abnormalities still rely on generic missing-range fallback | remains user-deferred |
| DCA-M-28 / G22 | DCA2-GAME-CLAIM-014 / DCA2-GAME-003 | live equipment recipes cover only `standard_armor` | remains user-deferred |
| DCA-M-26 / EX-01 | — | current craft-disabled wording and equip/loadout behavior remain non-contradictory | excluded disposition retained |
| Remaining 47 DCA findings | mapped current claims, no new finding | runtime/data/Unity searches, focused suites, broad Unity suites and live smoke | completed/not-applicable dispositions retained |
| 9 CPR aliases + 6 unique CPR findings | mapped DCA claims plus runtime/result/data claims | result transaction, bundle validation, capability and Unity recovery evidence | completed dispositions retained |
| MSTR-01..07 | current employee/result, legacy, Shop, compendium, battle authority and Unity test claims | focused Core/server suites, removed-vocabulary search, full Unity suites | completed dispositions retained |

Exact DCA IDs in the 47-finding retained group:

- `DCA-C-01`.
- `DCA-H-01`, `DCA-H-02`, `DCA-H-03`, `DCA-H-05`, `DCA-H-06`, `DCA-H-07`,
  `DCA-H-08`, `DCA-H-09`, `DCA-H-10`, `DCA-H-11`, `DCA-H-12`, `DCA-H-13`,
  `DCA-H-14`, `DCA-H-15`.
- `DCA-M-01` through `DCA-M-07`; `DCA-M-09` through `DCA-M-20`; `DCA-M-22`
  through `DCA-M-25`; `DCA-M-27`; `DCA-M-29` through `DCA-M-31`.
- `DCA-L-01` through `DCA-L-05`.

CPR alias and unique coverage:

- Aliases: `CPR-C-01`, `CPR-H-04`, `CPR-H-05`, `CPR-H-06`, `CPR-H-07`,
  `CPR-M-01`, `CPR-M-03`, `CPR-M-04`, `CPR-M-05`.
- Unique: `CPR-H-01`, `CPR-H-02`, `CPR-H-03`, `CPR-H-08`, `CPR-H-09`,
  `CPR-M-02`.

Revalidation interpretation:

- Historical completion was not accepted from checkboxes alone. The current audit re-traced the
  affected runtime/data/wire owners and reran current tests. Only transitive nested RON strictness
  has demonstrably regressed; three content-policy items remain intentionally deferred, and the one
  excluded finding remains non-actionable.
- `DCA2-WIRE-001`, `DCA2-WIRE-002`, `DCA2-WIRE-003`, `DCA2-DOC-001`,
  `DCA2-DOC-002`, `DCA2-BATTLE-002`, `DCA2-GAME-002`, `DCA2-CONTENT-001`, and
  `DCA2-TEST-001` are new/refined current-audit findings rather than inherited historical verdicts.

## Policy Questions

- DCA2-CONTENT-001: Is the detailed WhiteNight 12-apostle/fragment-risk design retained only as an
  explicitly future concept, or should it become an implementation goal? Until decided, do not
  reinterpret the current one-step placeholder or research acquisition as that design.
- DCA2-GAME-003: Should Maintenance remain intentionally narrow with only `standard_armor`
  equipment recipes and be documented as such, or should deferred G22 resume with approved recipe
  coverage/balance? Do not generate recipes from the generic verbs.

## Follow-Up Candidates

Do not implement these under the audit goal. Group candidates by common owner and validation surface
only after evidence is complete.

- Immediate correction candidates: restore transitive unknown-field rejection for shared live-RON
  gameplay types under one data-schema goal; filter/setup-test actual non-empty abnormality catalog
  refs; no live balance values need invention.
- Documentation correction candidates: source count 23→21; replace G15/G24 historical labels with
  domain terms; clarify current versus future WhiteNight acquisition/effect wording; repair the
  external MapNodeCategory and TargetingProfile inventories.
- Test/evidence hardening candidates: add production-shaped negative nested-field tests and a direct
  Endless all-response-complete RunComplete regression; add Unity event-projection/render tests.
- Cross-layer implementation candidates: implement canonical Unity presentation consumers for
  TileArea telegraphs/effect tiles, attack/cast/projectile lifecycle, death/withdrawal positions,
  buff/stat/resonance and battle-end events under a dedicated Core-server-Unity goal.
- Deferred policy candidates: abnormality authored attack ranges; equipment block semantics;
  active-only versus BasicAttackModifier fragment policy; reserved GrenadeLauncher schema;
  WhiteNight detailed risk loop; live equipment Maintenance recipe breadth.
