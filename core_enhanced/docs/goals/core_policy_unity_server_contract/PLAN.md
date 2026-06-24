# Unity and Server Contract Policy Implementation

## Objective

Consolidate gameplay command/result/snapshot contracts into typed core/shared DTOs while keeping the WebSocket transport envelope separate.

## Policies Covered

- `shared gameplay command DTO`
- `BehaviorResult boundary`
- `typed run snapshot DTO`
- `combat result timeline attachment ownership`
- `remove dormant battle resync setup field`
- `battle_records export`

## Plan

1. Read shared DTO crates/modules, server WebSocket transport, Unity command/snapshot expectations, battle update/resync flow, and export/debug artifacts.
2. Keep the WebSocket envelope as transport metadata only.
3. Merge duplicated gameplay command payload enums into core/shared typed DTOs.
4. Split `BehaviorResult` into domain result contracts and update server mapping at the boundary.
5. Convert run snapshot payloads to typed DTOs in one coherent migration unless code proves staged migration is safer.
6. Make core provide typed combat result timeline attachments and server handle compression/serialization.
7. Remove dormant `BattleResync.setup`; rely on battle setup snapshot plus battle updates.
8. Keep always-on debugging artifacts that are useful during development, including battle record export behavior confirmed by policy.
9. Update DTO serialization tests, WebSocket command tests, snapshot fixtures, and resync/update tests.

## Completion Conditions

- Gameplay command payloads have one shared typed source.
- Server transport envelope remains separate from gameplay semantics.
- Run snapshot is typed, with serialized shape protected by tests.
- Combat result attachments have typed core meaning and server-owned compression.
- `BattleResync.setup` is gone from normal contracts.
- Debug/export behavior remains available without per-session environment setup.
