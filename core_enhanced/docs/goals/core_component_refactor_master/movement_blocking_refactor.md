# 이동/저지 시스템 Refactor Audit

## Scope

- 기준 문서: `docs/refactor_preparation_plan.md`
- Runtime code:
  - `src/game/battle/core/movement/types.rs`
  - `src/game/battle/core/movement/engine.rs`
  - `src/game/battle/core/movement/rapier_backend.rs`
  - `src/game/battle/core/movement/steering.rs`
  - `src/game/battle/core/movement/path.rs`
  - `src/game/battle/core/movement/blocking.rs`
  - `src/game/battle/core/movement/planner.rs`
  - `src/game/battle/core/movement/lifecycle.rs`
  - `src/game/battle/core/mod.rs`
  - `src/game/battle/core/sim.rs`
  - `src/game/battle/core/build.rs`
  - `src/game/battle/core/basic_attack.rs`
  - `src/game/battle/core/commands.rs`
  - `src/game/battle/timeline.rs`
  - `src/game/battle/scenario.rs`
- Live RON/data touchpoints:
  - `../game_resources/data/abnormalities/base.ron`
  - `../game_resources/data/enemies/corroded_employees.ron`
  - `../game_resources/data/map/battlefield_templates.ron`
  - `../game_resources/data/pve/encounters.ron`
  - `src/game/data/abnormality_data.rs`
  - `src/game/data/corroded_employee_data.rs`
- Unity-facing contract:
  - `docs/game_rulebook.md`
  - `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
  - `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
- Tests:
  - `src/game/battle/core/movement/*` module tests
  - `src/game/battle/core/*` movement-adjacent tests
  - `src/game/world/tests/combat.rs`
  - `tests/ron_loading.rs`

## Current Structure

- `src/game/battle/core/movement/types.rs:5` defines world units as the canonical runtime movement coordinate space. Tile coordinates are input/projection/debug data.
- `src/game/battle/core/movement/types.rs:19` defines `DATA_UNITS_PER_WORLD`, the conversion bridge from authored integer data units to runtime world units.
- `src/game/data/abnormality_data.rs:74` defines `MovementDef` with `speed_units_per_ms` and `radius_units`.
- `src/game/battle/core/build.rs:92` copies movement speed into battle stats, converts radius from data units to world units, and builds `UnitBody` with world-space speed.
- `src/game/battle/core/mod.rs:61` stores `active_movement_segments`, `block_state`, `last_continuous_movement_tick_ms`, and the `movement_backend` inside `BattleCore`.
- `src/game/battle/core/sim.rs:1224` schedules `ContinuousMovementTick` at `DEFAULT_MOVEMENT_TICK_MS`; `src/game/battle/core/sim.rs:2828` processes it, runs movement, starts pending attacks, and schedules the next tick.
- `src/game/battle/core/movement/engine.rs:139` canonicalizes movement input order by unit id before resolution.
- `src/game/battle/core/movement/engine.rs:241` implements backend-independent continuous movement resolution and emits `MovementOutput`.
- `src/game/battle/core/movement/engine.rs:533` records each body movement as immutable `MovementSegmentStarted` timeline event and stores an `ActiveMovementSegment` interpolation cache.
- `src/game/battle/core/movement/engine.rs:643` applies movement outputs to `RuntimeUnit.body`, opponent facing, active movement segments, and current target.
- `src/game/battle/core/movement/rapier_backend.rs:461` adapts Rapier to the same `ContinuousMovementResolver` contract. `ContinuousMovementBackend::default` is Rapier (`src/game/battle/core/movement/engine.rs:150`).
- `src/game/battle/core/movement/path.rs:5` converts `EnemyMovementPlan::PathAlongCells` cells into next world-space route targets and route progress.
- `src/game/battle/core/movement/planner.rs:21` builds movement goals from current units, scenario tactical plan, per-unit enemy movement plan, block state, and ranged reposition state.
- `src/game/battle/core/movement/blocking.rs:16` recomputes explicit `BlockRuntimeState` for `FixedDefense` and applies block target preferences.
- `src/game/battle/core/movement/lifecycle.rs:8` records `MovementStopped` timeline events for explicit interruptions such as attack start, cast start, hard CC, and death.
- `src/game/battle/core/commands.rs:484` samples active movement segments when other systems need a unit position at a specific battle time.

Policy docs match this shape. `docs/game_rulebook.md:210` says `BattlefieldRoute.cells` is the authoritative route path, and `:213` says setup snapshot routes, runtime enemy movement plan, movement event/checkpoint must derive from the same cells. `docs/game_rulebook.md:215` says movement/collision are 2D continuous-coordinate based. `/mnt/f/unity projects/ark/docs/unity_core_contract.md:949` says movement blocking is not physics collision; it is explicit core blocking state/target preference. `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md:458` fixes the live movement tick policy at 50ms movement segments.

## Source Of Truth Judgment

- Runtime position source of truth is `RuntimeUnit.body.position` in world units.
- Presentation movement source of truth is append-only `TimelineEvent::MovementSegmentStarted` entries. Checkpoint `world_position` is a reconcile target, not animation timing source.
- `active_movement_segments` is a derived interpolation cache used for in-flight sampling between immutable movement events and current body state. It is not a separate canonical source.
- Route path source of truth is `BattleScenario`/per-unit `EnemyMovementPlan::PathAlongCells`, which comes from combat preview/setup route cells.
- Blocking engagement source of truth is `BlockRuntimeState`, derived from current units, block stats, route progress, and existing engagements. `RuntimeUnit.current_target` is also used by attack selection, so block code writes target preferences into it.
- Ground static collision source is `Battlefield` static obstacles and void tiles projected into `MovementStaticObstacle` input (`src/game/battle/core/movement/engine.rs:627`).
- Rapier state is backend-internal acceleration state. Gameplay source remains `MovementTickInput`, `RuntimeUnit.body`, and `Battlefield`, not Rapier handles.
- Live data source for movement stats is unit combat profile data (`MovementDef`, `mobility_kind`, `block_capacity`, `block_radius_units`, `blockable`, `ranged_reposition_ms`).

## Refactor Candidates

### 1. `BlockRuntimeState`와 `RuntimeUnit.current_target`의 target ownership boundary 명확화

근거:

- `refresh_block_state` recomputes explicit block engagements and preserves existing pairs first (`src/game/battle/core/movement/blocking.rs:16`).
- `replace_block_state` then clears released target preferences and applies current block target preferences (`src/game/battle/core/movement/blocking.rs:110`).
- `apply_block_target_preferences` writes both blocker and enemy `current_target` (`src/game/battle/core/movement/blocking.rs:351`).
- `basic_attack.rs` also reads and writes `current_target` for persisted attack target behavior (`src/game/battle/core/basic_attack.rs:36`, `:255`, `:276`).
- game rulebook says blocking is sustained engagement, not distance-only physics collision (`docs/game_rulebook.md:149`).

판단:

- `BlockRuntimeState` and `current_target` are not pure duplication because `current_target` also serves basic attack persistence and non-block targeting.
- But block engagement can temporarily own target preference, so the boundary is subtle and easy to violate when adding target logic.
- Refactor candidate: introduce small helpers/names that make block-owned target preference explicit, such as `apply_block_engagement_target_preferences`, `clear_released_block_engagement_targets`, or a documented `TargetPreferenceSource`.
- Avoid adding a broad targeting strategy layer now; the problem is ownership clarity, not insufficient abstraction.

필요 검증:

- Existing block engagement persists while distance/order changes.
- Released block pairs clear only block-owned targets and do not erase valid non-block persisted target intent.
- Blocked enemy receives no movement segment while blocker keeps target preference.

### 2. `MovementOutput::MovementStopped`와 `TimelineEvent::MovementStopped`의 semantic split 정리

근거:

- The movement engine emits `MovementOutput::MovementStopped` for dead, movement-locked, no-goal, static-obstacle-blocked, and target-reached/no-goal paths (`src/game/battle/core/movement/engine.rs:260`, `:270`, `:300`, `:334`; `src/game/battle/core/movement/steering.rs:57`).
- `apply_continuous_movement_outputs` currently removes the active segment for `MovementOutput::MovementStopped` but does not convert it into a `TimelineEvent::MovementStopped` (`src/game/battle/core/movement/engine.rs:685`).
- `TimelineEvent::MovementStopped` is recorded by `interrupt_movement` and death/focus/CC paths (`src/game/battle/core/movement/lifecycle.rs:8`, `src/game/battle/core/basic_attack.rs:269`, `src/game/battle/core/commands.rs:347`, `src/game/battle/core/sim.rs:401`, `:2686`).
- Unity contract lists both `MovementSegmentStarted` and `MovementStopped` as events, but movement presentation primarily samples immutable movement segments (`/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md:435`, `:458`).

판단:

- This may be intentional: natural segment completion is represented by segment `ends_at_ms` and the next segment/checkpoint, while `MovementStopped` records interruptive stops.
- However the type names make it easy to assume every engine stop becomes a Unity-facing stop event.
- Refactor candidate: rename internal `MovementOutput::MovementStopped` to `MovementHalted`/`NoMovement`/`ContinuousMovementStopped` or document that only interruptive stops become timeline stop events.
- If Unity needs natural stop events for VFX/state machines, adding timeline stop events is a Unity-facing event contract change.

정책 영향:

- Natural target reached/static obstacle/no-goal stops should or should not emit `TimelineEvent::MovementStopped`: `사용자와 정책 논의 필요`.

필요 검증:

- Existing tests for immutable 50ms movement segments remain green.
- Add focused test that natural route segment end does not emit `MovementStopped`, or the opposite if policy changes.

### 3. Direct/Rapier backend role 명확화

근거:

- `ContinuousMovementBackend::default` is Rapier (`src/game/battle/core/movement/engine.rs:150`).
- Direct backend exists and is selectable in tests (`src/game/battle/core/movement/engine.rs:522`).
- Engine tests compare Direct and Rapier ordering, ground static obstacle stop policy, airborne obstacle policy, and board clamp policy (`src/game/battle/core/movement/engine.rs:833`, `:863`, `:892`, `:912`).
- Rapier backend keeps handles/internal colliders but runtime gameplay body state remains in `RuntimeUnit` (`src/game/battle/core/movement/rapier_backend.rs:19`).

판단:

- This is not a legacy dual implementation to delete. Direct is a useful deterministic oracle/fallback for backend equivalence and unit tests.
- Still, the public-looking `use_direct_continuous_movement_backend` and `use_rapier_continuous_movement_backend` names could imply runtime policy toggles. The direct switch is `#[cfg(test)]`, while rapier switch is public.
- Refactor candidate: document Direct as test oracle, and keep backend equivalence tests close to the shared resolver contract.
- Do not introduce a larger physics strategy abstraction unless a second production backend appears.

필요 검증:

- Backend equivalence tests continue covering ordering, obstacle response, airborne behavior, board clamp, and body state application.

### 4. Movement policy constants ownership

근거:

- `DEFAULT_MOVEMENT_TICK_MS` is hard-coded at 50ms (`src/game/battle/core/movement/types.rs:12`).
- `DEFAULT_UNIT_RADIUS` is a hard-coded fallback and comment says final tuning should live in unit data (`src/game/battle/core/movement/types.rs:25`).
- Actual unit radius and speed are already live data via `MovementDef` (`src/game/data/abnormality_data.rs:74`, `src/game/battle/core/build.rs:98`).
- Unity transport contract also documents 50ms as current live stream policy (`/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md:458`).

판단:

- `DEFAULT_UNIT_RADIUS` appears to be a fallback for test/default `UnitBody`; live units use profile radius, so it is not a live balance source duplication.
- `DEFAULT_MOVEMENT_TICK_MS` is both simulation cadence and Unity stream policy. Moving it to RON would be a gameplay/transport policy change, not a small refactor.
- Refactor candidate: keep code constant, but ensure contract/tests reference it indirectly where possible and do not fork another constant in server/Unity.

정책 영향:

- Changing movement tick cadence or making it data-driven affects Unity presentation and battle timing: `사용자와 정책 논의 필요`.

필요 검증:

- Movement segment tests assert adjacent 50ms segments.
- Unity/server contract tests, if changed, should validate the same cadence.

### 5. Static obstacles and void tiles are already composed through one movement collision input

근거:

- `continuous_static_obstacles` maps both battlefield static obstacles and void tiles into `MovementStaticObstacle` (`src/game/battle/core/movement/engine.rs:627`).
- Ground movement applies static obstacles; airborne movement ignores them through `MovementTerrainPolicy::applies_static_obstacles` (`src/game/battle/core/movement/engine.rs:24`, `:402`; `src/game/battle/core/movement/rapier_backend.rs:480`).
- Game rulebook says static obstacle and non-rectangular void tiles are projected into movement collision input (`docs/game_rulebook.md:215`).

판단:

- This is a good existing composition: no separate "void tile death/stop" rule is needed for movement.
- Not a refactor target. It is an example of the desired source-of-truth reduction style.
- Future movement changes should not reintroduce special-case void tile movement logic while obstacle projection covers the same gameplay meaning.

필요 검증:

- Keep tests that ground static obstacles stop movement and airborne units ignore static obstacles.
- Add/keep live non-rectangular battlefield movement test if route/void interactions become more complex.

### 6. Spawn collision fallback belongs to scenario spawn, not movement blocking

근거:

- `place_scenario_spawn` allows opponent spawn to fall back to nearest open spawn position when preferred spawn cell is occupied (`src/game/battle/core/build.rs:187`).
- Movement blocking is explicit engagement and not physics collision (`/mnt/f/unity projects/ark/docs/unity_core_contract.md:949`).
- Movement resolver allows moving units to overlap each other in continuous space; block state handles engagement.

판단:

- Spawn fallback is not a movement/blocking source-of-truth duplicate. It is battle setup collision resolution for tile occupancy.
- However it can affect route progress and initial block priority because spawned world position changes.
- If encounter/wave authoring wants deterministic multi-spawn placement, this fallback should be documented or replaced by load-time validation that enough spawn cells exist.

정책 영향:

- Removing opponent spawn fallback in favor of hard validation could alter live encounter behavior: `사용자와 정책 논의 필요`.

필요 검증:

- Multi-enemy wave spawn positions remain deterministic when preferred spawn is occupied.
- If policy changes, invalid wave capacity fails before battle start.

## Existing Composition Simplification Candidates

- Void tiles and static obstacles already share `MovementStaticObstacle` collision projection. Keep this composition.
- Route progress, block priority, and endpoint targeting all read the same `PathAlongCells` source. Do not duplicate route progress in another per-unit field unless profiling proves the need.
- Airborne behavior is already expressed as `MovementTerrainPolicy::Airborne` plus `mobility_kind`/`blockable` target policy. Do not add separate "airborne obstacle bypass" branches outside the movement input policy.
- Movement presentation already uses immutable `MovementSegmentStarted` plus checkpoint reconciliation. Do not push full `state_snapshot` every tick for movement.

## Legacy, Fallback, Compatibility Removal Candidates

- Direct movement backend is not legacy; keep it as a test oracle unless production no longer benefits from backend equivalence checks.
- `TILE_UNITS_PER_TILE` and `HALF_TILE_UNITS` in `src/game/battle/core/movement/mod.rs:3` look like older fixed tile-unit constants. Audit usage before deleting; current runtime movement uses `WORLD_UNITS_PER_TILE` and `DATA_UNITS_PER_WORLD`.
- Opponent spawn nearest-open fallback should be revisited with wave/spawn-zone authoring policy. It may be valid gameplay robustness, but it can also hide insufficient spawn authoring.
- Natural movement stop not becoming `TimelineEvent::MovementStopped` is likely a contract distinction, not a bug, but should be named/documented to avoid accidental compatibility assumptions.

## Deferred Or Not Doing

- Do not merge movement with tile range/pathfinding. Movement is continuous world-space; range preview and deploy selection are tile-based contracts with different source of truth.
- Do not replace explicit block state with Rapier collisions. Current contract says blocking is core engagement state, not physics overlap.
- Do not move all movement constants to live RON in this audit. Cadence, radius fallback, and stream policy have different lifecycles.
- Do not refactor basic attack target selection here except where block target ownership crosses movement. Full target usefulness and attack judgement belong to components 8 and 9.
- Do not change movement event DTO shape in this component audit. Event shape belongs to component 15 unless a movement refactor requires it.

## Tests And Verification To Add When Refactoring

문서 감사 단계에서는 코드를 바꾸지 않았으므로 테스트를 실행하지 않았다. 실제 리팩토링 시 우선순위 검증은 아래와 같다.

- `cargo test -p game_core battle::core::movement`
- `cargo test -p game_core battle::core`
- `cargo test -p game_core world::tests::combat::live_battle_update_exposes_adjacent_immutable_movement_segments`
- `cargo test -p game_core world::tests::combat::generated_defense_route_setup_snapshot_matches_runtime_route_cells`
- `cargo test -p game_core --test ron_loading`

추가하면 좋은 focused test:

- block-owned `current_target` preference가 release될 때 non-block persisted target을 의도치 않게 지우지 않는지 검증.
- natural movement stop policy를 확정하는 test: no timeline stop event vs explicit timeline stop event.
- opponent spawn fallback이 route progress/block priority에 미치는 deterministic ordering test.
- `TILE_UNITS_PER_TILE`/`HALF_TILE_UNITS`가 live path에서 미사용이면 제거 전 `rg` evidence와 focused compile check.

## Policy Discussion Required

- Natural target-reached/static-obstacle/no-goal continuous movement stop을 Unity-facing `MovementStopped` event로 보낼지 여부: `사용자와 정책 논의 필요`.
- `DEFAULT_MOVEMENT_TICK_MS` 50ms cadence를 계속 code/contract constant로 둘지 data-driven policy로 옮길지 여부: `사용자와 정책 논의 필요`.
- Opponent spawn nearest-open fallback을 유지할지, wave/spawn-zone capacity validation failure로 바꿀지 여부: `사용자와 정책 논의 필요`.
- Future forced movement/knockback/teleport가 block engagement를 어떻게 해제해야 하는지 세부 정책: `사용자와 정책 논의 필요`.
