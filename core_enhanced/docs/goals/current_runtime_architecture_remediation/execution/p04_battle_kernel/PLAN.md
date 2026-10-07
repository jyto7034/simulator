# P04 Battle Kernel Execution Plan

상태: `GEX-00 passed / GEX-10 exit criteria met; target freeze pending`

## Current Execution Identity

- Handoff / root target: `HA-CRAR-P2G60-002` / full `GS-CRAR-P2G50-002` goal-run
- Selected member: `(GS-CRAR-P2G50-002, GOAL-CRAR-C2-P04-BATTLE-KERNEL)`
- Phase / execution node: `PHASE-CRAR-C2-04-BATTLE-KERNEL` / `WP-CRAR-C2-05`
- Current pass: GEX-10 implementation and direct validation complete; successor-currentness evidence is current and
  the exact Audit Target is being frozen before a separate GEX-20 audit.
- Goal Quality Contract: `2`
- Authorization: Core-only selected-Goal mutation and test execution. `game_server`, Unity, sibling resources,
  transports and external consumers are excluded.

## Exact Basis and Dependencies

- Freeze / Plan / Receipt / Goal Set: `FPF-CRAR-P2G30-002` / `PP-CRAR-P2G40-002` /
  `PVR-CRAR-P2G40-002` / `GS-CRAR-P2G50-002`.
- P01: `Complete / High`, current on `ATR-CRAR-C2-P01-GEX20-003`.
- P02: `Complete / High`, current on `ATR-CRAR-C2-P02-GEX20-003`.
- P03: `Complete / High` on frozen `ATR-CRAR-C2-P03-GEX10-001`; its executor/lifecycle surface is current at P04
  intake.
- Applicable durable seam: `IVA-CRAR-C2-05`; exact policy decision `DR-CRAR-006-R1` selects scoped mutation and
  pre-mutation rejection, not normalization or a non-finite total order.

## Objective and Boundary

- Close the Core battle kernel behind a default-release-inaccessible module and scoped Core command/query/event/read
  ports. The application executor remains the only production command producer.
- Remove production-public unit-field, kernel-field and raw event-injection mutation authority without rewriting
  battle algorithms.
- Make float-bearing world position, speed/radius and time conversion inputs constructible only through fallible finite
  validation at authority boundaries. NaN and positive/negative infinity must return typed errors before mutation;
  clamp/default/normalization is forbidden.
- Preserve finite boss, movement, skill, same-bucket event ordering and deterministic results exactly.
- Keep active history/result lifetime, checkpoint format, transport-neutral public projection and performance PASS in
  P05/P06/P07/P08 respectively.

## GEX-00 Current Inventory

- `game::battle::core` is production-public. `BattleCore` exposes 11 public mutable authority fields, including units,
  battlefield, event log/sequences, seed and recording stacks.
- `RuntimeUnit` exposes 36 public fields plus a direct world-position setter. The core subtree contains 26 syntactically
  public `&mut self` methods across kernel, simulation, spawn and value helpers.
- `BattleCore::enqueue_event` accepts public `BattleEvent` directly. Float-bearing impact/area events can contain raw
  `WorldVec2`; `BattleEvent` manually implements `Eq` for `Ord`/`BinaryHeap` while only documenting finiteness.
- `WorldVec2` exposes raw public `x/y`, an infallible raw-float constructor and derived deserialization. `UnitBody`
  exposes raw position/radius/speed and infallible raw-float constructors. No uniform typed non-finite error exists.
- The actual production `GameCore` active battle is private after P03 and validated live data is opaque after P02, but
  standalone release callers can still construct and mutate the exported kernel independently. Runtime public-caller
  frequency is not a completion claim.
- Current integration skill/runtime oracles intentionally use direct kernel fixtures. They are test capability, not a
  production authority contract, and must remain runnable through the established debug/test-fixtures gate while the
  default release surface closes.
- Existing kernel algorithms and tests are strong: stable event comparator/permutation tests, boss task/route tests,
  continuous movement characterization, skill delivery/targeting and full live-RON world paths already exist.

## Approved GEX-10 Design

1. Make the `battle::core` module crate-private in default release and public only under the existing
   test/debug/test-fixtures capability gate. Default release therefore cannot name `BattleCore`, `RuntimeUnit`, kernel
   movement types, mutation helpers or raw event injection; Core world/event owners retain crate-scoped access.
2. Keep the kernel's current internal algorithms in place. Do not introduce a second battle state or adapter writer;
   production setup and live commands continue through `GameCore` -> P03 application draft -> `ActiveBattleSession` ->
   kernel command methods.
3. Turn `WorldVec2` into a finite boundary value: hide raw coordinates from production callers, add fallible raw-float
   construction with a typed component error, reject non-finite deserialization, and keep only crate-private internal
   construction for already validated/calculated values.
4. Add fallible finite radius/speed and battle-time scalar constructors and use them at scoped body/movement boundary
   preparation. Integer-authored conversions remain semantically identical but cross a checked finite gate before
   kernel mutation.
5. Add exhaustive NaN/positive-infinity/negative-infinity tests for position components, radius, speed and time. Each
   attempted scoped command/setup must compare the complete kernel authority before/after first rejection and retry,
   including units, queue/order, event log/sequences, battlefield and RNG/recording state.
6. Add valid finite-construction/serialization round-trip and same-bucket ordering tests. Reuse current boss,
   continuous-movement, skill-runtime and live `GameCore` corpus as the finite differential oracle.
7. Add standalone default-release probes: a valid probe must exercise actual validated publication -> `GameCore` ->
   battle route; an invalid probe must fail to import/name/mutate the kernel, unit/event queue and raw finite internals.
8. Reconcile `docs/skill_target_contract.md` and the canonical runtime contract only to the implemented private-kernel
   boundary. Do not claim external physics, wire, consumer or cross-platform bitwise replay compatibility.

## Completion Contract

- default-release public unit/kernel field mutation and raw `BattleEvent` injection bypass: 0
- application-owned Core battle route is the sole production command producer; duplicate kernel mutation facade: 0
- every NaN/positive-infinity/negative-infinity boundary case returns a typed error before units, event queue/order,
  log/revision, battlefield, seed or recording state changes; retry is identical
- valid finite construction and serialization cannot manufacture a non-finite `WorldVec2`/speed/radius/time value
- finite boss/movement/skill outcomes and event ordering remain identical and deterministic
- all ten validation dimensions have direct evidence or the approved checkpoint/history-lifetime N/A rationale
- default-release negative API probe, live producer-consumer probe, full Core oracle and canonical documents agree
- external compatibility and performance PASS claims: 0

## Stop Conditions

- finite rejection would require normalization/defaulting or a new non-finite total-order policy
- privacy cutover cannot preserve the approved finite boss/movement/skill behavior
- a public kernel bypass is required by a production Core caller outside the P03 application authority
- policy meaning, Goal/Phase/node mapping, history/checkpoint/public-projection ownership or external scope must change
- predecessor evidence becomes stale without a direct approved successor-currentness audit
- dirty overlap cannot be isolated safely

## Pre-edit Identities

- `Cargo.toml` `ec456bbe068e8009575d6168986d2bddfa4e14a34f76bbaaa866e843f2af2025`
- `src/game/battle/mod.rs` `6100a99907befaa0c7556f8437a07400c07c8d17af6d8088deaed24ed5cb79a1`
- `src/game/battle/core/mod.rs` `ee35e213dc4ed05f428acb4ba410e61282d85bedd60a9a27b073028a817410ca`
- `src/game/battle/core/types.rs` `62c225ff67f74e9789447e375ae5eae4cd368ec618739d86ccf53f5028e88264`
- `src/game/battle/core/movement/types.rs` `5841163963cce63729b7d496b02b88cf016045eb4795de7f3d4d0f7170f2a5e2`
- `src/game/battle/core/sim.rs` `0d3fb7af215d12ab34612e4e15f3d33faa03627b4101233e05f41153775b23ed`
- `src/game/battle/core/build.rs` `cf8130d1bf10ccb996b1aab0354b606d10232371745e987dbfa55738d3178470`
- `src/game/battle/enums.rs` `81ea053b37dbf964859f86d1a9d949dcabe00e2608ee8775a33b6be69ffe76b2`
- `src/game/world/state.rs` `93dd4dffd14dfcb9e9fd8872b4f63e59aff7c5b5979f3b3e6028adc8398daded`
- `src/game/behavior.rs` `66f037715fd574277b04e8842f33cbb89d9ce3cd62f1c34be59f1d9915c2bd40`
- `docs/skill_target_contract.md` `a35a20243be2ea64c35931292d11f26bda3cf3980e816370c7ee88b0e220789c`

All are pre-existing dirty/mixed paths except this new execution directory. P04 owns only exact kernel privacy,
finite-boundary, validation and documentation hunks; all unrelated bytes remain user-owned.

## GEX-10 Implemented Boundary

- Default release exposes `battle::core` only crate-wide; public visibility exists solely under
  `test`, `debug_assertions` or explicit `test-fixtures`. The actual `GameCore` owner remains able to construct and
  command its private active battle, while a standalone release caller cannot name the kernel or inject a raw event.
- `WorldVec2` now hides coordinates, validates both components through typed fallible construction and custom
  deserialization, and exposes read-only accessors. Checked finite radius, speed and battle-time values reject NaN and
  both infinities; negative physical radius/speed/time are also rejected.
- Scenario group construction validates radius/speed before spawn ordering, placement or unit-authority mutation.
  Millisecond simulation time uses a checked finite conversion; no clamp, default, normalization or non-finite total
  order was introduced.
- The rejection matrix covers position x/y, radius, speed and time for NaN/positive infinity/negative infinity. Every
  first rejection and retry compares the complete kernel authority snapshot. Finite signed-zero/ordinary values
  round-trip, and direct deserializer injection of non-finite values reaches and fails the typed finite gate.
- Test/debug-only inspection helpers were capability-gated precisely enough to keep default release warning-free; no
  lint suppression or production test-name/fixture-value branch was added.
- Canonical Core and skill-target contracts now describe the implemented private finite kernel. History retention,
  checkpoint format, public projection and performance policies remain untouched for P05-P08.

## GEX-10 Exit Evidence

- `cargo fmt --all -- --check`: exit `0`.
- `cargo check -p game_core --release`: exit `0`, warnings `0`.
- Focused finite rejection/round-trip, heap permutation, boss-order, movement-repeatability and skill runtime suites all
  pass.
- Valid default-release live probe: actual embedded validated data -> `GameCore` -> live combat entry, exit `0`.
- Invalid default-release kernel probe: expected exit `101` with `E0603` for the private module.
- Fresh full Core oracle: library `1,087` plus integrations `51`, total `1,138`; failed/ignored `0`.
- Fresh predecessor review: P01 `ATR-CRAR-C2-P01-GEX20-004`, P02
  `ATR-CRAR-C2-P02-GEX20-004`, P03 `ATR-CRAR-C2-P03-GEX20-002`; all `Complete / High` and current.
