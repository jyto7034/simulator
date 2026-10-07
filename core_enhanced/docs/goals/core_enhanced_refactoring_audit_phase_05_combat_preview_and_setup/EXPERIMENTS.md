# Core Enhanced Refactoring Audit Phase 05 — Experiments

Status: complete

## 2026-08-07 — Activation gate

### Evidence and result

- Parent Master and Phase 0 boundary were rechecked.
- Direct predecessor Phase 4 is `Complete / High`; Phase 5 reserved path did not exist.
- Phase 1 C02, Phase 2 run/difficulty carrier, Phase 3 roster/economy handoff and Phase 4 Event-combat handoff were
  identified as predecessor decisions.
- No other active child exists. Phase 5 is investigation-only and production mutation remains prohibited.
- Exact start commit remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.

### Tests

No runner was executed during activation.

### Decision

Phase 5 may perform read-only production investigation and write only audit artifacts. Phase 6 remains pending.

## 2026-08-07 — Scoped production baseline

### Exact categories and initial hashes

| Category | Initial SHA-256 |
| --- | --- |
| `combat_preview/**` (5 files) | `4aed5617eb6b380fb598bf04b159c9dcd4836a3aa2ba8785db395daf8f5618c5` |
| `combat_setup/**` (9 files) | `ff28e961f42593c479ec442dae0124ae7df3dc9f2a44fb3cf7d96bf15585c9c3` |
| shared handoff/runtime (6 files) | `c7f2067e427cf5e6b4c0dd2dd4573d62b502024949f7f78c9e904a5d1d0ccdf6` |
| typed data + live RON (8 files) | `d69711b49028add693fb4f835cd3f5f81084b525133dabce83b7b8e49e0dff75` |
| focused test source (3 files) | `b8d5e387304896d60918cd1bd339c7eccd569d522da54c894db417dc8886af72` |
| nearest canonical (3 files) | `da3f35594f77174b9251da9b6ee7a2fb2e9e4fb9bfeb2da33c366034325c6fac` |
| scoped Git status text | `7e768dd5d69ea320d641cdb919f65ea8adc12030977d28a7879f951e4846025a` |

Shared paths are `wave_resolution.rs`, `combat_power.rs`, `pve_bonus_objectives.rs`, `reward_policy.rs`,
`world/combat.rs`, `world/state.rs`. Data paths are the three typed modules plus live encounter, two battlefield,
run-policy and equipment RON files. Test paths are world combat/map-flow and `tests/ron_loading.rs`.

### Baseline correction

The first data hash command used the nonexistent `game_resources/data/equipments.ron`; `sha256sum` reported that
error, so its partial hash was discarded. The command was corrected to `game_resources/data/equipments/base.ron`
before detailed file reads, producing the authoritative data hash above.

### Dirty-state result

Sixteen scoped production paths were already modified: the three canonical files, combat power, mission policy,
player spawns, three typed data modules, world combat/state, world combat tests, RON loading test, and live
equipment/encounter/run-policy RON. These are user-owned baseline changes.

### Tests

No runner was executed for the baseline.

## 2026-08-07 — Preview, setup and handoff trace

### Evidence and result

- Traced manifested node/Event input through deterministic template/wave/briefing generation, cached preview,
  rewards, scenario groups, defense object, `BattleCore`, execution preflight and world commit.
- Confirmed `BattlefieldGenerationRequest.floor_index` has no read and manifested difficulty is the sole scale,
  wave-budget and reward authority.
- Confirmed `CombatPreview.enemy_stat_scale` is skipped by serde while `RunCheckpointPayload` serializes preview
  caches. Node cancel preserves a cache entry; durable export/import restores it; both setup resolvers trust it;
  enemy drafts consume the defaulted scale. Recorded as C01 correctness correction.
- Confirmed official validation generates baseline previews but does not run setup-only capacity/tactical/reference
  validation. Recorded as distinct C02 correctness correction, coordinated with Phase 1 C02.
- Confirmed preview conversion/resolution duplication (C03), aggregate setup assembly duplication (C04) and no-op
  mission policy/test shapes (C05).
- FacilityEntity, manual-wave scaling, armed eligibility, Event paired commit and post-start battle lifecycle were
  classified without reopening predecessor/later ownership.

### Static commands

```text
rg -n "floor_index|try_generate_for_node_at_floor|explicit_combat_preview|combat_preview_for_node|BattlefieldGenerationRequest|enemy_stat_scale" core_enhanced/src core_enhanced/tests
rg -n "combat_previews" core_enhanced/src core_enhanced/tests core_enhanced/docs
rg -n "build_battle_with_combat_preview|resolve_rewards_for_mission|BattleStartPlan|enemy_spawn_groups_from_preview" core_enhanced/src/game/events/combat.rs
rg -n "battlefield_archetype_does_not_change_mission_variant|preferred_node_types_for_map_node|fallback_mission_variant_for_archetype" core_enhanced/src core_enhanced/tests
rg -n "#\[ignore" <all Phase 5 source/test paths>
```

No ignored test marker was found in the scoped source/test paths.

## 2026-08-07 — Focused validation

| Command/filter | Passed | Failed | Ignored | Meaning |
| --- | ---: | ---: | ---: | --- |
| `cargo test -p game_core --lib combat_preview -- --nocapture` | 31 | 0 | 0 | preview topology, determinism, scale/serde plus overlapping consumers |
| `cargo test -p game_core --lib combat_setup -- --nocapture` | 8 | 0 | 0 | mission and reward setup policy |
| `cargo test -p game_core --lib game::events::combat::tests -- --nocapture` | 13 | 0 | 0 | preview-to-scenario strict handoff |
| `cargo test -p game_core --lib game::wave_resolution::tests -- --nocapture` | 2 | 0 | 0 | generated/manual wave budget semantics |
| `cargo test -p game_core --lib game::combat_power::tests -- --nocapture` | 4 | 0 | 0 | certified formula and armed membership |
| `cargo test -p game_core --lib game::reward_policy::tests -- --nocapture` | 4 | 0 | 0 | mission reward identity |
| world combat filter `combat_map_node` | 2 | 0 | 0 | no-deployable entry policy |
| world combat filter `battle_setup_snapshot` | 3 | 0 | 0 | setup scene projection |
| world combat filter `weaponless` | 4 | 0 | 0 | armed admission/recovery behavior |
| exact run-power authority test | 1 | 0 | 0 | manifestation/entry peak handoff |
| exact authored-route setup snapshot test | 1 | 0 | 0 | preview route equals runtime setup |
| exact setup-loss recovery test | 1 | 0 | 0 | setup recovery read-only parity |
| exact live DefenseRoute playable smoke | 1 | 0 | 0 | official full setup/result path; 33.89 s |
| exact missing manifested difficulty test | 1 | 0 | 0 | fail-closed preview authority |
| exact `ron_loading::live_pve_references_resolve` | 1 | 0 | 0 | official reference graph |
| exact `ron_loading::live_pve_scenario_authoring_contracts_drive_preview_data` | 1 | 0 | 0 | official authored preview/setup content |
| exact `ron_loading::live_combat_previews_include_required_ad_ap_threat_warning_tags` | 1 | 0 | 0 | official warning coverage |

Seventeen invocations executed 79 passing test instances. Three overlapped earlier filters, yielding 76 observed
unique tests. Green execution does not close C01's missing durable-cache sequence or C02's accepted-but-infeasible
publication cases.

## 2026-08-07 — CRA-14 disposition

- Classified 15 assigned families by production path, unique invariant, assertion quality and overlap.
- Confirmed four exact merge/removal opportunities: two no-op mission helper tests, one strict-subset default-defense
  test and one duplicate Defense currency-only reward-policy test.
- All other apparent overlap retains distinct topology, live publication, world transaction, route, retry or setup
  projection meaning.
- Missing C01~C04 safety nets are candidate prerequisites and are not hidden by the 79/79 runner result.

## 2026-08-07 — Final no-mutation proof

All final hashes equal their initial values:

| Category | Initial SHA-256 | Final SHA-256 | Result |
| --- | --- | --- | --- |
| `combat_preview/**` | `4aed5617eb6b380fb598bf04b159c9dcd4836a3aa2ba8785db395daf8f5618c5` | `4aed5617eb6b380fb598bf04b159c9dcd4836a3aa2ba8785db395daf8f5618c5` | identical |
| `combat_setup/**` | `ff28e961f42593c479ec442dae0124ae7df3dc9f2a44fb3cf7d96bf15585c9c3` | `ff28e961f42593c479ec442dae0124ae7df3dc9f2a44fb3cf7d96bf15585c9c3` | identical |
| shared handoff/runtime | `c7f2067e427cf5e6b4c0dd2dd4573d62b502024949f7f78c9e904a5d1d0ccdf6` | `c7f2067e427cf5e6b4c0dd2dd4573d62b502024949f7f78c9e904a5d1d0ccdf6` | identical |
| typed data + live RON | `d69711b49028add693fb4f835cd3f5f81084b525133dabce83b7b8e49e0dff75` | `d69711b49028add693fb4f835cd3f5f81084b525133dabce83b7b8e49e0dff75` | identical |
| focused test source | `b8d5e387304896d60918cd1bd339c7eccd569d522da54c894db417dc8886af72` | `b8d5e387304896d60918cd1bd339c7eccd569d522da54c894db417dc8886af72` | identical |
| nearest canonical | `da3f35594f77174b9251da9b6ee7a2fb2e9e4fb9bfeb2da33c366034325c6fac` | `da3f35594f77174b9251da9b6ee7a2fb2e9e4fb9bfeb2da33c366034325c6fac` | identical |
| scoped Git status text | `7e768dd5d69ea320d641cdb919f65ea8adc12030977d28a7879f951e4846025a` | `7e768dd5d69ea320d641cdb919f65ea8adc12030977d28a7879f951e4846025a` | identical |

Only Master/Phase audit artifacts changed. Cargo produced ignored `target` artifacts; no production/data/test/
canonical/server/Unity file was edited, formatted, moved or deleted.
