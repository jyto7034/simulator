# Core Unity Battle Setup Snapshot Contract Experiment Notes

This file records source-of-truth findings, implementation judgments, policy questions, and follow-up candidates.

## 2026-06-16 Initial Notes

Target contract:

- `/mnt/f/unity projects/ark/docs/core_unity_battle_setup_snapshot_contract.md`

Related contract:

- `/mnt/f/unity projects/ark/docs/core_unity_battle_update_contract.md`

Important initial conclusion:

- `battle_setup_snapshot` is not an emergency fix for local offline disconnects.
- It is a long-term responsibility split for battle scene construction.
- Current runtime likely has enough information through `CombatPreview`, active battle session, battle scenario, battlefield, and deployment validation paths.
- The first implementation should confirm that runtime source map before adding DTOs.

## Contract Principles To Preserve

- `combat_preview` remains pre-battle UI and player-facing preview.
- `battle_setup_snapshot` becomes the Unity battle runtime scene construction source.
- `battle_update.events_delta` remains the presentation timeline source.
- `battle_update.checkpoint` remains current state/recovery source.
- `command_result` remains command acknowledgement.
- `BattleStart` remains a timeline event; setup snapshot does not replace it.
- Setup snapshot must not contain mutable current-state fields that belong to checkpoint.

## Source-Of-Truth Questions To Answer By Reading Code

- Where is the active live battle's battlefield shape stored after battle creation?
- Does `CombatPreview` contain all route/deployment/spawn zone cells needed by Unity runtime?
- Which tactical points/static objects exist in runtime battle scenario but not in `CombatPreview`?
- Does live DefenseRoute movement use route data that can be exposed without recomputation?
- Does deploy validation use deployment zone data exactly matching what setup snapshot should expose?
- Are there start-time scenario units or defense objects that should be represented in `initial_units` or `static_objects`?
- Can setup snapshot be built from active battle session without reading live RON directly?
- What catalog references does Unity actually need to instantiate actors/effects without embedding large metadata?
- Where should setup snapshot DTOs live so server serialization and core tests share the same typed contract?
- How should `battle_setup_snapshot` side message order compose with the already implemented `battle_update` side message?

## Initial Implementation Judgment

Likely safe first slice:

- Add top-level `battle_setup_snapshot` server message.
- Add typed core setup DTO using existing runtime fields.
- Include battle identity, encounter id, node type, mission variant, battlefield, routes, deployment zones, spawn zones where available, tactical/static object references where available, and setup version.
- Send setup only when entering a live battle.
- Keep hard resync unsupported until setup snapshot is stable.

Likely unsafe without further reading:

- Making `initial_units` required.
- Embedding display/catalog metadata instead of ids.
- Removing or narrowing `combat_preview`.
- Sending setup snapshot from every battle state request as a migration fallback.
- Implementing hard resync in the same slice.

## Policy Questions If Encountered

Stop and ask the user if any of these become necessary:

- `initial_units` required vs optional vs omitted.
- ids-only catalog refs vs embedded display metadata.
- setup snapshot on `RequestBattleState { since_seq: None }`.
- hard resync response shape.
- global vs per-section setup version.
- `combat_preview` field removal/rename/meaning change.
- live RON schema/content change.
- temporary compatibility layer for old Unity client behavior.

## Follow-Up Candidates

These are not in scope unless required to finish the setup snapshot safely:

- Hard resync implementation for `request_battle_resync { need_setup: true }`.
- Unity client migration from `combat_preview` scene construction to `battle_setup_snapshot`.
- Dedicated WebSocket smoke probe for battle enter message order.
- Saving/resuming an in-progress battle using setup snapshot plus checkpoint.
- Splitting setup DTOs into smaller modules if `src/game/behavior.rs` becomes too large.

## 2026-06-16 Runtime Setup Inventory

Runtime source findings:

- Live battle start is created in `src/game/world/combat.rs::handle_map_combat_node`.
- The same `CombatPreview` used for pre-battle UI is already stored on `ActiveBattleSession`, so setup can reuse runtime-resolved battlefield/template/route/zone data without Unity reading live RON.
- The actual battle scenario is built by `CombatExecutor::build_battle_with_combat_preview`, then owned by `BattleCore`.
- `BattleCore` owns the authoritative `BattleScenario`; a read-only `scenario()` accessor is sufficient for setup snapshot construction.
- Tactical plan points and synthetic defense object placement are more authoritative in `BattleScenario` than in `CombatPreview`.
- Deployment/current mutable state remains in `LiveBattleDeploymentDto` under `battle_update.checkpoint`.
- The first battle update still carries `BattleStart` and start-time `UnitSpawned` events. Setup snapshot does not replace those presentation events.

DTO/source judgments:

- `battle_setup_snapshot` can be built from `ActiveBattleSession` plus `BattleCore::scenario()`.
- `initial_units` is emitted as an empty list for the current live DefenseRoute runtime because start-time units are presentation-spawned through timeline events, not pre-timeline scene actors.
- Defense objective data is not emitted under `static_objects`. It is a battle actor with HP/state, so `UnitSpawned` and checkpoint unit state are the single actor source.
- `catalog_refs` is ids-only and currently includes `battlefield_template_id` and sorted/deduplicated abnormality ids. No embedded display metadata was added.
- Route endpoints are emitted as `tactical_points` with `point_type: "route_endpoint"` so Unity can mark route objectives without deriving them from live RON.

Policy questions resolved from code evidence:

- No live RON schema/content change is required.
- No compatibility layer or dual schema is required.
- Hard resync is still out of scope; `need_setup: true` remains rejected.
- `combat_preview` remains intact as pre-battle UI data.

Remaining follow-up:

- If Unity later needs start-time enemy actors created before presentation events, revisit `initial_units` as a policy decision.
- If Unity needs localized names/icons in setup, decide ids-only vs embedded display metadata before extending `catalog_refs`.

## 2026-06-16 Review Follow-Up Notes

Review fixes applied after the first setup snapshot slice:

- `static_objects` is reserved for non-actor scene props. The current live DefenseRoute implementation emits an empty list.
- Defense objective creation remains authoritative through the presentation timeline (`UnitSpawned`) and current-state checkpoint, avoiding duplicate Unity actor creation.
- Setup battlefield dimensions, valid tiles, and obstacles now read from the runtime `BattleScenario` instead of `CombatPreview`; preview remains the source for presentation-only `tiles` and template id.
- Live battle tick scheduling now starts only after the WebSocket session has sent command response, setup/update side messages, and state snapshot.
