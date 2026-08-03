# Unity Bridge Protocol

`/game` is the active Unity single-player WebSocket route.

The route is intentionally a thin bridge:

```text
Unity JSON command
-> PlayerBehaviorRequest
-> game_core::game::behavior::PlayerBehavior
-> GameCore::execute_with_transport_context(...)
-> CommandResult
-> StateSnapshot from GameCore::get_run_snapshot_json()
```

## Request Naming

Command behavior payloads use `type` with `snake_case` values.

Examples:

```json
{ "type": "start_new_game", "game_mode": "Standard" }
{ "type": "select_starter_employees", "employee_definition_ids": ["field_medic", "breach_guard", "records_clerk"] }
{ "type": "recruit_employee", "employee_definition_id": "hq_field_medic" }
{ "type": "request_map_data" }
{ "type": "select_map_node", "node_id": "..." }
{ "type": "confirm_enter_node" }
{ "type": "finish_combat_replay" }
```

Active battle mutation commands also carry the exact revision that Unity rendered:

```json
{
  "type": "command",
  "request_id": "pause-001",
  "battle_authority": {
    "battle_uuid": "...",
    "battle_state_revision": 7
  },
  "behavior": { "type": "pause_battle" }
}
```

`battle_authority` is mandatory for active battle mutations. Read-only battle queries and
setup-loss recovery omit it. Core validates the supplied UUID/revision before mutation; the server
does not infer or replace stale authority.

Removed phase/suppression-era request names are not part of the active bridge:

- `request_phase_data`
- `select_event`
- `start_suppression`
- `finish_suppression_replay`
- `claim_combat_reward`
- `exit_combat_reward`

Use the current node-map and combat replay requests instead.

## Response Shape And Battle Ordering

Non-battle commands normally keep the existing two-message flow after command execution:

1. `command_result`
2. `state_snapshot`

`command_result.payload` follows the current `core_enhanced` `BehaviorResult` contract. Combat timelines are returned as `gzip+base64` payloads to keep WebSocket messages smaller.

Research completion delivery is an exact, one-shot fact on successful planning-return command
results. The server preserves `research_deliveries` on direct `NodeCompleted`/`FloorAdvanced`
payloads and inside typed recursive completions (`CombatRewardsGranted`, `EmployeeRecruited`,
`EmergencySuppliesGranted`). It does not synthesize delivery on node entry, reconnect, or snapshot
serialization. The command result is sent before the trailing `state_snapshot`; that snapshot may
already show the committed inventory and empty pending queue, but it is not a notification source.

`state_snapshot.state` is produced from `GameCore::get_run_snapshot_json()`. Outside an active
battle, its typed `action_capabilities` are the source of truth for current context actions. The
snapshot also owns map progression, selected node/session, roster, inventory, and resources.
It also forwards Core's typed placeholder-aware `encounter_compendium` root field unchanged; the server does
not derive discoveries/defeats or resolve catalog metadata. The host adds exact
`encounter_compendium_persistence { state, dirty, recovered_from_backup, reset_required, system_issue }` and
title recovery capabilities.

Employee transport is also pass-through authority. Starter and recruitment candidates use the
same typed `EmployeeDefinition` shape with authored `employee_definition_id` and required
`profile`; roster entries carry both run-local `uuid` and that authored identity/Profile.
The server does not derive personality from UUID/trust, reconstruct stats, or accept the removed
candidate `id`/`candidate_id` and command `candidate_ids` aliases.

The inventory projection is also pass-through authority. `reserved_slot_count` and
`occupied_slot_count` come from Core, equipment entries retain owned instance UUIDs, and consumables
are definition-backed stacks with required `quantity`. The server does not impose capacity, allocate
owned consumable UUIDs, synthesize `count`, or split/merge stacks.

The Shop projection and command payloads are likewise Core-owned pass-through contracts.
`selected_event.type == "shop"` carries stable typed offers with `offer_id`, explicit quantity,
authoritative total price/eligibility, refresh cost/use/reason, and quantity-aware sale candidates.
Purchase sends `offer_id`; sale sends the owned/static inventory selector plus positive `quantity`;
refresh sends `refresh_shop`. The server does not resolve merchandise, calculate buy/sell prices,
regenerate pages, translate the removed `reroll_shop`, or synthesize Shop eligibility.

The load-balance actor owns one process-shared local-install compendium persistence runtime, not a
transport-player map. Bootstrap injects the durable profile into every new `GameCore`; actor commands merge
the latest shared union before execution and publish the newest full union afterward. `AttachSession` also merges
the readable shared union before constructing its root snapshot, so reconnect cannot expose a stale smaller Core
profile. Fatal persistence keeps the title Retry/Reset snapshot path instead of treating an empty profile as
durable authority. The version-1 profile
store has its own current/backup/temp/lock/checksum lifecycle and never reads or writes an active-run payload.
Write failure leaves gameplay committed, marks the shared profile dirty, emits the loss-risk notification
once on the clean-to-dirty transition, and retries the full in-memory union. Battle traces are never an input.

Active-run persistence is a separate host-internal strict envelope version 3 containing Core payload version 2.
Core owns checkpoint `power_model_version` and current Floor anchor together with the checkpoint map/node
difficulty. The host envelope carries only monotonic `loads_used` and run peak; backup recovery keeps the backup
checkpoint's own anchor while max-merging those two values. This schema is not a Unity wire DTO, and old envelope,
Core payload, and removed monotonic-anchor fields are rejected rather than migrated.
The same opaque Core payload preserves each manifested difficulty's required typed monolith stage. The server does
not infer, strip, or migrate that field; Core rejects a missing stage field and role/stage/ledger mismatch.

An active battle uses revisioned side messages instead of root snapshot capabilities as execution
authority. `battle_setup_snapshot`, `battle_update`, and `battle_resync` carry a complete projection for one
`battle_uuid` and `battle_state_revision`, including:

- `checkpoint.units[owner == Player].range_previews` as the required deployed-player preview;
- no `range_previews` field on opponent/non-player checkpoint units. The server forwards this
  side-specific Core shape and does not add an empty compatibility object.

- typed context `action_capabilities`;
- checkpoint and playback payload eligibility;
- deployment candidate, withdrawal, and skill eligibility;
- current battle VN and ordered battle events.

Unity applies that projection atomically and does not reconstruct availability from resources,
targets, playback state, or a static battle whitelist. Every disabled payload uses a stable
`disabled_reason_code`; Core execution reuses the same preflight and validates again atomically.

Normal battle command delivery preserves the actor-owned side-message order around the request
result. When Core rejects missing or stale battle authority, the server preserves the distinct
request-scoped typed `error` code and then emits the current `battle_resync` from the same recovery
builder, with no intervening mutation. Event sequence is delivery continuity only; it is not a
substitute for `battle_state_revision`.

`start_new_game` now starts the run-start flow rather than immediately exposing
starter candidates. The command result type is `RunStartFlowStarted`, and the
following state snapshot enters `pre_run_briefing` with root `visual_novel`
populated from core. Unity should open the global VN overlay from that
`visual_novel` field. Starter candidates appear only after the HQ briefing VN is
completed and core transitions to `selecting_starter_employees`.

## Event Employee Check Delivery

The server deserializes `resolve_event_choice_check` as the exact Core identity command
`node_id,event_id,current_scene_id,choice_id,employee_uuid`. It does not accept or synthesize a roll,
DC, modifier, chance, key, outcome, reward, or compatibility alias.

For non-battle Event progress, the actor sends the request-scoped `command_result` first and the root
`state_snapshot` immediately after it. Both expose the same Core-authored Event inner DTO. Choice-
specific candidates, typed modifier/key provenance, chance and committed result are passed through
without server repair or recalculation.

An exact retransmission whose five identities match an already committed check returns that stored
result and the trailing snapshot without rerolling or granting twice. This execution idempotency does
not make resolve a presented capability on the result scene; a replay with any different identity is
rejected. Reconnect begins with `authed` and a root snapshot containing the same uncommitted or
committed Event session.

Static `skill_catalog.version == 2` separates cast acquisition (`cast_target.reach`, optional
`target_range`, and `air_capable`) from delivery geometry (`steps[*].tile_area.footprint`). Steps no
longer carry range/air authority. The server forwards this typed Core snapshot unchanged and does not
reconstruct removed `range_policy`/`defense_tile_range` fields.

## Deprecated Legacy Paths

The old multiplayer/matchmaking paths remain buildable for possible future reuse, but they are not the active single-player game bridge:

- `/ws/`
- `/events/stream`
- `matchmaking::*`
- `MatchCoordinator`
- Redis match/pubsub glue
