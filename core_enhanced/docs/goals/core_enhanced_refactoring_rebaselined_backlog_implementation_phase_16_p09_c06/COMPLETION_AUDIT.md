# Phase 16 CRA-P09-C06 Completion Audit

Audit Mode: phase_exit
Audit Target: Phase 16 CRA-P09-C06 — Centralize Typed Tile Reach Queries And Projections
Current Status At Audit Start: `review_pending`
Implementation Complete Claim At Audit Start: `false`
Audit Date: 2026-08-10 (`Asia/Seoul`)
Policy Source: policy-neutral behavior-preserving refactor; CRA-POL-006 compatibility boundary
Goal Documents: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
Master: `../core_enhanced_refactoring_rebaselined_backlog_implementation_master/PLAN.md`
Direct Predecessor: Phase 15 CRA-P09-C01, `Complete / High`
Repository: `/mnt/f/work/simulator`
Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
Dirty State At Audit Start: 450 expanded entries; normalized status SHA-256
`0efbf502ec0817950f7ae8f4368cfc0796c3b6e8b7183fae8e4dfeb0c59263c3`
Completion Guide: all 567 lines of `core_enhanced/docs/goal_completion_review_guide.md` reread immediately
before this independent audit

## Audit Scope And Approved Meaning

CRA-P09-C06 is a behavior-preserving internal refactor. One typed query must own pattern/global tile membership and
stable valid-cell materialization over captured battlefield, anchor and facing context. Basic/skill target selection,
explicit player-tile acceptance and capability, range preview, TileArea runtime and TileArea usefulness must consume
that authority instead of independently adapting geometry.

The approved meaning preserves every current pattern transform, anchor-inclusion rule, row-major order, board/void
clipping and intentional inclusion of obstacle/occupied cells. It must not merge cast acquisition
`reach + target_range` with `DeliveryDef::TileArea.footprint`, add spatial filters, change public/RON/save/wire shape,
or prebuild Phase 17 usefulness or Phase 18 effect-preview policy.

## Completion Evidence Matrix

| ID | Completion condition or invariant | Policy basis | Runtime/data/contract evidence | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- |
| P16-A01 | One typed membership/materialization authority exists | CRA-P09-C06 target authority | crate-private `TileReachQuery` captures battlefield plus validated Pattern or WholeField state and exposes `contains` and `materialize_valid_cells` | typed query intended red then 1/1 green; source/duplicate audit | pass |
| P16-A02 | Pattern/global/facing/anchor/clipping/order parity is exact | behavior-preserving scope | Pattern delegates to existing validated rotation/sort/dedup; WholeField delegates to existing row-major `valid_positions`; both share `is_valid_tile` clipping | production characterization 1/1 before and after extraction across both policies, four facings, anchor inclusion, edge and sparse void | pass |
| P16-A03 | Selector and explicit cast membership use the authority | Phase objective | `is_target_in_tile_range_policy` and `is_tile_in_skill_reach` obtain `TileReachQuery` through the sole runtime caster adapter | targeting 21/21; manual selection 1/1; whole-field skill 1/1; Core debug/release 980/980 | pass |
| P16-A04 | Preview/capability/area/usefulness materialize through the authority | Phase objective | preview resolver, player-tile capability, `valid_tile_area_tiles`, instant/persistent area and usefulness share query materialization | preview 9/9; TileArea 9/9; usefulness exact 1/1; skill runtime debug/release 14/14 | pass |
| P16-A05 | Obstacles/occupancy/LOS/path filters remain absent | preserved invariants | query clips only with `BattlefieldLayout::is_valid_tile`; it never reads static obstacles, unit occupancy, physics or path state | characterization retains a static-obstacle cell; existing obstacle preview test and full Core suites pass | pass |
| P16-A06 | Cast reach and delivery footprint remain distinct and later policy is not prebuilt | Master Stage B dependency | cast consumers pass `SkillTargetReach + target_range`; area consumers pass `TileArea.footprint`; DTO cell assignment and usefulness decision logic are unchanged | static caller inspection, integration suites, exact Phase 17/18 owner comparison | pass |
| P16-A07 | Public/RON/save/wire/event shape remains exact | CRA-POL-006 | query/error are `pub(crate)` with no serde; no public DTO, checkpoint, server message, event or Unity protocol type changed by Phase 16 | RON debug/release 25/25; server 59/59; shape/diff inspection | pass |
| P16-A08 | Production live data and canonical remain aligned | compatibility and completion guide §12 | live Skill RON is unchanged; `skill_target_contract.md` records shared mechanics and separate source ownership | live catalog 4/4, live item 4/4, live RON hash and canonical comparison | pass |
| P16-A09 | Dirty user/campaign work is preserved and scoped | Goal invariant | pre-existing targeting/preview/usefulness/area/simulation hunks remain; only one clean production path entered status; no reset/restore/clean/stash | activation/final hashes, forensic simulation reverse-diff, repository-wide `git diff --check` | pass |
| P16-A10 | Phase 17 receives the required trustworthy seam | Master exact order | all relevant consumers now select their typed source and share query mechanics; no compatibility shim or duplicate production projection remains | duplicate search, complete Core/integration/server/release checks | pass |

## Runtime And State Evidence

- `TileRangePattern` remains geometry authority: validation, facing rotation, anchor inclusion, stable `(y, x)` sort
  and dedup are unchanged. `TileReachQuery` validates the borrowed pattern once, captures immutable anchor/facing and
  applies current battlefield validity to both membership and materialization.
- `BattleCore::tile_reach_query_for_unit` is the sole runtime adapter from unit identity to projected tile/facing.
  Pattern queries require the same source body and facing as before. Whole-field queries intentionally require
  neither, preserving global targeting for facingless units.
- `is_target_in_tile_range_policy` still obtains the live target body and tests its projected tile. Air, lifecycle,
  hostility, usefulness and target-rule checks remain outside the geometry query and are unchanged.
- Explicit `PlayerTile` activation and readiness now use the same membership/materialized set. Activation still
  validates the concrete payload; readiness still asks whether any concrete valid payload passes the existing
  activation predicate. Costs, focus, resonance, revision and mutation ordering were not changed.
- Instant and persistent TileArea execution use `valid_tile_area_tiles`, which now delegates only its projection to
  the query. Anchor resolution, stationary/facingless policy, tracking, target filters, tick cadence, source damage
  snapshot and event ordering remain outside and unchanged.
- Usefulness previously passed an un-clipped footprint to target collection while runtime delivery passed a clipped
  footprint. Gameplay units are constrained to valid battlefield cells, so observed targets agreed; the full suite
  and explicit area/usefulness tests confirm the shared clipped list preserves results. Phase 17 still owns whether
  usefulness should reason over different delivery steps or targets.
- No command input, authoritative state mutation, pause/time scale, retry, rollback, checkpoint, reconnect or stale
  event behavior changed in this internal geometry refactor.

## Data Evidence

- Phase 16 adds no data field, preset, schema, loader or validation rule. `TileRangePolicy`, `FacingDirection` and
  `TileRangePattern` serde derives/fields remain unchanged.
- Production `game_resources/data/skills/base.ron` was not edited and hashes to
  `4087df054a663e20ddab14f184a206d532f8df7b472c656bbd708835298f1770`.
- `core_enhanced/tests/ron_loading.rs` remains byte-identical to activation at
  `75558ff28be83cafcfe535252f552903b2077249c2cd81ef00b41d63195854b3`; its 25 production-data tests pass in
  debug and release.
- No live rewrite, migration, default compatibility or dual schema is necessary.

## DTO, Transport And Persistence Evidence

- `TileReachQuery` and `TileReachQueryError` are crate-private and carry no serialization derives. Existing public
  tile-range data types are unchanged.
- Preview still constructs the same `LiveBattleBasicAttackRangePreviewDto` and
  `LiveBattleActiveSkillRangePreviewDto`, with the same availability/reason/source/targeting-kind and cast/effect cell
  assignment. Only duplicate cell production is replaced.
- No checkpoint envelope, active-run state, event-log type, server request/message mapping, JSON tag/nullability or
  Unity protocol file was edited by Phase 16. No save version bump or rollout coordination is applicable.
- Game server all-target tests pass 59/59, including actual Core DTO-to-server mapping tests. A WebSocket/Unity probe
  is not required because serialized shape and ordering are unchanged.

## Consumer Evidence

- Confirmed workspace consumers: basic attack selection, automatic skill target acquisition/retargeting, explicit
  player-tile validation, player skill readiness, basic/active-skill range preview, instant/persistent TileArea
  runtime and automatic TileArea usefulness.
- Confirmed workspace data/contract boundary: live Skill RON and `skill_target_contract.md`.
- Canonical external consumer: Unity consumes Core-computed preview/event cells and does not recompute authored
  geometry. That repository was not accessed in this Phase; no external shape or ordering changed, so it is not a
  completion dependency under CRA-POL-006.
- Inaccessible/unknown required consumer: none.
- Explicitly excluded: Boss navigation/path cost, projectile movement, Phase 17 usefulness policy and Phase 18
  effect-preview meaning.

## Legacy And Compatibility Audit

- Production search for `affected_tiles` and `contains_target_tile` leaves direct calls only inside
  `TileReachQuery`. Other occurrences are `TileRangePattern` self-invariants or tests.
- Whole-field projection in the migrated range/cast/area surfaces flows through the query. Battlefield
  `valid_positions` remains the underlying stable board primitive rather than a competing reach adapter.
- No new alias, fallback, dual-read/write, debug bypass, ignored test, placeholder state or compatibility shim was
  added. Missing pattern/anchor/facing and invalid patterns remain fail-closed.
- Legacy continuous-shape area and step-local range semantics remain absent and were not reintroduced.
- Material legacy/compatibility finding: none.

## Scoped Diff And Dirty Preservation

- Activation had 449 expanded entries. The implementation checkpoint and audit start have 450; the only new dirty
  production path is previously clean `core_enhanced/src/game/battle/tile_range.rs`, exactly where the typed
  authority belongs.
- Existing targeting changes were limited to a test fixture; preview to explicit cast selection; usefulness to
  trigger/resource changes; area to damage snapshot/facing persistence. Phase 16 integrated around and retained
  those hunks.
- `battle/core/sim.rs` was discovered after activation and was already heavily dirty. A temporary forensic copy with
  only Phase 16 hunks reversed recovered the pre-Phase-16 hash
  `050dc74b736d97796909ab1468b003018d5b07a269d651be880cbe53b239bde5` and confirmed separability; the temporary
  file was removed.
- Final relevant hashes before this audit record: tile range
  `73930102fc59b03f028189681934de2859b2dad19e76d3622f4967b5f594e8aa`, targeting
  `44e9af5263125cf3a88282dba0c354065a4e58c6ac5eaf29f582308a08307abf`, simulation
  `432fabbd57659c8f4417dc3fec43082345f3981d442017ccc6659903da615602`, preview
  `e9e963f6e9b256d5713ad0cebc8d04060fe81043bebda70bc1f9de3e6c1c1a62`, usefulness
  `1b67efc7429c0c802674007b8b224ba9831cade04160ab6e7647ecba36c4da02`, area
  `23e25cdfb7831dedc970912d531c20c669083bd36218f00af37cd0d9de9b8f42`.
- Battlefield field and integration-test hashes remain identical to activation. No file was reset, restored, cleaned,
  stashed, moved or deleted; no broad formatter was run. Repository-wide diff check is clean.

## Canonical Alignment

- `skill_target_contract.md` remains the nearest targeting/delivery canonical. It now states that one typed internal
  query owns shared membership/materialization mechanics, clips only to valid battlefield cells, retains obstacles,
  and keeps cast reach distinct from delivery footprint.
- The canonical still assigns Core-computed final cells to Unity, global reach only to unit acquisition and
  `TileArea.footprint` to actual affected tiles. That matches current code and does not claim Phase 17/18 behavior.
- Final canonical hash before the audit record is
  `b9d583f5042794ce72e2f56200aa684eaf18147f0d4399b3e3030e8e96f1a03d`.
- No canonical conflict or goal-document-only ownership remains.

## Validation Results

- Baseline `cargo test -p game_core --lib tile_range`: 8 passed, 0 failed, 970 filtered.
- Baseline `cargo test -p game_core --lib targeting`: 21 passed, 0 failed, 957 filtered.
- Baseline `cargo test -p game_core --lib range_preview`: 8 passed, 0 failed, 970 filtered.
- Baseline `cargo test -p game_core --lib tile_area`: 9 passed, 0 failed, 969 filtered.
- Pre-extraction characterization exact nonzero filter: 1 passed, 0 failed, 978 filtered.
- Proposed typed query exact nonzero filter before implementation: compile failed with two `E0433` missing-type
  errors, the intended structural red.
- Corrected typed query exact nonzero filter: 1 passed, 0 failed, 979 filtered.
- Corrected characterization exact nonzero filter: 1 passed, 0 failed, 979 filtered.
- Corrected preview family: 9 passed, 0 failed, 971 filtered.
- Corrected targeting family: 21 passed, 0 failed, 959 filtered.
- Corrected TileArea family: 9 passed, 0 failed, 971 filtered.
- Manual player unit/tile selection exact filter: 1 passed, 0 failed, 979 filtered.
- Whole-field skill without facing exact filter: 1 passed, 0 failed, 979 filtered.
- Automatic TileArea usefulness exact filter: 1 passed, 0 failed, 979 filtered.
- `cargo test -p game_core --test skill_runtime_contract`: 14 passed, 0 failed.
- `cargo test -p game_core --test ron_loading`: 25 passed, 0 failed.
- `cargo test -p game_core --test live_skill_catalog_audit`: 4 passed, 0 failed.
- `cargo test -p game_core --test skill_test_suite`: 4 passed, 0 failed.
- `cargo test -p game_core --test live_item_skill_activation`: 4 passed, 0 failed.
- `cargo test -p game_core --lib`: 980 passed, 0 failed/ignored/filtered.
- `cargo test --release -p game_core --lib`: 980 passed, 0 failed/ignored/filtered.
- `cargo test --release -p game_core --test skill_runtime_contract`: 14 passed, 0 failed.
- `cargo test --release -p game_core --test ron_loading`: 25 passed, 0 failed.
- `cargo test -p game_server --all-targets`: 59 behavioral tests passed; binary/example harnesses selected zero and
  are not counted as behavior evidence.
- `cargo check --workspace --all-targets`: passed; compile-only evidence is static coverage, not behavior proof.
- `cargo fmt --all -- --check`: passed.
- `git diff --check`: passed repository-wide.

## Baseline Failures

- No nonzero baseline behavior test failed.
- Exploratory integration filter `manual` and the first characterization invocation with an incomplete `--exact`
  name selected zero tests. Both were rejected as evidence and replaced by nonzero exact/family runs.
- No test is ignored, suppressed or left failing.

## Cross-Component Findings

- Selector, manual cast, preview and area paths now share mechanics without competing source ownership.
- Preview DTO and TileArea event consumers still receive their existing respective cast/effect cells; Phase 18 may
  change preview meaning later without replacing the query seam.
- Usefulness now receives the same valid footprint as delivery, creating the intended Phase 17 seam without deciding
  which step/delivery targets should govern usefulness.
- No schema/version/save/server assumption conflicts with the internal query because no serialized boundary moved.
- Material cross-component finding: none.

## Long-Term Direction Review

| Candidate ID | Work type | Current→target authority result | Preserved invariant result | Removal/consolidation result | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| CRA-P09-C06 | behavior-preserving refactor | repeated policy/pattern/facing/valid-board adapters → one crate-private typed query plus one runtime caster adapter | geometry, ordering, clipping, obstacles, public/data/save/wire and cast-vs-delivery ownership preserved | duplicate production membership/materialization loops removed; no shim or later-policy placeholder | intended red, cross-product, production families, Core debug/release, integrations/server/static | pass |

- Fit: `High`.
- Improvement class: `none`.
- Reason: the authority is placed beside tile-range geometry, borrows existing domain state instead of duplicating it,
  exposes only the two required operations and is reused by every in-scope consumer. It reduces drift without a
  public abstraction or serialized migration, retains distinct typed source inputs, and gives Phases 17/18 a stable
  seam without preimplementing their policy. Tests protect cross-product geometry and production consumers rather
  than a helper layout alone.

## Primary Verdict

- `Complete`.
- Every approved completion condition and invariant has current runtime/contract evidence and nonzero focused/broad
  validation. No material legacy, compatibility shim, unverified required consumer, canonical drift, dirty-worktree
  violation, scope expansion or unresolved failure remains.

## Status Transition

- From: `review_pending`, `Implementation Complete: false`.
- To: `complete`, `Implementation Complete: true`.
- Master Phase 16 registry: may transition to `complete — Complete / High`.
- Next Phase activation: allowed only after this audit's final record-only refresh and Master coordination update.

## Remaining Risk And Required Action

- Residual risk is low: `TileReachQuery::contains` retains the existing linear pattern materialization cost per
  membership check. Performance was not a defect in this candidate and no behavior or scale regression appeared in
  complete debug/release runs; optimizing it would be a separate measured refactor, not an exit blocker.
- No immediate correction, policy question, compatibility migration or follow-up refactor is required.
- After recording audit-inclusive status/hash, update Child/Master status and activate only Phase 17 according to the
  exact linear registry.

## Audit Record Integrity

- Adding this audit record changes expanded status from 450 to 451 entries; the only new path is
  `COMPLETION_AUDIT.md` under the already scoped Phase 16 Child.
- Audit-inclusive normalized status SHA-256 is
  `4e497fb3edf5fc01967182509cf6a1cc014bbe2f7f983e00ea83d774e26cbb34`.
- Repository-wide `git diff --check` and `cargo fmt --all -- --check` both pass after the implementation/canonical
  state used by this review. The audit record contains no production mutation.
