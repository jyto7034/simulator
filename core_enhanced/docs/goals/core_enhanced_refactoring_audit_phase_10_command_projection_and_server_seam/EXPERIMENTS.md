# Core Enhanced Refactoring Audit Phase 10 — Experiments

Status: complete

## 2026-08-07 — Activation gate

### Evidence and result

- Parent Master, Phase 0 boundary and Phase 9 completion audit were rechecked.
- Direct predecessor Phase 9 is `Complete / High`; the reserved Phase 10 path did not exist.
- Phase 1~9 domain authority and Phase 9 projection/public-consumer handoffs are fixed inputs.
- No other active child exists. Phase 10 is investigation-only and Core/server production mutation remains prohibited.
- Exact start commit remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.

### Tests

No runner was executed during activation.

### Decision

Phase 10 may perform read-only Core/direct-server investigation and write only audit artifacts. Final consolidation
remains pending.

## 2026-08-07 — Scoped production baseline

### Exact categories and initial hashes

| Category | Initial SHA-256 |
| --- | --- |
| Core behavior DTO (1 file) | `a8e4256c9b53ac64511cdb3af83c2df2e9d8008a541aac9807aed573c1ae9b40` |
| world host/projection (6 files) | `c1fd628986c6b7485c318dfa6734413cf8bbf01289195e1bf8546ba879e6fd89` |
| admin command/fixture (6 files) | `d3d2af722ac32e0b0fd7a8685eb7658e907bd89b5520440b51eba7448aefd93b` |
| Core fixture binaries (5 files) | `1efd1eb31076954990c0ed899af40e49846626ddf32c73e684665d26e9d3a5f8` |
| Core package/tooling (5 files) | `8432fcbaa2943b3b57dfb2e3f5cfd652721073d6868b0631ee84007fb22184b0` |
| focused Core tests (3 files) | `dbd6bd9a018520dc103a8f1fdeed0e6ce1df2b9f6f7620df796077c1a2be4e1c` |
| server direct bridge (8 files) | `29752322a7fb06e12e1f9f41decefbf49a6fd3ac1be76fa8858f53a81ba1542e` |
| server fixture examples (3 files) | `da8e1f1ce842643b148ab3c95252eb11edf9001ea1d70da5e2ce41f2315e0b02` |
| bridge/canonical docs (5 files) | `369676382e250871cd0a06990f3de518e2d5c0f31b67bc133a9a426e14b15b41` |
| scoped Git status text | `730b8acf5fd0475217400d6aee8cf45ab72674834248ce954b9164d1c5c4c58b` |

World host/projection is `world.rs` plus `world/{action_capability,battle_capability,combat_trace,snapshot,state}.rs`.
Admin includes `world/admin.rs` and its five direct files. Core package/tooling is `.cargo/config.toml`, Core CI,
`Cargo.toml`, `src/{lib,main}.rs`. Focused tests are world combat/snapshot tests and the disconnected-looking
`tests/test_actor/mod.rs`, whose reachability is not yet assumed.

Server direct bridge is its `Cargo.toml`, two load-balance actor files and five player-game actor files. Server fixture
examples are the action-capability, battle-revision and employee-profile generators. Canonical/supporting documents
are Core docs README/runtime/rulebook and the two server bridge/protocol documents.

### Dirty-state result

Fifteen scoped Core/server/test/canonical paths were already modified, including behavior/world projection,
capability/snapshot state, Core world tests, server player actor mapping, battle-revision fixture and protocol docs.
These are user-owned baseline changes.

### Tests

No runner was executed for the baseline.

## 2026-08-07 — Static command, projection and server-seam trace

### Command ownership

- `GameCore::execute_with_transport_context` validates the supplied battle UUID/revision before mutation, applies
  the same common/battle capability evaluators used by projection, dispatches to the authoritative domain handler,
  and attaches source-command/visual-novel transport context to the result.
- `ActionKind`, `ActionKind::{scope,requires_battle_authority}`, `PlayerBehavior::kind`, the projected action lists,
  common/battle ownership matches, Core dispatch and the server recovery-command intercept each repeat part of the
  command registry. Exhaustive matches catch new enum variants, but independently maintained arrays and owner lists
  can still omit or misclassify a variant.
- `ContinueRun`, `ResetActiveRun`, `RetryEncounterCompendium` and `ResetEncounterCompendium` are Core wire enum
  variants and `ActionKind`s, but Core dispatch deliberately returns `InvalidAction`; the server intercepts them and
  adds their host-owned capabilities to a serialized Core snapshot. They are host recovery commands carried in the
  Core player-command vocabulary, not Core domain mutations.
- For ordinary player commands the server captures an active-run rollback, executes Core, persists the exported
  checkpoint when it changed, and restores only the active-run aggregate if persistence fails. Encounter compendium
  persistence is intentionally a separate transaction. Stale/missing battle authority returns an error plus current
  resync without executing the mutation.

### Result, error and publication shape

- Public `BehaviorResult` is converted into `BehaviorCommandResultContract`, eleven domain result enums and a second
  manual `result_type`/`payload_value` mapping. The same fields and variant identity are therefore authored in the
  result enum, conversion match and JSON projection match. The direct server consumes the Core contract and is thin
  for normal command payloads, but the duplication now lives inside Core.
- `RosterCommandResult` has no construction path. Public `BehaviorResult` shop/reward/result helper methods have no
  workspace production consumer and are asserted only by `behavior_result_helpers_match_variants`.
- `RunCommandResult::RunComplete { map }` serializes the `map` value directly while the semantic variant contains a
  named `map` field. No direct server exact-shape assertion covers this case. This is a compatibility-policy signal,
  not an authorized correction: a result-authority refactor must first freeze the current direct-map wire shape or
  explicitly approve a migration.
- Core `LiveBattleUpdateDto` and `LiveBattleSetupSnapshotDto` already own tagged wire DTOs. The server message enum
  repeats their fields and constructors destructure/copy them. Server tests pin the current top-level JSON shape, so
  reuse/flattening can be behavior-preserving only with exact serialization parity.
- `GameError` is exhaustively mapped to stable codes in two Core snapshot helpers and again in the server adapter.
  Codes currently agree; messages are legitimately context-specific. Existing server tests cover only
  `StaticObstacleBlocked` and two `ActionUnavailable` cases, not the full variant/code table.

### Snapshot, capability and ordering projection

- Core `RunSnapshotDto` owns typed `action_capabilities`, and the server's generic `PlayerStateSnapshotDto` flattens
  the Core snapshot. `build_state_snapshot`, however, serializes to `Value`, indexes
  `value["action_capabilities"]`, appends host recovery capabilities, and clears/rebuilds the list for fatal host
  recovery. This is a typed Core projection followed by an untyped host overlay.
- Outside battle, root snapshot capabilities are authoritative. During battle that root list is empty and the
  revisioned `battle_update.action_capabilities` list is authoritative, matching the durable server protocol.
- Accepted battle mutation ordering is response (`CommandAccepted`) then setup/update side messages; there is no
  root snapshot. Stale authority sends error then current resync. Tick completion publishes battle update then root
  snapshot. Reconnect publishes root snapshot then setup/update. Existing actor/session tests meaningfully observe
  these order and no-extra-snapshot invariants.
- The server forwards range preview DTOs without inspecting their cells. No direct server source consumes the Phase 9
  ghost `range_units`; only typed `range_previews`/preview request results are present. Phase 9 C04's per-step-versus-
  union UI policy and C07's external public-consumer decision remain Unity/external gates, not Phase 10 conclusions.

### Admin, fixture and package reachability

- `AdminCommand` is a separate top-level server request and Core admin fixtures/grants have meaningful atomicity,
  live-projection and invalid-input tests. The server admin path does not share ordinary active-run persistence,
  rollback or battle-side publication. Since admin is environment/token gated and its durability semantics are not
  canonical, this is a policy signal rather than a confirmed behavior-preserving consolidation.
- Cargo metadata reports Core's library, an empty auto-discovered `game_core` binary, five fixture binaries and five
  integration-test targets. `tests/test_actor/mod.rs` is not a Cargo target and has no module edge; it is an empty,
  disconnected file.
- No `build.rs` exists, but `globwalk`, `serde`, `serde_json` and `serde_yaml` remain build-dependencies. Static source
  inventory also finds no Core source/test use for `rand_core`, `base64`, `byteorder`, `rayon`, `clap`, `actix*`,
  `async-tungstenite`, `reqwest`, `futures*`, `argon2`, `dotenv`, `gnuplot`, `criterion`, `url`, `tokio*`,
  `console-subscriber`, `lazy_static`, `structopt`, `hex`, `parking_lot`, `paste`, `cargo-llvm-cov` or
  `serial_test`. `ctor` is used only by an empty startup hook; Core logging support uses the tracing dependencies but
  has no workspace consumer.
- `core_enhanced/.github/workflows/main.yml` is below the workspace member, not repository-root
  `.github/workflows`, so it is not active GitHub Actions configuration for this repository. `.cargo/config.toml`
  applies `tokio_unstable` despite no Core tokio source use. Standalone packaging or external automation was not
  inspected, so package cleanup needs an external-use gate. GitHub's current root discovery rule was checked against
  [official workflow documentation](https://docs.github.com/en/actions/concepts/workflows-and-actions/workflows).
- Historical Goal evidence ties the three server fixture examples and at least the Core event fixture to exact Unity
  fixture comparisons. Current Unity is out of scope, so these generators are `retain / external consumer
  unverified`, not dead-code candidates. The remaining Core fixture binaries likewise require a current external
  consumer inventory before removal.

### Static test gap recorded before runner

- Retain execution/capability/revision/rollback/order tests that observe state, no-mutation, exact transport kind or
  sequence; they protect distinct boundaries rather than merely returning `Ok`.
- Strengthen the result family with an all-variant exact wire table, especially `RunComplete`, and strengthen errors
  with a complete stable-code table.
- Merge/remove the helper-only result test together with the unused public helpers after an external API inventory;
  do not remove it alone while the API remains public.
- Remove the empty disconnected `tests/test_actor/mod.rs`; it executes zero tests and protects no contract.
- Retain admin atomicity/live-fixture tests pending an admin durability policy. Retain fixture generators as
  provenance artifacts, but do not count their compilation or zero-test execution as behavior coverage.

### Tests

No runner was executed during this static pass. The gaps above were recorded first, as required by the Phase plan.

## 2026-08-07 — Focused validation after static gap inventory

### Commands and actual results

| Command | Valid executions | Result |
| --- | ---: | --- |
| `cargo test -p game_core --lib game::behavior::tests` | 8 | 8 passed, 0 failed, 0 ignored; 854 filtered |
| `cargo test -p game_core --lib game::world::admin::tests` | 16 | 16 passed, 0 failed, 0 ignored; 846 filtered |
| `cargo test -p game_server --lib` | 56 | 56 passed, 0 failed, 0 ignored; 0 filtered |
| `cargo test -p game_core --lib game_core_projects_context_capabilities_and_revalidates_on_state_transition` | 1 | passed; 861 filtered |
| `cargo test -p game_core --lib pending_deployment_range_preview_returns_all_facings_without_mutating_battle` | 1 | passed; 861 filtered |
| `cargo test -p game_core --lib supplied_stale_battle_authority_rejects_before_playback_mutation` | 1 | passed; 861 filtered |
| `cargo test -p game_core --lib battle_revision_overflow_rejects_mutation_without_changing_playback` | 1 | passed; 861 filtered |
| `cargo test -p game_core --lib blocking_visual_novel_execution_preflight_rejects_every_omitted_battle_action_without_mutation` | 1 | passed; 861 filtered |
| `cargo test -p game_core --lib active_run_transaction_rollback_preserves_profile_and_peak_but_restores_floor_anchor` | 1 | passed; 861 filtered |

Total valid invocations: 9. Total test executions: 86. Unique test executions: 86. Failures/ignored: 0/0. The
filtered counts are not treated as executions. Package-cache/build-directory lock waits came from concurrent read-only
test invocations and did not change the results.

### Interpretation

- The runner confirms that the current result mapping, admin fixture behavior, direct server bridge, capability
  transition, preview no-mutation, authority rejection, revision overflow, VN preflight and rollback contracts agree
  with the static trace at this working-tree baseline.
- Green results do not close the all-variant result/error gaps and do not prove external fixture/package consumers.
- Fixture generators were not run, and no Unity artifact or production source/data/test/canonical file was written.

## 2026-08-07 — Final production and dirty-state verification

The exact baseline hash recipe (`sha256sum <ordered files> | sha256sum`) was repeated after all source reads, Cargo
metadata queries and focused tests.

| Category | Initial SHA-256 | Final SHA-256 | Result |
| --- | --- | --- | --- |
| Core behavior DTO | `a8e4256c9b53ac64511cdb3af83c2df2e9d8008a541aac9807aed573c1ae9b40` | `a8e4256c9b53ac64511cdb3af83c2df2e9d8008a541aac9807aed573c1ae9b40` | identical |
| world host/projection | `c1fd628986c6b7485c318dfa6734413cf8bbf01289195e1bf8546ba879e6fd89` | `c1fd628986c6b7485c318dfa6734413cf8bbf01289195e1bf8546ba879e6fd89` | identical |
| admin command/fixture | `d3d2af722ac32e0b0fd7a8685eb7658e907bd89b5520440b51eba7448aefd93b` | `d3d2af722ac32e0b0fd7a8685eb7658e907bd89b5520440b51eba7448aefd93b` | identical |
| Core fixture binaries | `1efd1eb31076954990c0ed899af40e49846626ddf32c73e684665d26e9d3a5f8` | `1efd1eb31076954990c0ed899af40e49846626ddf32c73e684665d26e9d3a5f8` | identical |
| Core package/tooling | `8432fcbaa2943b3b57dfb2e3f5cfd652721073d6868b0631ee84007fb22184b0` | `8432fcbaa2943b3b57dfb2e3f5cfd652721073d6868b0631ee84007fb22184b0` | identical |
| focused Core tests | `dbd6bd9a018520dc103a8f1fdeed0e6ce1df2b9f6f7620df796077c1a2be4e1c` | `dbd6bd9a018520dc103a8f1fdeed0e6ce1df2b9f6f7620df796077c1a2be4e1c` | identical |
| server direct bridge | `29752322a7fb06e12e1f9f41decefbf49a6fd3ac1be76fa8858f53a81ba1542e` | `29752322a7fb06e12e1f9f41decefbf49a6fd3ac1be76fa8858f53a81ba1542e` | identical |
| server fixture examples | `da8e1f1ce842643b148ab3c95252eb11edf9001ea1d70da5e2ce41f2315e0b02` | `da8e1f1ce842643b148ab3c95252eb11edf9001ea1d70da5e2ce41f2315e0b02` | identical |
| bridge/canonical docs | `369676382e250871cd0a06990f3de518e2d5c0f31b67bc133a9a426e14b15b41` | `369676382e250871cd0a06990f3de518e2d5c0f31b67bc133a9a426e14b15b41` | identical |
| scoped Git status text | `730b8acf5fd0475217400d6aee8cf45ab72674834248ce954b9164d1c5c4c58b` | `730b8acf5fd0475217400d6aee8cf45ab72674834248ce954b9164d1c5c4c58b` | identical |

All fifteen baseline dirty paths remain user-owned and unchanged by this Phase. Only Phase/Master audit artifacts are
eligible to differ.
