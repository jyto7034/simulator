# Phase 28 CRA-P08-C05 Experiments

Status: complete
Implementation Complete: true
Candidate: CRA-P08-C05
Master Phase: 28 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Experiment Rules

- Do not inspect Phase 28 production source, test implementation, live data or canonical content before sole
  activation and the complete activation protocol.
- Record actual nonzero test counts; zero selected cases are not evidence.
- Characterize instant/projectile/general committed damage, drift, crit, retaliation, kill, lineage and command order
  on old production before refactoring.
- Retain the pure formula, Phase 11/13 occurrence matrix and Phase 12 HP/death/provenance authorities.
- CRA-POL-006 forbids public carrier narrowing on workspace-only evidence.
- Preserve all unrelated dirty work, including the 57-byte Phase 27 approval marker; do not reset, restore, clean,
  stash, delete artifacts or broadly format.
- Phase 29 work is forbidden.

## Historical Hypothesis To Verify

Historical evidence reports one pure damage formula surrounded by overlapping instant-basic, projectile-impact and
general `ApplyDamage` orchestration. A private typed committed-damage authority should remove the duplicate context,
commit and summary mechanics while keeping explicit delivery/trigger policy and all observable behavior exact.

The formula-derived `BattleCommand::UnitDied` and broad request/context/result fields may be ghost authority, but
current reader/consumer inventory must decide removal. Unknown deployed Rust consumers require retaining public
surfaces unless separate migration evidence exists.

## Required Experiment Sequence

1. Activation/status/prospective fingerprint capture.
2. Complete formula/request/context/commit/trigger/resonance/death/result/caller and consumer inventory.
3. Nonzero focused old-production baseline.
4. Test-first instant/projectile/general drift/crit/retaliation/kill/lineage/order characterization.
5. Smallest typed committed-damage consolidation around retained formula and death authority.
6. Carrier/public/fallback/duplicate/data/persistence/canonical audit.
7. Focused then broad Core/server debug/release validation.
8. Independent 567-line completion-guide phase-exit audit.

## 2026-08-10 — E01 Initialization

- Phase 27's fresh 282-line `Complete / High` audit was read after its exact one-path exception review. Master records
  Phase 27 complete and Phase 28 ready.
- Phase 11 P08-C01, Phase 12 P08-C02 and Phase 13 P08-C04 final `Complete / High` audits were read as the direct
  occurrence, HP/death/provenance and publication-matrix dependencies.
- Original/re-audited P08-C05 evidence, Final catalog/order/test/gate records and CRA-POL-001/002/006 were read before
  this Child was written.
- Pre-initialization expanded status is 513 rows at normalized SHA-256
  `0df73ab3f4add04e7a281d6a50eb9d4d26c9f38a2949b710cc4214c82f1a6471`. The current 57-byte user approval marker
  at `game_server/logs/app.log.2026-08-09` is SHA-256 `614d2fbd...` and is a protected dirty-boundary input.
- Goal path collision: none. This initialization creates only the three required Phase 28 documents. Production
  investigation/mutation and test execution are forbidden until Master sole activation and mandatory reread/hash
  capture.

## 2026-08-10 — E02 Sole Activation And Prospective Fingerprints

- Master and all three Child headers identify Phase 28 as the sole active Phase. The active 222-line PLAN, 69-line
  experiments, 80-line notes, final 282-line Phase 27 audit and final Phase 11/12/13 dependency audits were reread
  completely in bounded sections after activation; an earlier truncated combined read was discarded.
- HEAD is `f318414fb0156a67e04c5479000c2a2683fc0c09`. Expanded activation status is 516 rows at normalized
  C-locale-sorted SHA-256 `8c67776899e009651dfb5e75110fbae4926c9344ce2e251f4e45be6de7f3c83d`.
  The three-row increase from the 513-row pre-initialization state is exactly the three Phase 28 Goal documents.
- Candidate paths were discovered by filename only before reading source, test, live-data or canonical content.
  Most prospective tracked targets were already modified and therefore remain user-owned overlap boundaries; a
  clean-path assumption is not made.
- Prospective formula/carrier hashes: `battle/damage.rs` `e92d0377...`, `battle/types.rs` `5d714af9...`, Core
  `types.rs` `ba33c56b...`, `commands.rs` `08c77399...` and `triggers.rs` `567bc636...`.
- Prospective execution hashes: `damage_runtime.rs` `4253a1a2...`, `basic_attack_projectile.rs` `32abc304...`,
  `death_runtime.rs` `096a6826...`, `resonance_runtime.rs` `2fb54332...`, `sim.rs` `9d4eecc6...` and
  `projectile_math.rs` `e96d9f1d...`.
- Prospective skill-delivery hashes: `skill_runtime/mod.rs` `6bd8faff...`, `cast.rs` `8c586e5a...`,
  `projectile.rs` `07dd8844...` and `area.rs` `23e25cdf...`.
- Prospective integration-test hashes: live item `84e4e600...`, skill runtime `afd09b3f...`, skill suite
  `4f2186b1...`, live catalog `bf475aa3...` and RON loading `75558ff2...`.
- Prospective live-data hashes: Equipment `322d7455...`, Skill `4087df05...` and Buff `e7cc335f...`.
- Prospective canonical hashes: Core runtime `6e20d6a6...`, rulebook `6c53ecd5...`, data loading `30ed5fd5...`, server
  bridge `4d734af0...` and external Unity Core contract `a3224e24...`.
- The protected approval marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`, with unchanged mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`.
- Activation protocol steps 1–5 are complete. Current production/detail inspection is now authorized for Phase 28
  only; production mutation still waits for complete inventory, dirty-overlap capture, nonzero baseline and
  characterization-first tests.

## 2026-08-10 — E03 Current Authority, Caller And Consumer Inventory

- `battle/damage.rs::calculate_damage` is the single public pure arithmetic authority. Read-only
  `target_usefulness.rs` and `tactical_stance.rs` call it without mutation. Its public `DamageRequest`,
  `DamageContext`, `DamageResult`, `BattleCommand` and `DamageSourceSnapshot` carriers remain compatibility surface;
  workspace-only absence cannot authorize field or variant removal under CRA-POL-006.
- Three committed execution assemblies remain. Instant basic calls `calculate_basic_attack_damage_snapshot`, then
  grants dealt resonance, calls `apply_damage_result_and_record` and releases occurrence batches. Basic projectile
  impact independently reconstructs the same occurrence/context/formula/resonance/commit/batch order. General
  `BattleCommand::ApplyDamage` independently rebuilds formula context, commits HP/death and derives execution counts.
- `apply_damage_result_and_record` is the existing singular damage HP authority: one active-target lookup, actual HP
  subtraction, one metadata-complete `HpChanged`, actual-delta basic received resonance and one handoff to
  `finalize_unit_death`. Phase 12's runtime-stat HP bridge is separate and retained.
- Basic source lifetime is launch/attack-start `DamageSourceSnapshot`; target owner, defense, magic resistance,
  incoming modifiers and HP are read live at commit. Instant occurrence identity is the source-instance UUID and
  its occurrence time is the snapshot commit time; projectile identity is the projectile UUID and occurrence time
  is impact time. Both keep snapshot OnAttack damage inputs even if the source later dies, while only a live source
  releases OnAttack targeted effects/activations. OnHit is collected from the live target.
- General `ApplyDamage` has no basic occurrence or resonance policy. Producers are skill instant/tile-area/
  projectile/persistent-area leaves, periodic Buff ticks and CRA-POL-002 retaliation. Skill projectile/area source
  templates retain release-time source state and materialize deterministic per-target/hit roll identity at impact;
  the target remains live-read by the command executor.
- Phase 11/13 `dispatch_item_trigger_occurrence` remains the only occurrence classifier/assembler. Current-hit
  effects enter the formula; targeted/retaliation effects are batched before activations. Retaliation emits one
  owner-sourced non-recursive general `ApplyDamage` leaf with `include_on_attack_damage_effects = false`.
- Formula output still publishes `BattleCommand::UnitDied` in public `DamageResult.triggered_commands`, while every
  Core executor treats that command as a no-op because committed HP reaching zero owns death. The public shape and
  output are retained for unknown Rust consumers; it will not become a second lifecycle authority.
- `DamageContext.attacker_side`, `target_side`, `attacker_attack` and `target_max_hp`, plus
  `DamageResult.attacker_id`/`target_killed`, have no current Core semantic reader. They are public and therefore
  retained with an explicit compatibility disposition rather than narrowed.
- Boss runtime has no separate formula or ApplyDamage path. It releases ordinary AttackResolve/skill work and thus
  consumes the same leaf authorities. Runtime projectile/event carriers are non-serde battle-lifetime state; save,
  V3 and wire schemas do not embed the private execution plan.
- `game_server` forwards Core event/checkpoint shapes without direct Rust damage-carrier construction. Accessible
  Unity consumes `HpChanged` metadata (`damage_breakdown`, `final_damage`, `feedback_tags`) and `UnitDied`; exact
  event fields/order must remain unchanged.
- Live Equipment contains OnAttack current-hit/target effects and the CRA-POL-002 OnHit retaliation row; live Skill
  spans instant/projectile/tile-area damage and live Buff contains periodic damage. Their activation hashes remain
  unchanged and no data edit is indicated.

## 2026-08-10 — E04 Old-Production Focused Baseline

- No Phase 28 source or test mutation preceded this run.
- Pure damage formula: 22 passed, 0 failed, 0 ignored.
- Core command/damage/instant/projectile/death/retaliation family: 39 passed, 0 failed, 0 ignored.
- Skill instant/projectile/area delivery contract: 14 passed, 0 failed, 0 ignored.
- Live item trigger integration: 4 passed, 0 failed, 0 ignored.
- These nonzero green suites protect current arithmetic, detached source snapshots, live target defense, roll
  identity, general command metadata, death/provenance, trigger/retaliation and released skill behavior. A new
  cross-delivery characterization is still required and must pass on this old production before consolidation.

## 2026-08-10 — E05 Test-First Cross-Delivery Characterization

- Added three behavior-facing tests to the already-dirty `commands.rs`; no production code changed first.
- `instant_projectile_and_general_damage_share_snapshot_live_target_and_crit_contract`: 1 passed. It proves all
  three leaves retain launch-time base/source input, read changed target defense live, produce the same critical
  breakdown/raw/final/actual HP delta and keep basic-only dealt/received/release resonance.
- `committed_damage_uses_actual_overkill_delta_and_finalizes_one_causal_death`: 1 passed. It proves instant and
  projectile basic overkill grant dealt resonance from the actual 5 HP delta rather than planned 75 damage, and
  general damage returns one actual/one kill with exactly `HpChanged -> UnitDied`, causal killer and parent lineage.
- `general_damage_leaf_does_not_collect_basic_attack_or_hit_occurrences`: 1 passed. Source and target both own
  OnAttack/OnHit-capable equipment, but an Ability `ApplyDamage` leaf applies only its authored 20 damage, emits no
  proc/ability/retaliation and grants no basic resonance.
- The complete old-production command family is now 42 passed, 0 failed, 0 ignored. The test-first file hash is
  `2b7c09a901499ccddd00304cf6e92182d81c067ce3a8545ee4ada1d3afaf2d0e`.
- Expanded status remains 516 rows at the activation normalized hash because `commands.rs` was already a modified
  path. `git diff --check` is clean and the protected marker remains `614d2fbd...`.
- Production consolidation is now authorized. These tests must stay green without asserting a private helper name.

## 2026-08-10 — E06 Minimal Private Authority Consolidation

- Added private `CommittedDamagePolicy::{BasicAttack, GeneralCommand}` and `CommittedDamageInput` in
  `damage_runtime.rs`, visible only within the battle Core. The basic policy carries occurrence UUID and occurrence
  time explicitly; it is never inferred from `DamageSource`.
- `execute_committed_damage` now live-reads the active target's owner, defense, magic resistance, incoming
  modifiers and HP; reconstructs snapshot OnAttack damage inputs; dispatches OnAttack only for a live source and
  OnHit only for a basic policy; calls the retained pure `calculate_damage` exactly once; commits through
  `apply_damage_result_and_record`; and returns the existing actual-damage/kill execution summary.
- Basic dealt resonance is derived from the actual committed HP delta and remains before the HP commit in the
  established event sequence. Basic received resonance remains in the retained HP authority. The effect batches,
  retained formula `UnitDied` no-op batch and activation batches retain their established order.
- Instant basic and basic-projectile impact now construct the typed basic input and schedule autocasts after the
  common result. Their owners still retain release resonance, target/range acceptance, projectile movement,
  collision/miss/impact logging and impact time. General `ApplyDamage` uses only `GeneralCommand` and therefore
  cannot collect basic occurrences or resonance.
- Removed the superseded private `AttackTargetSnapshot`, `BasicAttackDamageSnapshot`,
  `BasicAttackDamageResolution` and `calculate_basic_attack_damage_snapshot`. No public formula, request, context,
  result, command, event, serde, save, DTO, RON, server or Unity surface was narrowed.
- Implementation hashes are `commands.rs`
  `47a371a85a5d42bc899d04fcb92523c8044200600e57e6c058ec1b980f10d1fc`, `damage_runtime.rs`
  `7b4d43792b45b08b0b3c872813059ff1ded233736e67e9c47ef11cdab154f7cb` and
  `basic_attack_projectile.rs` `54d1d95c72da0f17ab98d2f5cb700dbfbce457718a0b1b721335545db14258d1`.
- Static ownership search finds the three intended production callers only: instant basic and general command in
  `commands.rs`, plus projectile impact in `basic_attack_projectile.rs`. The only production call to
  `apply_damage_result_and_record` is inside the new authority. The projectile leaf no longer contains
  `TriggerOccurrenceContext`, `DamageContext`, `calculate_damage` or `RuntimeCommandBatch` assembly.
- The retained production formula call graph is one mutating call in `damage_runtime.rs` plus read-only preview
  calls in `target_usefulness.rs` and `tactical_stance.rs`. Searches for all removed private names return no match.
  Public `BattleCommand::UnitDied` and broad public damage fields remain compatibility-only and are not a second
  Core lifecycle authority.

## 2026-08-10 — E07 Focused, Broad And Compatibility Validation

- Post-change focused Core results are: command/damage family 42, formula 22, skill runtime 14, live item 4, live
  RON 25, Boss released-work 22 and result/death 7 passed; every suite has 0 failed and 0 ignored. The three new
  tests also pass individually, so their selected count is nonzero.
- Focused release results are command/damage 42, skill runtime 14 and Boss released-work 22 passed, all with 0
  failed and 0 ignored.
- Complete Core debug and release `cargo test --all-targets --quiet` each pass 1,070 tests: 1,019 library plus
  official integration suites of 4, 4, 25, 14 and 4. Every counted target has 0 failed and 0 ignored; zero-test
  binary targets are not counted as evidence.
- Complete server debug and release all-target tests each pass 59 tests with 0 failed and 0 ignored. Core and server
  debug/release all-target checks pass. Final `cargo fmt --all -- --check` and `git diff --check` pass.
- Non-strict Core Clippy exits 0 while reporting established warning debt (82 library and 135 library-test warnings,
  plus integration warnings); no warning names the new committed-damage authority. Server all-target Clippy exits 0
  on a concise rerun with warnings allowed after the first unbounded output was discarded.
- Formula, trigger, death and resonance authorities retain their activation hashes: `damage.rs` `e92d0377...`,
  `triggers.rs` `567bc636...`, `death_runtime.rs` `096a6826...` and `resonance_runtime.rs` `2fb54332...`.
  Skill delivery, live Equipment/Skill/Buff data and Core/rulebook/data/server/Unity canonical hashes also remain
  exactly at their E02 prospective values; no migration or canonical edit occurred.
- Expanded status remains exactly 516 rows at normalized C-sort SHA-256
  `8c67776899e009651dfb5e75110fbae4926c9344ce2e251f4e45be6de7f3c83d`; Phase edits stayed inside already-listed
  paths and the three initialized Goal documents. The protected marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730` with unchanged mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`.
- No Phase 29 symbol, placeholder or production edit was introduced. Implementation evidence is closed; only the
  mandatory independent completion-guide audit remains before any phase transition.

## Evidence Closure

- Activation, dirty-boundary, graph, caller, lifetime, consumer and compatibility evidence: E02–E03.
- Old-production baseline and characterization-first matrix: E04–E05.
- Minimal implementation and negative duplicate/public-narrowing evidence: E06.
- Focused/broad debug/release/static/data/canonical evidence: E07.
- Independent verdict: `Complete / High`, improvement class `none`, in the fresh 320-line
  `COMPLETION_AUDIT.md`.

## 2026-08-10 — E08 Phase Exit

- A separate completion pass reread the full 874-line authoring/execution guide, full 567-line completion-review
  guide, 198-line resume attachment, finalized Child documents, Master gate and current runtime/data/consumer
  evidence before issuing a verdict.
- Fresh audit-time results are commands 42, formula 22, live item 4, skill runtime 14, Boss runtime 22, result stats
  7 and RON loading 25 passed, all with 0 failed and 0 ignored. Formatting, scoped diff, authority searches, data/
  canonical hashes and dirty preservation remain green.
- The 320-line audit finds no material or cross-component defect and returns `Complete / High`, improvement class
  `none`. All P28-C01 through P28-C11 are pass; Phase 29 may become ready through the Master record-only transition.
- Adding the required audit explains the exact 516→517 expanded-row increase. The normalized C-sort SHA-256 is
  `9b5cf1cf53edd101028b3f18cf76d64d1faeafa248c85969089c892e4c515ef6`; the protected marker remains exact at
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`.
