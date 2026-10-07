# Phase 36 CRA-P05-C05 Experiments

Status: complete
Implementation Complete: true

## 2026-08-11 — E01 Initialization And Evidence Freeze

### Hypothesis

Phase 36 can be initialized as the sole next candidate because Phase 35 exited `Complete / High`, Master marks
CRA-P05-C05 `ready`, no implementation Child is active, semantic dependency Phase 27 P05-C03 is `Complete / High`,
and historical/final evidence defines one assertion-transfer cleanup boundary under CRA-POL-006.

### Evidence reviewed

- the complete 874-line Goal authoring/execution guide;
- Master scope, CRA-POL-006, exact Phase registry, cleanup gates and activation/validation rules;
- the complete 212-line Phase-35 audit and complete 282-line semantic Phase-27 P05-C03 audit;
- original and post-Boss P05-C05 candidate, test and boundary evidence;
- final/rebaselined candidate catalogs, exact linear item 36, CRA-TC-02/03/04, test-rebaseline and consumer gates.

The assignment is one current-proof-first cleanup: remove no-op mission policy shape and redundant tests only after
caller/visibility and assertion inventories. No public API narrowing, content policy, data-driven rewrite or Phase-37
work is included.

### Exact initialization checks

- repository HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`;
- expanded porcelain status before this Child: 548 rows;
- canonical C-sort SHA-256:
  `9c03cb1c82f8acdd41686da5291ac30222aa155aa810d28ed7c389b3f4690172`;
- Phase 36 and 37 paths were absent; Master Phase 35 is `complete — Complete / High`, Phase 36 is `ready`, Phase 37
  is pending and no implementation Child is active;
- the protected marker remains exact.

### Commands and tests

Initialization used guides, Goal/history/catalog/registry, path-existence, status and marker evidence only. No
Phase-36-directed production content, helper/caller/assertion graph, test, source/data/canonical or external-consumer
inspection occurred. Executed tests: 0; pass/fail/ignored: not applicable.

### Result and decision

Initialize the Child around current caller/visibility proof, exact assertion transfer, behavior parity and explicit
public compatibility disposition. Detailed current-tree inspection remains forbidden until sole activation, full
active-document reread and prospective fingerprints close.

## Pending Evidence

- Master/Child sole activation and exact post-document status delta;
- full activation reread and prospective source/test/data/canonical fingerprints;
- current helper visibility/caller graph and exact candidate/survivor assertion matrix;
- nonzero focused baseline, test-first assertion transfer and minimal cleanup;
- focused/broad validation, compatibility/legacy audit and independent completion audit.

## 2026-08-11 — E02 Sole Activation

### Change and result

- All three Child headers changed from `initialized` to `active`; `Implementation Complete` remains `false`.
- Master Phase 36 changed from `ready` to `active`; Phase 35 remains `complete — Complete / High` and Phase 37
  remains pending.
- The three Child files explain the exact 548→551 expanded-status increase. Canonical C-sort SHA-256 is
  `88a595dd4801e12528349a8220a28318d6133219aa3ee7af74343d2526dade4a`.
- The protected marker remains byte/hash/time exact. No prospective production content, caller/assertion graph,
  source/data/canonical, test or external-consumer detail was inspected or changed.

### Commands and tests

Only document, registry, expanded-status and marker checks were used. Executed tests: 0; pass/fail/ignored: not
applicable.

### Decision

Phase 36 is the sole active Child. Detailed current inspection remains gated on a complete reread of all three
active documents, both dependency audits and path-known prospective fingerprints.

## 2026-08-11 — E03 Activation Reread And Prospective Fingerprints

### Procedure

- Fully reread all 401 active Child lines, the complete 212-line Phase-35 audit, complete 282-line semantic Phase-27
  audit and the full Master contract after sole activation.
- Reconfirmed original/post-Boss P05-C05 candidate/test/boundary evidence, exact linear item 36, CRA-TC-02/03/04,
  final test/consumer gates and CRA-POL-006.
- Used filename-only discovery plus path-known hash/status checks. No prospective production/test/data/canonical
  content or current caller/assertion graph was read.

### Prospective fingerprints

| Path | SHA-256 | Activation status |
| --- | --- | --- |
| root `Cargo.toml` | `98c9cf4dfefe3eda0ff7c692af94666a491a3460f073b3022c4c360b1a3e849c` | clean |
| root `Cargo.lock` | `4223318b9d62b4b1ecfe966543795a00445b938add11cae9413bbe6fd54d45da` | clean |
| `core_enhanced/Cargo.toml` | `40d8a9ec74b9c36862fbb580ad4bef554be03ab27875ceb9dadf9db93b5929af` | clean |
| `core_enhanced/src/lib.rs` | `3fbfdca2626de2a511ef6ffbbefe798b93f5469b79e63bfd135fa4cbc3dc8017` | clean |
| `combat_setup/mission_policy.rs` | `a186be5c6d1a57307581796925ff69cc809484e210d0ef8ed909b25da1ebcb5b` | modified before Phase 36 |
| `combat_setup/mod.rs` | `1d2471ba782f70fe8ee6810e3f5cf4ed0a0003deffe2fc3ef76da6b50a934346` | modified before Phase 36 |
| `events/combat.rs` | `999f9fb16711f90dfc63619749bbda565ec8fa5765169794aaaa1014155a326a` | modified before Phase 36 |
| `reward_policy.rs` | `c6d271a69fb5fa0c856d0d26e7e6cfbc947f4d0708b29a496d0e415f1124ee5b` | clean |
| `combat_preview/mod.rs` | `eec295fef907f18e73645f68a0cef14c4f7439d3733891ef68ef73fab654a567` | modified before Phase 36 |
| `world/combat.rs` | `918e8659cdcaff802a94dc823332e29429de95d7902cb40062a51a0016826d58` | modified before Phase 36 |
| live PVE encounters | `0b3f03b7e68c3aff5db6270e9eaa83eacb4eb2da188e55058899db22f0a3ee1c` | modified before Phase 36 |
| battlefield templates | `1f51d0e4cc1a784c684e60cf4bef9ee4b4e943aa776cd7db781ad7078144223f` | clean |
| battlefield archetypes | `c8251aab51532ee5f045fe182999e0f627e89c7e8ca936f2714bf165fc043dd9` | clean |
| node definitions | `64cc593e09712d6d619306679a97642635fad9deec6d9ad8108dcf4b0ebaa2e7` | clean |
| live run policy | `83278e068302506f683eb593260ff3fe1024f71c30236c3a4b46c3e47ef7fe55` | modified before Phase 36 |
| game rulebook | `6c53ecd5d0588aace1a2b043b31434d004327d60938ecf61066ac19acfd31951` | modified before Phase 36 |
| Core runtime canonical | `ab7d2f98040f15bfcb9b253ee06479b9989efc764fb57192447a23bc502c167e` | modified/staged before Phase 36 |
| data-loading canonical | `30ed5fd5924904b1376e23bd675b95732a4ede1360149224268077a426c3a83a` | modified before Phase 36 |

### Activation preservation

- HEAD remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded status remains 551 rows at canonical C-sort SHA-256
  `88a595dd4801e12528349a8220a28318d6133219aa3ee7af74343d2526dade4a`.
- The three Child paths are the only Phase-36 additions. Phase 36 is the sole active row; Phase 37 remains pending
  and absent.
- The protected marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, with exact timestamps.

### Decision

The activation gate is closed. Current content, visibility/callers, assertion bodies and focused baseline may now be
investigated. Existing modifications on five prospective source paths, two data paths and three canonicals must be
treated as immutable activation baselines unless an exact candidate-scoped merge is proven necessary.

## 2026-08-11 — E04 Current Caller, Assertion And Old/Current Baseline

### Visibility and caller graph

- `CombatMissionPolicy` lives below public `game::combat_setup::mission_policy`; both candidate methods are public
  Rust API.
- `preferred_node_types_for_map_node` has one production workspace caller in `world/map_encounters.rs`; that caller
  uses the returned node-type list as an encounter filter. Its elite and ordinary Combat paths currently both return
  exactly `[Defense]`; Boss returns `[Boss]`; every non-Combat/non-Boss category returns `[]`.
- `fallback_mission_variant_for_archetype` has no non-test workspace caller, but public visibility plus unknown
  deployed Rust consumers makes removal unauthorized under CRA-POL-006. Its exact signature and node-type-default
  behavior will be retained.
- No scoped candidate test carries `#[ignore]` or `#[should_panic]`.

### Candidate-to-survivor assertion matrix

| Removal candidate | Unique current assertion | Required survivor disposition |
| --- | --- | --- |
| `elite_map_nodes_prefer_high_pressure_missions` | elite Combat preference begins with `Defense` | move to exact all-category preference table in `only_current_encounter_node_types_are_supported` |
| `battlefield_archetype_does_not_change_mission_variant` | Defense + Surrounded wrapper returns `Defense` | expand `live_execution_is_explicit_per_mission_variant` to exact valid/invalid live pairs and all archetype/node-type wrapper outputs |
| `defense_node_type_builds_fixed_defense_objective` | objective is `ProtectUnit`, player plan is `FixedDefense`, win condition is `ProtectUnit` | no unique assertion: stronger `defense_node_type_builds_default_black_box_defense_objective` already asserts those three plus exact unit ref, route, wave movement, object source/position and full default shape |
| `defense_policy_requires_defense_identity_reward` | empty pool succeeds; currency-only Defense fails and error contains `featured grant kind` | move empty success and exact complete error text to `policy_requires_at_least_one_featured_grant_kind_when_rewards_exist` |

The SplitRoom survivor currently checks only `is_err`; strengthen it to the exact `InvalidStaticData` text while the
policy is unchanged. The Boss-positive and forbidden-kind reward tests remain separate because they protect distinct
success and rejection contracts.

### Old/current focused baseline

All commands ran against activation production before any Phase-36 source/test mutation:

- mission-policy module: 5 passed, 0 failed, 0 ignored, 1,021 filtered;
- reward-policy module: 4 passed, 0 failed, 0 ignored, 1,022 filtered;
- stronger default black-box Defense scenario: 1 passed, 0 failed, 0 ignored, 1,025 filtered;
- strict-subset fixed-Defense scenario: 1 passed, 0 failed, 0 ignored, 1,025 filtered;
- `ron_loading` integration: 25 passed, 0 failed, 0 ignored.

### Decision

Proceed test-first. Add the exact category/live/wrapper/SplitRoom and reward-empty/error assertions to the named
survivors and run those survivors before removing any body. Then simplify only the identical elite branch and remove
the four proven-redundant test bodies. Retain the public wrapper unchanged.

## 2026-08-11 — E05 Test-First Assertion Transfer

### Transfer performed before removal

- `only_current_encounter_node_types_are_supported` now pins the exact supported list plus ordinary/elite Combat,
  Boss and every non-encounter category output.
- `live_execution_is_explicit_per_mission_variant` now pins both valid and both invalid node-type/variant pairs and
  the retained public wrapper's Defense/Boss result across all eight battlefield archetypes.
- the SplitRoom survivor now matches the exact `InvalidStaticData` message;
- the featured-reward survivor now proves empty-pool acceptance and the exact complete currency-only Defense error.
- no scenario assertion was moved because the shorter test's three predicates already exist as stricter assertions
  in the stronger default black-box Defense survivor.

### Intermediate old-production results

Each named survivor selected exactly one test against the still-unsimplified production implementation and before
any redundant body was removed: 4 passed, 0 failed, 0 ignored in total. Each command reported 1,025 filtered tests.

### Decision

The assertion-transfer gate is green. It is now safe to simplify the identical elite branch and remove exactly the
two no-op mission bodies, one strict-subset scenario body and one duplicate reward body. The public wrapper remains.

## 2026-08-11 — E06 Minimal Cleanup And Focused Post-Change Validation

### Implementation

- Replaced the elite/ordinary identical branch in `preferred_node_types_for_map_node` with one exact category match:
  Boss → `[Boss]`, Combat → `[Defense]`, all other categories → `[]`. The public typed signature and sole production
  caller remain; only the now-unused argument name became `_kind_id`.
- Retained public `fallback_mission_variant_for_archetype` unchanged under CRA-POL-006.
- Removed exactly the four bodies proven redundant in E04/E05. No production data, manifest, DTO, save, canonical,
  server or Unity file was edited.

### Focused results

- mission-policy module: 3 passed, 0 failed, 0 ignored, 1,019 filtered;
- reward-policy module: 3 passed, 0 failed, 0 ignored, 1,019 filtered;
- stronger default black-box Defense scenario: 1 passed, 0 failed, 0 ignored, 1,021 filtered;
- `ron_loading`: 25 passed, 0 failed, 0 ignored;
- exact legacy-name search: no source hit for any of the four removed test names;
- caller search: the preference helper still has its one production caller; the public wrapper remains defined and
  is now exercised only by the expanded compatibility survivor.

`cargo fmt --all -- --check` initially identified only one trailing blank line at each of the two edited test-module
ends. Those two lines were removed with the candidate patch; the rerun passes. Candidate-scoped `git diff --check`
also passes.

### Exact discovery delta

Post-cleanup `cargo test -p game_core --all-targets -- --list` exposes exactly 1,073 sorted `: test` rows at SHA-256
`391152c09e75da5c1d5a8006f1dbf28403f3dacea17e6fc55cd0d50c02510c31`. Adding only the four intentionally removed
rows and C-sorting reconstructs the Phase-35/pre-Phase-36 1,077-row table exactly at
`be0d705e1d8aadc3fbf669fc29372333fe5c42a8d309260fc33ea0720a7b3c38`. Therefore there is no hidden added, renamed
or removed test beyond the approved four-body delta.

### Decision

The candidate is implementation-complete but not Phase-complete. Run actual debug/release all-target suites,
workspace checks, format/diff/Clippy gates, prospective hash/status preservation and the separate completion audit.

## 2026-08-11 — E07 Broad Validation And Preservation Closure

### Actual broad test execution

- `cargo test -p game_core --all-targets`: 1,073 passed, 0 failed, 0 ignored in debug.
- `cargo test -p game_core --release --all-targets`: 1,073 passed, 0 failed, 0 ignored in release.
- Both profiles have the exact harness distribution: library 1,022; integrations 4 + 4 + 25 + 14 + 4; the
  existing default main plus five fixture binaries each report 0/0/0.
- The 25-test live-RON suite therefore passed both independently in focused debug and within both all-target
  profiles.

### Static gates

- `cargo check --workspace --all-targets`: pass;
- `cargo check --workspace --release --all-targets`: pass;
- `cargo fmt --all -- --check`: pass after the E06 two-line whitespace correction;
- whole-worktree `git diff --check`: pass;
- `cargo clippy -p game_core --all-targets` and release equivalent: exit 0. Both retain the inherited baseline of
  82 library and 134 library-test warnings; no candidate-path warning was introduced.

### Dirty-work and compatibility preservation

- Expanded status is 552 rows at canonical C-sort SHA-256
  `90fdf6dcef93f88cd0b2d097bcfdc4ff9a1250d6b7eb71dcecbb6c298551316c`. Relative to the 551-row activation set,
  the only new status path is formerly-clean `reward_policy.rs`; `mission_policy.rs` and `events/combat.rs` were
  already modified and received only the exact candidate hunks. No unrelated status path was added or removed.
- Final candidate hashes are mission policy `87465db2df6771c9be2b508c519c246167329338f98f8fd6ab6055779ff2205c`,
  event combat `7b25d74a56c96481eb1a2e560597ce40342edb7caf3fb6cda8285db757a500a6`, and reward policy
  `b2b4b5f7ecd5b67a4317869a3261e6308e0e9c9822a75b13d4de14d3dd27fe1c`.
- Every non-target prospective hash remains activation-exact: manifests/lock/library root; public module root;
  combat preview and world combat; live encounters, battlefield templates/archetypes, node definitions and run
  policy; game rulebook, Core runtime contract and data-loading contract.
- The first final hash command used shorthand paths that do not exist for those eight data/document inputs and
  reported `No such file or directory`; filename discovery resolved their actual repository paths immediately, and
  the rerun produced all eight exact activation hashes. No file was mutated by either read-only command.
- The protected marker remains exactly 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, with mtime and ctime both
  `2026-08-10 22:55:07.178423600 +0900`.

### Decision

Implementation and validation evidence satisfies P36-C01 through P36-C10. Keep all headers active/false and Master
Phase 36 active while a separate completion audit rereads the current artifacts and independently tests the claim.

## 2026-08-11 — E08 Pre-Audit Input Normalization

The first completion-review reread found two PLAN sentences that still described current caller/test discovery as
pending even though E04 through E07 had completed it. Before creating an audit artifact, those two documentation-only
sentences were updated to point at the current public-caller disposition, exact four-body assertion transfer and
recorded nonzero filters. No production, test, data, canonical or Master status changed. Because an audit input
changed, the completion review restarts from the current PLAN/experiments/notes rather than relying on the partial
review pass.

## 2026-08-11 — E09 Completion Audit And Linear Exit

- The restarted independent 266-line audit at SHA-256
  `65c2adfe765df5e01b9bcb1fe4acbf8c54bc5175c786409d353eeec724eca32b` returns `Complete / High`, improvement
  class `none`, with all P36-C01 through P36-C11 rows passing and no material finding.
- Audit-refresh focused results are mission 3/3, reward 3/3, stronger Defense 1/1 and live RON 25/25. Fresh Core
  debug all-target execution passes 1,073/0/0. Exact discovery and synthetic four-name reconstruction retain hashes
  `391152c0...` and `be0d705e...` respectively.
- The audit artifact alone explains the exact 552→553 expanded-status increase at canonical C-sort SHA-256
  `729b1a8b113f67de2fc0662ce54481fedbf39b5316c3b2f0460e53a9d918e086`. The protected marker remains exact.
- Phase 36 transitions to `complete` / `Implementation Complete: true`; the Master row transitions to
  `complete — Complete / High`. Phase 37 becomes `ready` only and remains absent until its own initialization and
  sole-activation protocol.
