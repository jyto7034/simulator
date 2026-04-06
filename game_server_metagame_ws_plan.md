# Game Server MetaGame WS Implementation Plan

This document describes the intended implementation plan for the Unity meta-game connection path against `game_server`.

The target audience is another engineer or AI agent that will implement the server-side changes in:

- `/mnt/f/work/simulator/game_server`
- `/mnt/f/work/simulator/core`

This plan is written from the current codebase state as of 2026-03-28.

## 1. Goal

Implement the missing Unity-facing meta-game session path so that:

1. Unity connects to `game_server` over WebSocket.
2. The server creates or reuses a per-player `PlayerGameActor`.
3. `PlayerGameActor` owns a `GameCore` instance.
4. Unity sends game commands expressed as `PlayerBehavior`.
5. The server executes `GameCore::execute(player_id, behavior)`.
6. The server returns `BehaviorResult`-based responses to Unity.
7. PvE replay timelines are forwarded to Unity.
8. Reconnect is supported by rebinding the socket to the existing player actor.

This is separate from the existing matchmaking WebSocket path at `/ws/`.

## 2. Current State

### 2.1 `core`

`core` already has the main game logic needed for the meta-game loop:

- command input: `/mnt/f/work/simulator/core/src/game/behavior.rs`
- result output: `/mnt/f/work/simulator/core/src/game/behavior.rs`
- state transitions and execution: `/mnt/f/work/simulator/core/src/game/world.rs`
- DTOs for event selection and shop/bonus/random previews:
  - `/mnt/f/work/simulator/core/src/game/enums.rs`

Important existing commands:

- `StartNewGame`
- `RequestPhaseData`
- `SelectEvent`
- `PurchaseItem`
- `SellItem`
- `RerollShop`
- `ExitShop`
- `ClaimBonus`
- `ExitBonus`
- `StartSuppression`
- `FinishSuppressionReplay`
- `EquipItem`
- `MoveUnit`

Important existing results:

- `StartNewGame`
- `RequestPhaseData(Box<PhaseEvent>)`
- `EventSelected`
- `ShopState`
- `RerollShop`
- `PurchaseItem`
- `SellItem`
- `RandomEventState`
- `BonusReward`
- `SuppressAbnormality { winner, timeline }`
- `RewardState`

### 2.2 `game_server`

`game_server` currently implements matchmaking-oriented WebSocket infrastructure, not the Unity meta-game session path.

Relevant files:

- server entry:
  - `/mnt/f/work/simulator/game_server/src/main.rs`
- shared protocol:
  - `/mnt/f/work/simulator/game_server/src/shared/protocol.rs`
- legacy matchmaking session:
  - `/mnt/f/work/simulator/game_server/src/matchmaking/session/mod.rs`
  - `/mnt/f/work/simulator/game_server/src/matchmaking/session/handlers.rs`
- load balancer:
  - `/mnt/f/work/simulator/game_server/src/game/load_balance_actor/mod.rs`
  - `/mnt/f/work/simulator/game_server/src/game/load_balance_actor/messages.rs`
  - `/mnt/f/work/simulator/game_server/src/game/load_balance_actor/handlers.rs`
- player actor stub:
  - `/mnt/f/work/simulator/game_server/src/game/player_game_actor/mod.rs`
  - `/mnt/f/work/simulator/game_server/src/game/player_game_actor/handlers.rs`
  - `/mnt/f/work/simulator/game_server/src/game/player_game_actor/messages.rs`
  - `/mnt/f/work/simulator/game_server/src/game/player_game_actor/state.rs`

Current facts:

- `/ws/` exists and is for legacy matchmaking/test-client flow.
- `/events/stream` exists and is for event streaming.
- `PlayerGameActor` exists only as a stub.
- `LoadBalanceActor` already stores `player_id -> Addr<PlayerGameActor>`.
- `main.rs` does not yet expose `/game`.

## 3. Confirmed Product Decisions

These decisions were already confirmed and should be treated as fixed for the initial implementation.

### 3.1 Transport

- Use `WebSocket + JSON`.

### 3.2 Auth

- Keep an auth step in the protocol.
- Use mock auth for now.
- No real auth server integration in the first implementation.
- `game_server` itself handles mock auth.

### 3.3 Identity

- Unity sends `player_id`.
- The server trusts the client-provided `player_id` for now.
- One client connection corresponds to one player session.

### 3.4 Snapshot policy

- After successful auth, the server sends a full `state_snapshot`.
- No patch/delta protocol is required for the first implementation.

## 4. Target Architecture

The intended request path is:

`Unity -> /game WebSocket -> PlayerGameActor -> GameCore -> BehaviorResult -> Unity`

If PvP is involved later:

`PlayerGameActor -> MatchCoordinator / Matchmaker -> battle result -> PlayerGameActor -> Unity`

Responsibilities:

- `core`
  - game rules
  - authoritative state transitions
  - replay timeline generation
- `game_server`
  - network session handling
  - player actor lifecycle
  - reconnect support
  - protocol validation
  - calling `GameCore`
- Unity
  - rendering
  - local state store for UI
  - command dispatch
  - replay visualization

## 5. Protocol Design

Do not reuse the current matchmaking `ClientMessage` / `ServerMessage` protocol from `shared/protocol.rs` for Unity meta-game traffic.

Create a separate protocol for `/game`.

Recommended file:

- `/mnt/f/work/simulator/game_server/src/game/player_game_actor/messages.rs`

Recommended wire message layout:

### 5.1 Client -> Server

#### Auth

```json
{
  "type": "auth",
  "player_id": "7d5d63d2-8f5b-4d40-9dcf-6f839b1a64d2",
  "token": "dev-anything"
}
```

#### Command

`behavior` should directly map to `PlayerBehavior` JSON.

```json
{
  "type": "command",
  "request_id": "req-1",
  "behavior": {
    "type": "StartNewGame"
  }
}
```

```json
{
  "type": "command",
  "request_id": "req-2",
  "behavior": {
    "type": "PurchaseItem",
    "item_uuid": "..."
  }
}
```

#### Ping

Optional.

```json
{
  "type": "ping"
}
```

### 5.2 Server -> Client

#### Auth OK

```json
{
  "type": "authed",
  "player_id": "7d5d63d2-8f5b-4d40-9dcf-6f839b1a64d2"
}
```

#### Command Result

```json
{
  "type": "command_result",
  "request_id": "req-2",
  "ok": true,
  "result_type": "PurchaseItem",
  "payload": {
    "enkephalin": 7,
    "inventory_diff": { ... }
  }
}
```

#### Error

```json
{
  "type": "error",
  "request_id": "req-2",
  "code": "invalid_action",
  "message": "PurchaseItem is not allowed in the current state"
}
```

#### State Snapshot

```json
{
  "type": "state_snapshot",
  "state": { ... }
}
```

#### Battle Replay Ready

Either use a dedicated message:

```json
{
  "type": "battle_replay_ready",
  "winner": "Player",
  "timeline": { ... }
}
```

or return this through `command_result` with `result_type = "SuppressAbnormality"`.

For the first implementation, reusing `command_result` is acceptable.

## 6. Mock Auth Design

Implement auth inside `game_server`.

Recommended abstraction:

- `MockAuthProvider`
- or simple helper function in the `/game` flow

Behavior:

- If `player_id` is a valid UUID, auth succeeds.
- `token` is accepted but not actually verified.
- Missing token may still be accepted in development mode.

This should be structured so that a real auth provider can replace it later.

Possible trait shape:

```rust
trait AuthProvider {
    fn authenticate(&self, player_id: Uuid, token: Option<&str>) -> Result<AuthSession, AuthError>;
}
```

Initial implementation:

- `MockAuthProvider`

Later implementation:

- `RemoteAuthProvider`

## 7. `PlayerGameActor` Design

`PlayerGameActor` is the main server-side session runtime for one player.

Recommended struct contents:

```rust
pub struct PlayerGameActor {
    player_id: Uuid,
    game_core: GameCore,
    socket: Option<Recipient<PlayerGameServerMessage>>,
    load_balance_addr: Addr<LoadBalanceActor>,
    match_coordinator_addr: Addr<MatchCoordinator>,
    redis: ConnectionManager,
    metrics: Arc<MetricsCtx>,
    connection_state: PlayerConnectionState,
    last_known_game_state: Option<GameState>,
}
```

Notes:

- The actor should own `GameCore`.
- The actor should not depend on Unity-specific logic.
- The actor should support rebind on reconnect.

### 7.1 Responsibilities

- accept auth success / socket attach
- send initial `state_snapshot`
- receive command messages
- deserialize `PlayerBehavior`
- call `game_core.execute(player_id, behavior)`
- map `BehaviorResult` to wire responses
- handle reconnect
- later: enter/leave PvP queue

### 7.2 Do Not Put These In The Actor

Do not duplicate game rules:

- shop rules
- event selection rules
- bonus reward rules
- battle rules

All of those stay in `core`.

## 8. `LoadBalanceActor` Changes

Current `LoadBalanceActor` already stores player actor addresses.

Missing capability:

- query existing actor by `player_id`

Add message types like:

```rust
pub struct FindPlayer {
    pub player_id: Uuid,
}
```

```rust
impl Message for FindPlayer {
    type Result = Option<Addr<PlayerGameActor>>;
}
```

This is needed for reconnect.

Recommended additions:

- `FindPlayer`
- maybe `HasPlayer`

## 9. `/game` Endpoint Plan

Add a new route in:

- `/mnt/f/work/simulator/game_server/src/main.rs`

Recommended route:

```rust
#[get("/game")]
async fn player_game_ws_route(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error>
```

Recommended flow:

1. Start a lightweight WS session actor or directly start `PlayerGameActor`.
2. Wait for first `auth` message.
3. Extract `player_id`.
4. Run mock auth.
5. Ask `LoadBalanceActor` for existing actor.
6. If found:
   - reuse actor
   - rebind socket
7. If not found:
   - create `PlayerGameActor`
   - build `GameCore`
   - register in `LoadBalanceActor`
8. Send `authed`
9. Send `state_snapshot`

Important implementation choice:

### Option A: `PlayerGameActor` is itself the WebSocket actor

Pros:

- simpler topology
- one actor per player/session

Cons:

- WebSocket protocol and game runtime become tightly coupled

### Option B: dedicated WS session actor forwards to `PlayerGameActor`

Pros:

- cleaner separation between transport and game runtime
- reconnect is cleaner

Cons:

- more moving parts

Recommendation:

- For long-term cleanliness, use Option B.
- If speed matters more, Option A is acceptable in the first iteration.

If Option B is used:

- create a small `/game` WebSocket session actor
- it handles `auth`, socket I/O, ping/pong
- it forwards validated commands into `PlayerGameActor`

## 10. Initial State Snapshot

After auth, the server should immediately send a full state snapshot.

Recommended snapshot contents:

```json
{
  "game_state": "...",
  "enkephalin": 0,
  "phase_event": null,
  "current_shop": null,
  "current_bonus": null,
  "current_random_event": null,
  "inventory": { ... },
  "field": { ... }
}
```

This snapshot must be sufficient for Unity to render the current meta-game UI without needing another request first.

The exact snapshot shape can be server-defined, but it must be stable and explicit.

## 11. Mapping `BehaviorResult` to Wire Responses

The first version can map `BehaviorResult` almost directly.

Examples:

- `BehaviorResult::StartNewGame`
  - `command_result`
  - `result_type = "StartNewGame"`

- `BehaviorResult::RequestPhaseData(event)`
  - `command_result`
  - `result_type = "RequestPhaseData"`
  - `payload = event`

- `BehaviorResult::ShopState { shop }`
  - `command_result`
  - `result_type = "ShopState"`
  - `payload = shop`

- `BehaviorResult::SuppressAbnormality { winner, timeline }`
  - `command_result`
  - `result_type = "SuppressAbnormality"`
  - `payload = { winner, timeline }`

Errors:

- map `GameError` to a wire error envelope
- include request id when possible

## 12. Required `game_server` Code Changes

### 12.1 Add `core` dependency to `game_server`

If not already present, add the correct local crate dependency in:

- `/mnt/f/work/simulator/game_server/Cargo.toml`

The actual crate name must match the workspace crate export from `/mnt/f/work/simulator/core/Cargo.toml`.

This must expose:

- `GameCore`
- `PlayerBehavior`
- `BehaviorResult`
- supporting DTO types

### 12.2 Implement player-game protocol

Files:

- `/mnt/f/work/simulator/game_server/src/game/player_game_actor/messages.rs`
- possibly `/mnt/f/work/simulator/game_server/src/game/player_game_actor/state.rs`

Add:

- client messages
- server messages
- auth message
- command envelope
- snapshot DTO

### 12.3 Implement `PlayerGameActor`

Files:

- `/mnt/f/work/simulator/game_server/src/game/player_game_actor/mod.rs`
- `/mnt/f/work/simulator/game_server/src/game/player_game_actor/handlers.rs`

Add:

- actor state
- `GameCore` ownership
- command execution
- socket write path
- reconnect handling
- snapshot generation

### 12.4 Extend `LoadBalanceActor`

Files:

- `/mnt/f/work/simulator/game_server/src/game/load_balance_actor/messages.rs`
- `/mnt/f/work/simulator/game_server/src/game/load_balance_actor/handlers.rs`

Add:

- `FindPlayer`

### 12.5 Add `/game` route

File:

- `/mnt/f/work/simulator/game_server/src/main.rs`

Add:

- `/game` WebSocket endpoint
- mock auth path
- player actor creation / lookup / registration

## 13. Recommended Implementation Order

Implement in this order.

### Phase 1: Protocol and actor scaffolding

1. add `core` dependency to `game_server`
2. define `/game` protocol messages
3. implement `FindPlayer` in `LoadBalanceActor`
4. flesh out `PlayerGameActor` struct

### Phase 2: Session bootstrap

5. add `/game` route in `main.rs`
6. implement mock auth
7. create or reuse `PlayerGameActor`
8. send `authed`
9. send `state_snapshot`

### Phase 3: Command execution

10. deserialize `PlayerBehavior`
11. call `GameCore::execute()`
12. map `BehaviorResult` to wire `command_result`
13. map `GameError` to wire `error`

### Phase 4: Replay path

14. ensure `SuppressAbnormality` result delivers timeline cleanly
15. verify Unity can consume the replay payload

### Phase 5: Reconnect and lifecycle

16. implement socket rebind
17. add cleanup / deregistration strategy
18. add actor TTL policy if desired

## 14. Testing Plan

### 14.1 Unit tests

- protocol serialization/deserialization
- mock auth acceptance
- `FindPlayer`
- `BehaviorResult` to wire response mapping

### 14.2 Actor tests

- new connection creates actor
- reconnect reuses actor
- authenticated command reaches `GameCore`
- invalid command returns `error`

### 14.3 Integration tests

- `auth -> snapshot -> StartNewGame`
- `RequestPhaseData -> RequestPhaseData result`
- `SelectEvent -> ShopState`
- `PurchaseItem`
- `StartSuppression -> timeline result`

## 15. Known Open Questions

These are not blockers for the first version, but should be kept visible.

- Should `/game` use a dedicated transport session actor or let `PlayerGameActor` itself be the WS actor?
- How much of `BehaviorResult` should be exposed directly versus mapped into dedicated wire DTOs?
- What exact snapshot shape should Unity store as its source of truth?
- How long should a disconnected `PlayerGameActor` stay alive for reconnect?
- When real auth is added, should `/game` auth happen via header, first message, or both?

## 16. Recommended Short-Term Answer To The Open Questions

For a first implementation, the recommended defaults are:

- use `WebSocket + JSON`
- use mock auth inside `game_server`
- trust client-provided `player_id`
- send a full snapshot after auth
- use `command_result` with payloads closely matching `BehaviorResult`
- keep PvP integration out of the first implementation unless needed immediately

## 17. Summary

The important architectural decision is:

`PlayerGameActor owns GameCore`

That keeps the separation clean:

- `core` stays authoritative for rules
- `game_server` stays authoritative for session/networking
- Unity stays focused on rendering and user input

The current codebase already has the right placeholders:

- `PlayerGameActor` exists
- `LoadBalanceActor` exists
- `MatchCoordinator` exists
- `GameCore` already exposes the correct command/result API

What is missing is the actual Unity-facing `/game` path and the per-player actor implementation.
