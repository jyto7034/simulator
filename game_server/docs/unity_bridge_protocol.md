# Unity Bridge Protocol

`/game` is the active Unity single-player WebSocket route.

The route is intentionally a thin bridge:

```text
Unity JSON command
-> PlayerGameClientMessage::Command
-> ApplicationCommand::from_document(...)
-> CoreApplication::prepare_command(..., ApplicationExecutionContext)
-> host accepts a conditional active-run + profile application intent
-> PreparedApplication::commit()
-> CommandResult
-> StateSnapshot from CoreApplication::snapshot(ApplicationSnapshotKind::Run)
```

This is the configured paired-store mutation path. Automatic battle ticks use the same
prepare/accept/commit boundary. Queries use the public read/execute ports; an unchanged durable
checkpoint/profile needs no new intent. Host recovery commands use their explicit Core/host recovery
ports below. A host without active-run persistence retains direct `CoreApplication::execute` and,
when configured, the profile-only persistence behavior described below.

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
{ "type": "complete_combat_result" }
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

Use the current node-map, live-battle and result-acknowledgement requests instead.

## Response Shape And Battle Ordering

Non-battle commands normally keep the existing two-message flow after command execution:

1. `command_result`
2. `state_snapshot`

`command_result.payload` follows the public `ApplicationResult` kind/meaning/payload contract. Live battle messages preserve their existing update/setup/resync shapes. Production terminal results have no event-log attachment; `compressed_event_log` is omitted. Optional diagnostic traces are owned by the authoring/debug interface.

The command result is sent before the trailing `state_snapshot`, which reflects committed Core
state. Result receipts and reward rights follow the lifecycle below. Removed research progress and
`research_deliveries` are not current result fields or notification sources.

`state_snapshot` is serialized once from Core's typed `RunSnapshotDto`, flattened into the server snapshot DTO
with a private typed host overlay. Core fields and gameplay capabilities remain pass-through authority; the overlay
adds the optional active-run/profile status objects and composes only host recovery capabilities before final
serialization. Outside an active battle, the resulting typed `action_capabilities` are the source of truth for
current context actions. The snapshot also owns map progression, selected node/session, roster, inventory, and
resources.
It also forwards Core's typed placeholder-aware `encounter_compendium` root field unchanged; the server does
not derive discoveries/defeats or resolve catalog metadata. The host adds exact
`player_profile_persistence { state, dirty, recovered_from_backup, reset_required, system_issue }` and
host recovery capabilities. The server consumes Core's non-serialized typed `ActionRoute` to identify exactly
`ContinueRun`, `ResetActiveRun`, `RetryPlayerProfile`, and `ResetPlayerProfile` as host recovery;
their scope comes from the same command descriptor. This metadata does not route those operations through Core or
change their persistence, replacement, result, notification, snapshot, or ordering lifecycle.

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

## Durable Application And Recovery

The load-balance actor owns one process-shared local-install profile persistence runtime, not a
transport-player map. Bootstrap injects the durable profile into each new `CoreApplication`. Commands
and `AttachSession` merge the readable shared permanent union before using Core's current state;
reconnect does not reconstruct discoveries or replace the union with a smaller actor-local profile.
The player profile remains version 3. Battle traces are never persistence input.

With active-run and profile stores configured, mutations and automatic battle ticks prepare one
exclusive Core candidate. The host uses profile-then-active lock order and accepts
`application_commit.pending.json` containing the successor profile and active document or terminal
tombstone before committing the prepared Core root. Persistence failure before intent acceptance
returns `application_persistence_failed` and discards the candidate. Once the intent is renamed into
place, later sync/publication/cleanup failure keeps the accepted Core result and reports the
`application_recovery_pending` notification and persistence state. It does not report the mutation
as an unaccepted command or roll the Core root back. Pending recovery blocks further ordinary
mutations and automatic ticking; queries and the explicit recovery path remain available.
`retry_player_profile` and process startup replay the same accepted intent
idempotently. Reset cannot discard a pending intent. Projection or socket delivery failure does not
cancel accepted gameplay. See the [Core durable boundary](../../core_enhanced/docs/core_runtime_contract.md#durable-active-run-host-boundary)
for the exact preparation and publication contract.

The active-run store uses strict envelope version 5 containing Core continuation version 11, with a
checksum and opaque generation UUID. Conditional publication compares the actor's expected
generation with the current generation; a stale writer receives `active_run_revision_conflict`
without publishing its candidate. Tombstones retain a generation, so an actor that observed an old
run or missing file cannot overwrite a later completed, reset or replacement run. A client must
consume the resulting error/recovery state, not infer successful selection or grant from its local
preview. Battle UUID/revision authority and this host persistence generation are separate contracts;
Unity does not manufacture a persistence revision field in gameplay commands.

After completed publication, current and backup contain the same latest confirmed generation,
including the active payload or tombstone and monotonic load/peak metadata. Backup recovery does not
select an earlier gameplay checkpoint or erase a confirmed reward/choice. A recovered active-run
backup is surfaced by `active_run_recovered_from_backup`. Core owns `power_model_version`, current
Floor anchor and map/node difficulty together; the host preserves the opaque Core payload and
monotonic `loads_used`/run peak. Explicit SavePoint rewind restores its own anchor/map generation
while load use and peak do not decrease.

These storage schemas are not Unity wire DTOs. Unsupported active/profile/current/backup or journal
versions, including old V4/V9 saves and the retired Standard-only journal, are protected before
Continue, reset, recovery or new-run publication. They are not silently migrated, overwritten,
deleted or replaced with empty state. Ordinary corruption and unsupported versions have distinct
recovery outcomes. Core import validates checksum, content identity, required fields, seed,
map/progression, roster, ownership, inventory, fragment and monolith stage/ledger graphs, and
result/choice/phase context before mutation. Restore validates the chosen checkpoint graph, and
explicit load additionally requires the live run seed. The host does not repair or rebind an
inconsistent payload.

When only profile persistence is configured and active-run persistence is absent, the retained
profile-only path publishes the shared union after direct Core execution. A profile write failure
then leaves gameplay committed, marks the profile dirty and emits `player_profile_save_failed`
once on the clean-to-dirty transition; retry writes the full in-memory union. This mode is distinct
from the paired-store accepted-intent/recovery-pending contract above. Protected/fatal states retain
explicit recovery status rather than treating an empty profile as durable authority.

## Result, Choice, Retreat And Continue

Core settles a finished battle in its terminal transaction, including employee condition/death,
consumable use, unconditional rewards, node/encounter completion and permanent profile changes.
Both manual execution and automatic host ticks publish that settlement through the same prepared
boundary. `complete_combat_result` acknowledges the stored compact receipt and moves to its next
phase; it does not grant the settlement again. Unity displays Core's result summary and consumes
the acknowledgement outcome, even when an existing result type contains reward information.
Production results have no event-log attachment and Unity does not reconstruct settlement from logs.

For a nonempty `ChooseOne`, Core preserves the fixed candidates and original grant context as an
unclaimed right. The client uses the existing `select_reward`, `claim_reward` and reward-exit flow
as allowed by current Core capabilities. Selecting does not grant; claiming applies the selected
effect once. Repeated claims and stale actors cannot grant twice. Reopen, reconnect and process
restart preserve the same candidates and selected/claimed state. An empty candidate list skips
selection; the host does not invent a replacement reward. A terminal result with an outstanding
choice remains continuable until the required choice flow is complete.

Successful `retreat_battle` restores the battle-entry state and consumes one of two successful
retreats for that node. A third challenge has no retreat. Refusal consumes nothing, and victory or
defeat settles and consumes the node. Core's capability/reason is authoritative; the host does not
derive retreat availability from a local counter. Continue preserves successful retreat use;
explicit SavePoint load rewinds run-local retreat state to that SavePoint.

`continue_run` restores the latest confirmed continuation, including before the first SavePoint.
For an unfinished combat, Event, Shop, maintenance, ordinary reward, headquarters or sweep node it
restores the whole node's pre-entry boundary, with successful retreat use preserved. Confirmed
result receipts and combat reward rights survive separately. It does not resume mid-battle or spend
a SavePoint load. In contrast, `load_run_checkpoint` restores the independent SavePoint and consumes
one of the three monotonic loads; restart and backup recovery cannot restore spent loads. Permanent
profile records remain outside the rewound run-local state. A completed, failed or abandoned run
without an outstanding choice publishes a tombstone. Replacing an existing active run requires
explicit `replace_active_run: true` on `start_new_game`.

Disconnect/reconnect to the same live actor preserves and pauses its battle; it is different from
process restart or explicit `quit`, which use the stored continuation on a fresh actor. A native
runtime fault remains the first Core fault across reconnect and exposes `battle_faulted` with no
gameplay capabilities and `quit_then_continue_valid_checkpoint` recovery. It cannot publish a new
normal save or overwrite the previous valid files. Persistence recovery-pending is a separate host
condition, not a native battle fault. Unity consumes the supplied state, typed errors and recovery
capabilities instead of retrying gameplay blindly or rebuilding these rules locally.

## Active Battle Authority And Ordering

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

The player actor serializes active-battle mutations one request at a time. A client sends the next
mutation only after the preceding request result and revisioned update/resync. If two mutations use
the same rendered revision, the first may commit and increment the revision; the second is then
rejected as stale and receives `error` followed by the current `battle_resync`. The server does not
batch those requests or roll the first commit back.

Pause freezes only Core's production simulation tick. It does not create a separate command route or
relax payload eligibility. Accepted deploy, withdraw, manual-skill start, retreat, and stored-speed
mutations are delivered immediately at the frozen battle time and each consumes one revision;
read-only battle state/recovery queries consume none. Scheduled simulation work and battle-time
cooldowns do not advance until an explicit successful resume, and the server does not catch up wall
time spent paused.

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

## Trusted runtime profile bootstrap

The host validates optional `runtime_safety_path` before loading the required external content selection.
Omission selects embedded defaults; an explicit invalid path fails startup before persistence or actor creation.
`ValidatedGameData` retains the immutable selected profile through the load-balance actor into each application.
See the [loading contract](../../core_enhanced/docs/data_loading_contract.md#common-default-release-authoring-cli)
for packaging and startup, and the [runtime contract](../../core_enhanced/docs/core_runtime_contract.md#trusted-runtime-safety-profile)
for effective identity, current-host checkpoint validation, and the opaque persistence rollback capability.

## Employee resonance use projection

`battle_update.checkpoint.deployment.deployed_units[].skill_readiness` carries the Core-owned
`resonance_current`, `resonance_max`, `resonance_use` and `resonance_gain_block_reasons`.
`resonance_use` is null after the actual use ends; otherwise it contains `action_ordinal`, `skill_id`,
`consumed` and `phase` (`accepted`, `casting`, `acting`, `effects`). Block reasons are independent:
`hard_crowd_control`, `skill_use`, `autonomous_recovery`. Unity renders these values directly.

A successful final employee activation consumes full resonance at admission, including while paused.
The accepted action retains its credential with zero remaining resonance. Duplicate activation is refused;
resuming and reconnecting to the same actor do not consume again. A paused actor's pending action and
use projection remain in memory. A new process Continue restores the pre-entry/confirmed durable boundary
under external Core continuation document version11; it does not restore the paused live battle. The standalone
Core checkpoint payload encoder uses version12 and is a separate format. Version10 originals are protected
from overwrite. Fragment form projections include the required positive `resonance_max`.

Employee regeneration follows whole battle-second boundaries, with no accumulated gain for blocked or
capped pulses. The host supplies elapsed battle time and serializes Core projections; it owns no parallel
regeneration or use-expiry timer.

## Relic commands and projections

Relics use the existing generic `command` envelope with a `behavior` object and Core command-result plus
snapshot flow. The host parses Core's `ApplicationCommand` and publishes its typed results; it does
not maintain a parallel Relic inventory or recompute numeric authority. Command `type` values and
fields come from [`PlayerBehavior`](../../core_enhanced/src/game/behavior.rs):

| `type` | Fields |
| --- | --- |
| `request_relic_state` | none |
| `request_relic_numeric_plan` | `employee_uuid` |
| `equip_relic` | `copy_uuid`, `employee_uuid`, `slot` (0..2) |
| `unequip_relic` | `copy_uuid` |
| `preview_relic_corruption`, `corrupt_relic` | `copy_uuid` |
| `preview_relic_dismantle` | `copy_uuid` |
| `dismantle_relic` | `copy_uuid`, `confirmed` |
| `choose_relic_corruption` | `node_id`, `event_id`, `current_scene_id`, `copy_uuid` (UUID or null) |

Core applies its current action/context capabilities to these commands. The first two and both
previews are queries. Other commands retain Core mutation and durable-publication authority.
`RelicState`, `RelicPreview` and `RelicNumericPlan` are the exact command-result kinds; their payloads
retain Core's `result` object. [`RelicStateDto`](../../core_enhanced/src/game/relic.rs) supplies copies
(`copy_uuid`, `definition_id`, `variant`), employee loadouts with three slots, typed `resources`
(`equipment_dust`, `relic_corruption_catalysts`) and optional `not_sellable_copy_uuids`. This is a
dedicated response; the existing `inventory` snapshot has no added Relic field. Render `Normal` and
`Corrupt` exactly as Core serializes them and retain copy UUIDs when a variant changes.

Preview exposes the selected effects/penalties, `catalyst_cost`, `equipment_dust_gained`,
`irreversible` and `equipped_to`. It consumes nothing and does not certify later mutation eligibility.
Ordinary corruption consumes one catalyst and keeps the copy UUID. Confirmed dismantle removes and
unequips the copy; a declined confirmation leaves it unchanged. Relics cannot be sold in Shop.

An Event interaction of `type: relic_corruption` offers Core's eligible candidates, `can_decline`
and `catalyst_cost: 0`. Send the original source scene as `current_scene_id`, with a selected copy
UUID or null to decline. An exact committed retry is idempotent after scene advancement; Unity must
not infer a new grant or consume a local catalyst. Core preserves and validates source-scene/copy
selection history, including a copy legitimately dismantled later.

The numeric plan's scope is `prospective_loadout`: generation, provider addresses and active/dormant
effects explain a loadout, without granting an actor/action/epoch execution credential. Available
amounts exclude live aura/target/critical state; unresolved and rejected states remain explicit.
Actual damage uses the original captured Core source, with the existing basic aura before Relic
composition. Render the projection rather than replaying its arithmetic as battle authority.

Current and SavePoint Relic ownership, materials, UUID stream and Event history are part of Core's
strict save graph. Host ActiveRun V5 wraps external continuation V11; standalone Core payload V12
is a separate encoder format. Continue does not restore a paused live battle. See the
[Core Relic contract](../../core_enhanced/docs/core_runtime_contract.md#relic-copies-numeric-authority-and-save-graph)
for immutable generation lifetime and the limited meaning of test-only Weak observations.

## U06 battle interaction and previews

New battle mutations use the existing battle UUID/revision envelope: `focus_battle_unit`
(`unit_instance_id`), `clear_battle_focus`, `begin_deployment_selection` (`employee_uuid`),
`cancel_deployment_selection`, `open_battle_settings`, `close_battle_settings`. They return
`BattlePlaybackChanged` with the committed checkpoint. Core owns the shared selection speed of0.5,
Pause, settings input gate and disconnect restore priority; clients render the returned values.

`preview_tactical_command` is read-only. Its `command` is tagged `relocate` (employee_uuid,
destination, facing) or `swap` (first_employee_uuid, second_employee_uuid, first_facing,
second_facing). `TacticalCommandPreview` carries current revision, per-unit destination ranges,
TP/cooldown/forced-defense values and eligibility. The eventual mutation validates again.
Checkpoint `interaction`, unit `basic_action`, and tactical forced-defense deadline/remaining time
are authoritative. General and skill selection expose actual basic candidates and block exceptions.
Do not infer Heal targets or tactical availability from local weapon metadata.

## U07 independent objects and wall cost preview

Existing `activate_skill` and `preview_skill_target` envelopes carry object creation; no parallel client mutation is added.
Skill target preview may include `object_placements` with predicted definition/evolution, position, HP, health cost and
segment detachment. It is a current-state forecast; the eventual release validates again.

Live checkpoint `independent_objects` contains active object IDs, kind/form, position, epoch/TTL, HP where applicable,
creation source/copy/form and capabilities, creation settlements, and the latest removal. `sustained_effects` projects
segmented pool remaining HP, derived stacks/top/block bonus and the latest detachment. Clients render these Core values;
they must not infer a second pool balance or treat a trap/wall as a selectable employee. Same-actor reconnect preserves
pending creation while paused and existing objects without advancing TTL. Fresh Continue retains the established durable
pre-entry behavior; these fields do not introduce live battle persistence. Unity rendering is subsequent work.
