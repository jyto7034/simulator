# Testability And Behavior-Oracle Map

- Goal: `current_runtime_architecture_deep_review`
- Status: `complete`
- Baseline: `BASELINE-20260822-064427-KST`
- Primary evidence: E-003, E-009~E-012, E-018, E-021

## Assessment

Current testability is a major preservation asset, but not a complete architecture safety system. Domain algorithms and the
public `GameCore` façade can be tested without server, network or filesystem. The 1,131-test baseline provides strong
characterization, live-data and rejection oracles. The important gap is uniformity: each new mutating use case must still invent
or remember its own authority fingerprint, late-failure setup and rollback assertion.

Provisional A06 verdict: `PARTIAL`. The suite is strong enough to support incremental replacement behind current ports, but the
application transaction layer and battle/data dependency boundary are not isolated enough for cheap universal conformance.

## Test Layers And Seams

| Layer | Current construction/port | Oracle strength | Limitation | Rewrite reuse |
| --- | --- | --- | --- | --- |
| Pure/bounded domain | movement engine/planner, Boss Runtime, skill compiler/runtime, Monolith, map graph and economy planners | deterministic values, events, typed errors and invariant tests | some battle tests still require a broad `GameDataBase` because the kernel stores it | reuse algorithms and focused tests; introduce battle-scoped compiled context |
| Data validation/compilation | private live-RON parse/validate candidate and `load_live_embedded()` | exact inventory, strict parse/reference gates, parsed-to-published equality, generated battle feasibility | global bundle fixtures are expensive and schema changes touch a broad catalog | preserve production loader and validators as acceptance gate |
| Application/world | public/near-root `GameCore::execute*`, snapshots, rollback captures | observable `BehaviorResult`, root authority fingerprints, no-mutation rejection, retry behavior | broad fixtures and private helper access; no one mutation protocol covers every handler | run old/new use case behind the same façade and compare canonical result/state |
| Persistence | strict V4 export/continue and explicit checkpoint load | exact field set, 19 invalid graph mutations, restore-recapture semantic equality | strict current version has no migration matrix; one full future-feature sequence is not present | preserve V4 reader as a bounded import oracle during migration |
| Integration/live content | `tests/ron_loading.rs`, skill catalog/runtime tests | real embedded content through production loader | asserts current content/shape, not all target policies | retain as release gate after schema adapter/compiler changes |
| Performance | no repository benchmark target; E-020 `/tmp` harness | current local distributions and scaling mechanism | no stable CI runner, allocation counters or event-heavy boss fixture | convert accepted workloads to a dedicated benchmark/regression suite |

## Observable Oracles Worth Preserving

- Public command result plus final run snapshot, not private struct equality, is the primary old/new equivalence representation.
- Battle equivalence additionally requires ordered event delta, revision, authoritative checkpoint and terminal result.
- Save equivalence uses strict decoded semantic state and subsequent command results; raw JSON byte order is only used where the
  current contract deliberately makes it exact.
- Data equivalence uses validated/compiled publication and generated setup feasibility, not raw RON deserialization alone.
- Failure equivalence includes both returned typed error and unchanged relevant authority. Monotonic profile/run-peak exceptions
  must be named explicitly rather than hidden in a generic whole-root equality assertion.

## Failure-Control Inventory

| Concern | Existing control | Coverage | Missing reusable seam |
| --- | --- | --- | --- |
| time | explicit battle elapsed/server-tick inputs; pause tests | strong for local simulation and pause | no host scheduling/backpressure model in Core |
| RNG | seeded run/battle/data generation and deterministic tests | strong | no recorded command-tape runner spanning every subsystem |
| UUID | namespaced, checkpointed `UuidManager` | strong and independent streams | namespaces are manually declared; new call sites need review |
| physics | deterministic ordering before/after movement and focused Rapier tests | useful on current runner | cross-platform bitwise equivalence not certified |
| late failure | bespoke invalid cursor, revision overflow, reconcile and reward failure fixtures | strong on selected routes | no injectable commit/projection fault port and no all-command matrix |
| durable I/O | Core rollback token and export port | Core behavior testable | host filesystem failures are outside scope and not simulated here |

## Negative Space

- `Cargo.toml` has no property-testing or Criterion/dev benchmark dependency, and the code search found no
  `proptest`, `quickcheck` or `criterion` use.
- There is no generated inventory saying which `PlayerBehavior` is Query, RunMutation, BattleMutation or ProfileMutation and
  which transaction conformance suite it must pass.
- Direct skill/focus/playback mutations and staged tactical mutations do not share one test adapter, matching
  ARCH-REVIEW-004/005.
- World fixtures can manufacture private states that production constructors would not expose. This is useful for failure
  injection, but each such fixture must identify the production validation it intentionally bypasses.
- The suite freezes several internal V4 shapes. Those are useful current import oracles, not proof that target internal Rust
  layout or all future save versions should remain identical.

## Recommended Verification Architecture

1. Define a typed command classification table generated or exhaustively matched by the compiler.
2. Give every mutation class one harness: capture canonical authority, inject precommit/projection failure, execute, assert
   typed error plus unchanged authority, then assert retry equivalence.
3. Add command-tape differential tests that run current and replacement application ports against the same seed/data and compare
   canonical snapshots/results/events.
4. Keep leaf subsystem tests close to the kernel, but replace full `GameDataBase` fixture dependencies with compiled scoped
   contexts where possible.
5. Promote E-020 workloads into a stable release benchmark only after runner/noise policy is chosen.

This path makes a staged application/boundary rewrite safer than a big-bang rewrite: existing oracles remain executable while
new ports are introduced one vertical slice at a time.
