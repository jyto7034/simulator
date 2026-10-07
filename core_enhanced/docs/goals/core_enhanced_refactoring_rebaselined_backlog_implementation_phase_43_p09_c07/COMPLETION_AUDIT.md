# Phase 43 CRA-P09-C07 Completion Audit

Audit Mode: phase_exit
Audit Target: `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_43_p09_c07`
Current Status: active
Implementation Complete Claim: false pending this audit
Policy Source: CRA-POL-006 plus the user's 2026-08-11 approved public/serialized cleanup and explicit retained
tile-range/`Nearest` boundary
Goal Documents: this Child PLAN, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`; Master PLAN/EXPERIMENTS/NOTES
Repository/Commit/Dirty State: `/mnt/f/work/simulator`, `f318414fb0156a67e04c5479000c2a2683fc0c09`,
intentionally dirty; 602 expanded rows before this artifact at canonical C-sort SHA-256 `a7edd0b7...`
Audit Date: 2026-08-11 (`Asia/Seoul`)

## Audit Independence And Inputs

- This pass reread the complete completion-review guide, the Master Phase-43 policy/dependency/next-seam registry
  and all three current Child documents after implementation validation.
- It then read the current schema, tile eligibility, `Nearest`, equipment publication and per-unit delivery code;
  reran executable/data/external legacy searches; reran focused authority and strict-RON tests; and reran Core and
  server debug all-target suites.
- No production, data, canonical or package file was changed during the audit pass. This document is the audit's
  only new path.
- Phase 42 is `Complete / High` at audit hash `dc2abc41...`. Master still has Phase 43 as the sole active Phase and
  Phase 44 as pending/absent, so the direct linear gate is intact.

## Completion Evidence Matrix

| ID | Completion condition or invariant | Policy basis | Runtime/data/contract evidence | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- |
| P43-C01 | current range/delivery/public graph is exact | Child inventory gate; CRA-POL-006 | strict BasicAttack/Weapon schemas, authored conversion, `apply_weapon_profile`, tile query and per-unit delivery paths reread | exact definition/caller/writer search | pass |
| P43-C02 | retained eligibility authority is exact | user's tile-range/`Nearest` boundary | `targeting.rs:48-64,297-329`; `skill_runtime/cast.rs:395-435` | tile target 1/1, tile reach 1/1, Nearest 2/2 | pass |
| P43-C03 | dirty user work is preserved | CRA-POL-006; dirty-work rule | prospective fingerprints, five complete formerly-clean path diffs, current manifests/canonicals/marker | 602-row current status; scoped diff and hash review | pass |
| P43-C04 | every candidate surface is classified | Child P43-1 | two ghost serde fields, false base helper and scalar wrapper/leaf removed; typed query/runtime-unit paths retained | exact remaining-name classification | pass |
| P43-C05 | old behavior was characterized | required assertion-transfer gate | pre-removal authority tests and existing tile/Nearest/preview/live-data controls in E04 | all mandatory pre-removal filters selected nonzero | pass |
| P43-C06 | useful assertions preceded cleanup | review-guide test rule | runtime equipment and per-unit release assertions were added while old surface existed | old-shape 1/1 + 1/1 and post-shape 1/1 + 1/1 | pass |
| P43-C07 | public/serialized break was approved and inventoried | user approval replacing compatibility default | zero workspace/external runtime consumer; 66 official writers identified and migrated | accessible `/mnt/f/work` non-doc search | pass |
| P43-C08 | only ghost/false surface is removed | approved cleanup scope | exact field/copy/validator/helper/wrapper deletion; no alias/default/shim | strict legacy schema 2/2; shim search zero | pass |
| P43-C09 | targeting and delivery behavior is exact | hard retained boundary | tile membership and current-target/nearest order unchanged; runtime weapon delivery copied and read per unit | focused parity plus Core debug/release 1,063 each | pass |
| P43-C10 | data/DTO/save/server/Unity boundary is exact | CRA-POL-006 | 66 RON keys migrated; no candidate carrier in DTO/save/server/accessible external source | live RON 25/25; server debug/release 63 each | pass |
| P43-C11 | broad/static validation is green | Child minimum contract | Core/server/workspace manifests and production sources compile in both profiles | both-profile tests/checks, Clippy, format, diff | pass |
| P43-C12 | next Phase opens only after independent exit | Master linear protocol | this separate audit, no Phase-44 path or package mutation | audit refresh below; `Complete / High` | pass |

## Runtime And State Evidence

- `BasicAttackDef` retains `range_policy`, optional `defense_tile_range`, targeting, timing and delivery; it no
  longer contains a scalar range. `AuthoredBasicAttackDef` requires the same combat fields without a parallel
  scalar authority and denies unknown keys.
- `BattleCore::is_basic_attack_target_in_range_with_policy` first enforces current target capability and then calls
  only `is_target_in_tile_range_policy`. `basic_attack_range_policy_for_unit` derives delivery, tile policy/pattern
  and air capability from the runtime unit.
- `UnitTargetRule::Nearest` first retains the current eligible/useful hostile target. Its fallback considers only
  candidates admitted by `in_range` and `is_useful`, then uses world-distance squared and stable UUID tie-breaking.
  No Phase-43 code altered this body.
- `UnitCombatProfile::apply_weapon_profile` copies the equipped profile's tile pattern, targeting, timing and exact
  delivery into runtime state. Real start/release callers use Core-private `basic_attack_delivery_for_unit`, which
  resolves the current unit and defaults only for a missing unit; the removed base-UUID metadata helper cannot
  compete with equipped runtime state.
- Windup, release-time revalidation, projectile motion/impact, damage handoff, event order and target usefulness
  were not edited. Their unchanged carriers and full all-target suites provide the broad cross-path evidence.

## Data Evidence

- `BasicAttackDef` and `WeaponCombatProfile` both use `#[serde(deny_unknown_fields)]`; direct negative tests feed
  the removed `range_units` key and require deserialization failure.
- Official production migration removes exactly 30 abnormality, 6 corroded-employee and 30 equipment writers.
  Exact `range_units:` search over those three files selects zero, while the current strict bundle passes 25/25.
- Programmatic constructors, validation, test fixtures and the equipment-to-basic copy were migrated in the same
  cutover. There is no dual shape, alias, ignored key or silent scalar fallback.
- Skill-data `range_units` text is an independent strict rejection test for a removed skill-step key. Core mod's
  local `single_step_skill(range_units)` only materializes a test tile radius; neither is a retained
  BasicAttack/Weapon schema or production eligibility authority.

## DTO, Transport And Persistence Evidence

- Current Core/server source search finds no candidate field/helper in a DTO, save/checkpoint envelope, server
  message, handler, actor state or persistence store. Removing the candidate therefore changes no existing V3
  checkpoint or wire payload.
- Server production carriers keep their activation hashes, and both server profiles pass 63/63. No Unity-facing
  canonical shape or message ordering changed.
- A save-version migration probe is not applicable because neither removed field was checkpoint state. A Unity
  build is not required because there is no client schema or source consumer; server and canonical boundaries are
  unchanged and directly checked.

## Consumer Evidence

- Confirmed workspace consumers: every Core definition, writer, validator, copy, test helper and runtime caller was
  classified before removal; official live RON was migrated.
- Confirmed accessible external consumers: a fresh non-doc/non-build search under `/mnt/f/work` returns only this
  repository's two strict rejection tests and unrelated local skill radius helper. No other source consumer was
  found.
- Canonical external consumer: server/Unity wire behavior remains the existing battle DTO contract and contains no
  candidate member.
- Inaccessible/unknown consumer: no separately declared deployed package or inaccessible consumer is known. The
  user explicitly approved this public/serialized break after the accessible inventory, without a shim.

## Legacy And Compatibility Audit

- Exact function/field search finds no `bodies_in_range`, exact `basic_attack_delivery(...)`, `can_reach`, public
  `range_units` member or official RON writer.
- Alias/rename/default/ignored/shim search selects zero. The only removed names remaining are deliberate rejection
  strings and the unrelated test-local tile-radius convenience described above.
- No world-distance check was added to basic eligibility. The retained typed tile query and current `Nearest`
  controls both select nonzero and pass.
- No ignored/disabled test, debug bypass, dual-read/write, placeholder state or Phase-44 package compatibility work
  was introduced.

## Scoped Diff And Dirty Preservation

- All candidate implementation carriers that were dirty at activation remain bounded by their activation hashes;
  unchanged prospective carriers retain their exact hashes.
- Five clean paths became dirty only for Phase 43: spatial helper/test, defense-object constructor, skill-test common
  constructor, abnormality RON and corroded-employee RON. Their complete HEAD diffs contain only the classified
  Phase-43 removal/migration.
- The exact current status has 602 rows before this audit artifact at `a7edd0b7...`. Removing those five paths
  reconstructs 597 rows at `bdc1dbde...`, not the recorded activation `d981d0ee...`. This is an evidence-hash
  anomaly, not concealed equivalence. Exact path identities, content fingerprints and complete scoped diffs remain
  sufficient to prove preservation; no material status path is unexplained.
- This audit artifact alone changes expanded status to 603 rows at canonical C-sort SHA-256 `1510b29c...`.
- Workspace/Core/server manifests, lockfile and Core root remain `98c9cf4d...`, `40d8a9ec...`, `19b41cd...`,
  `4223318b...` and `3fbfdca2...`. Protected marker remains 57 bytes at `614d2fbd...` with timestamps unchanged.
- No reset, restore, stash, clean, deletion of unrelated files or external write was performed.

## Canonical Alignment

- `data_loading_contract.md` lists the current required authored BasicAttack fields and no removed scalar.
- `skill_target_contract.md` continues to make tile patterns/policies the eligibility authority and explains that
  body radius/continuous distance does not decide basic range.
- Runtime/gameplay/server canonicals remain unchanged because Phase 43 did not alter their contracts. Goal prose is
  not the sole owner of any implemented behavior.

## Validation Results

Focused audit refresh in the current dirty tree:

- `cargo test -q -p game_core --lib equipped_weapon_profile_is_runtime_basic_attack_delivery_authority -- --nocapture`:
  1 passed, 0 failed, 0 ignored; 1,011 filtered.
- `cargo test -q -p game_core --lib basic_attack_release_delivery_reads_runtime_unit_profile -- --nocapture`:
  1 passed, 0 failed, 0 ignored; 1,011 filtered.
- exact fixed-defense tile target and weapon-preview filters: 1/1 each, zero failure/ignore.
- `cargo test -q -p game_core --lib nearest_skill_rule -- --nocapture`: 2 passed, zero failure/ignore.
- typed tile reach: 1 passed; removed-schema rejection: 2 passed; zero failure/ignore.
- `cargo test -q -p game_core --test ron_loading -- --nocapture`: 25 passed, zero failure/ignore.

Broad audit refresh:

- `cargo test -q -p game_core --all-targets`: library 1,012 plus integrations `4 + 4 + 25 + 14 + 4` =
  1,063 passed, 0 failed, 0 ignored.
- `cargo test -q -p game_server --all-targets`: 63 passed, 0 failed, 0 ignored.

Current implementation validation, still valid because only Goal documents changed afterward:

- Core release all-targets: 1,063 passed, 0 failed, 0 ignored.
- Server release all-targets: 63 passed, 0 failed, 0 ignored.
- Workspace debug/release all-target checks: both exit 0.
- Normal Core/server Clippy: both exit 0 with inherited warnings and no new Phase-43 warning.
- `cargo fmt --all -- --check` and `git diff --check`: both exit 0.
- Sorted Core library list: 1,012 at `66f515f1...`; server: 63 at `beca9e7a...`.

## Baseline Failures And Rejected Evidence

- The first pre-cleanup test build omitted a local `DeliveryDef` import. No behavior binary ran; the test-authoring
  error was fixed and all required baselines then selected nonzero and passed.
- Two first post-cleanup guessed short filters selected zero. They are not counted as success; the current list
  supplied exact names and both were rerun 1/1.
- No focused or broad runtime failure remains. No ignored test is used as completion evidence.

## Cross-Component Findings

- No material finding. Live data, runtime tile targeting, equipped delivery and server boundaries use one coherent
  final shape. There is no temporary shim or duplicate scalar authority between this Phase and the Phase-44 package
  seam.
- Phase 44 remains a separate package/target/dependency cleanup. Its path is absent and all manifests retain their
  activation hashes, so this Phase did not pre-implement or constrain it.

## Long-Term Direction Review

| Candidate ID | Work type | Current to target authority outcome | Preserved invariant outcome | Removal/consolidation outcome | Validation | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| CRA-P09-C07 | cleanup | typed tile query and runtime equipped unit are now the only relevant authorities | tile sets, current `Nearest`, preview, delivery, DTO/save/wire remain exact | two ghost serde fields, 66 writers, false helper and scalar wrapper/leaf removed without shim | focused parity + both-profile broad suites | pass |

- Fit: High.
- Improvement class: none.
- Reason: the cleanup removes synchronized-but-unused scalar state and misleading public entry points, strengthens
  strict schema failure, and leaves the established typed/runtime ownership graph intact. It adds no replacement
  layer, special-case production branch or future placeholder.

## Primary Verdict

- Complete.
- Long-term fit: High.
- Confidence: High.

All twelve Child conditions pass with current runtime/data/consumer evidence. The status-hash anomaly is explicitly
bounded by exact current path evidence and is not a material implementation or preservation finding.

## Status Transition

- from: `active`, `Implementation Complete: false`.
- to: `complete`, `Implementation Complete: true`.
- Master Phase 43 registry: may transition to `complete — Complete / High; approved public/serde break applied`.
- Phase 44 activation: allowed only after the Child/Master completion records are synchronized; then it becomes
  `ready`, not active production work inside this audit.

## Remaining Risk And Required Action

- No material correction or policy decision remains.
- Preserve the explicit activation-hash anomaly in the durable Phase record; do not reuse `d981d0ee...` as a
  reconstructed status proof.
- Synchronize Child and Master completion state, capture this audit artifact's exact line/hash/status, and only then
  initialize/activate Phase 44 under the normal sole-active protocol.
