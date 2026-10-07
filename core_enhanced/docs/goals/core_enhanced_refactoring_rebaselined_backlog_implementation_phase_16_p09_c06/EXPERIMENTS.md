# Phase 16 CRA-P09-C06 Experiments

Status: complete
Implementation Complete: true

## 2026-08-10 — E00 Activation And Required-Document Initialization

### Authorization And Dependency

- The user's Master execution request remains active and authorizes the single active Child.
- Phase 15 CRA-P09-C01 has a final `Complete / High` audit with improvement class `none`.
- Phase 14 CRA-P09-C02 terminal delivery is also complete; P09 publication/lifecycle characterization prerequisites
  are satisfied.
- CRA-P09-C06 is policy-neutral; CRA-POL-006 preserves all public/RON/save/wire shapes and current behavior.
- Phase 16 is next in the exact registry. Phases 17–44 remain pending and unauthorized.

### Historical Evidence Carried Into Initialization

- CRA-P09-C06 remains a behavior-preserving refactor with `3/1/1/2/2/2/unknown = 11/18`.
- Historical/re-audited defect: targeting, manual cast validation, preview, usefulness and area runtime independently
  adapt tile policy/pattern/facing/materialization/valid-board clipping.
- Target authority: one typed query with explicit membership and stable valid-cell materialization over captured
  origin/facing, preserving the distinction between cast reach and delivery footprint.
- Required first proof: cross-product parity for every policy/pattern/facing/edge-clipping case, including membership
  equals materialized-set containment and stable ordering.
- Phase 17 usefulness and Phase 18 preview corrections depend on this seam and must not be prebuilt.

### Initialization Result

- Required Child documents initialized: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`.
- Master Phase 16 coordination changes from `ready` to `active`; no other Child is active.
- Phase 16 production status/source/test/canonical investigation: not started.
- Test executions: none.
- Production/test/data/canonical mutation before initialization: none.

### Next Experiment

Read all three Child documents and the complete Phase 15 audit. Then capture expanded dirty status and prospective
target hashes before tracing current adapters, tests or canonical text.

## 2026-08-10 — E01 Required Reads And Dirty Baseline

### Read And Dependency Gate

- All 254 lines across the Phase 16 Plan, Experiments and Notes were fully read after Master registered Phase 16 as
  the sole active Child.
- The complete 227-line final Phase 15 audit was then reread. Its `Complete / High`, improvement `none` verdict and
  positive-scalar/publication seam remain current.
- Phase 17–44 remain pending; no later behavior or production work is authorized.

### Dirty Boundary

- HEAD remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded post-initialization status is 449 entries with normalized SHA-256
  `907b3b9734cd8aa73151e3ca24f8a895387b76779126c98499b8907357c93f83`.
- The increase from Phase 15's audit-inclusive 446 is exactly the three required Phase 16 Child documents; Master
  coordination files were already dirty.
- Prospective pre-source fingerprints:
  - tile range `89524297d86da66cc5ffe95cd5a5e6e0470169743952cb043c4ec81553840365`
  - targeting `18fc87a3a397103d5df9e95f2aeec864e1d6426e0b86b2bde55b1dc34bd39174`
  - range preview `36b8f87f55414a01649340018b4e08fec19d26d2d2bb6519a58f70753cb8e9cf`
  - target usefulness `88effbf9691c7a0ec40efa96335ec6d0f3a7bb9e5f9b7a6603493ea232562fdc`
  - skill area runtime `2a22998063d482cc37eeeb620f61c6fac20a63ff1e0c5085fad094858b6978ee`
  - battlefield field `66859782c24b0b2ad65ee5c2b95b344c00c75128e11419224d46422aa1336047`
  - skill runtime integration `afd09b3f6392e8c0175a10d5be3781638e6221c956a55e8c4d129f80c1b49c58`
  - RON integration `75558ff28be83cafcfe535252f552903b2077249c2cd81ef00b41d63195854b3`
  - target canonical `17e8f36d2ac72902e51fd522155217eec1948da7d7ada6a4ec4fbf1bca375f2d`.
- Tile range and battlefield field are clean. Targeting, preview, usefulness, skill area, both integration tests and
  the canonical are already dirty from user/campaign work. Existing hunks remain user-owned until reconciled.
- Historical `skill_runtime/sim.rs` no longer exists at that path; actual current skill simulation callers must be
  rediscovered rather than inferred.

### Next Experiment

Inspect existing scoped diffs first, then trace the actual current adapter/caller graph and run focused baselines
before adding the cross-product characterization.

## 2026-08-10 — E02 Current Graph, Overlap Reconciliation And Baselines

### Reconciled Producer/Caller Graph

- `battle/tile_range.rs`: `TileRangePattern::affected_tiles` validates, rotates, sorts by `(y, x)` and deduplicates;
  `contains_target_tile` rematerializes that geometry for membership.
- `battle/battlefield/field.rs`: `is_valid_tile` is the clipping predicate and `valid_positions` is the stable
  row-major whole-field projection. Static obstacles and occupancy are intentionally separate.
- `battle/core/targeting.rs`: basic attacks and automatic unit skills adapt the caster projected tile/facing and
  perform pattern/global membership.
- `battle/core/sim.rs`: explicit player-tile acceptance independently adapted the same membership, while action
  capability searched every valid battlefield tile.
- `range_preview.rs`: basic attack and active-skill previews independently materialized whole-field/pattern cells and
  clipped patterns to valid tiles.
- `battle/core/skill_runtime/area.rs`: instant and persistent TileArea delivery independently materialized and
  clipped delivery footprints.
- `battle/core/target_usefulness.rs`: automatic TileArea usefulness materialized the same footprint without first
  clipping it. Its observed targets remain equivalent because runtime targets are constrained to valid battlefield
  tiles; Phase 16 freezes that equivalence and does not change usefulness policy.
- Cast reach (`SkillTargetReach`/cast `target_range`) and delivery footprint (`DeliveryDef::TileArea.footprint`) are
  distinct producers even though both require the same projection mechanics.

### Existing Dirty Overlap

- Targeting's pre-existing hunk only extends a runtime-unit test fixture. Preview's existing hunks implement explicit
  cast selection. Usefulness's existing hunks update trigger/resource names. Area's existing hunks implement area
  source snapshots/facing persistence. All were retained and Phase 16 edits are separable.
- The current simulation file is `battle/core/sim.rs`, not historical `skill_runtime/sim.rs`. It already contained
  large campaign changes. A forensic pre-Phase-16 copy/reverse-patch comparison isolated the two initial Phase 16
  hunks and recovered its pre-Phase-16 SHA-256 as
  `050dc74b736d97796909ab1468b003018d5b07a269d651be880cbe53b239bde5`; the temporary copy was then removed.

### Baseline Results

- `cargo test -p game_core --lib tile_range`: 8 passed, 970 filtered.
- `cargo test -p game_core --lib targeting`: 21 passed, 957 filtered.
- `cargo test -p game_core --lib range_preview`: 8 passed, 970 filtered.
- `cargo test -p game_core --lib tile_area`: 9 passed, 969 filtered.
- The exploratory integration filter `manual` selected zero tests and was rejected as evidence. `--list` confirmed
  the integration target contains 14 tests; exact production unit filters were used later.

### Disposition

No user-visible caller disagreement or policy decision was found. The un-clipped usefulness adapter is an internal
representation difference with equivalent target results under the battlefield validity invariant, so the
behavior-preserving extraction may proceed.

## 2026-08-10 — E03 Green Characterization And Intended Red

### Pre-Extraction Characterization

- Added `range_projection_characterization_covers_policy_facing_clipping_and_membership` against the existing
  production preview resolver before introducing the new authority.
- The table covers pattern and whole-field policies, a directional asymmetric pattern across all four facings,
  anchor inclusion, center/edge/sparse-void clipping, row-major ordering and retained static-obstacle cells.
- For every inspected board tile it proves current valid-board membership equals containment in the materialized
  list. The exact filter selected 1 test and passed; an earlier `--exact` invocation selected zero and was rejected.

### Intended Red

- Added `typed_tile_reach_query_unifies_membership_and_stable_materialization` before production implementation.
- Exact nonzero filter failed compilation only because `TileReachQuery` did not exist at both call sites
  (`E0433`), establishing the missing typed authority rather than a fixture failure.

## 2026-08-10 — E04 Typed Authority And Consumer Migration

### Correction

- Added crate-private `TileReachQuery` beside `TileRangePolicy`/`TileRangePattern`. `try_new` validates and captures
  policy, optional pattern, anchor, facing and battlefield context. Whole-field intentionally requires no
  pattern/anchor/facing.
- `contains` applies valid-tile clipping before pattern/global membership. `materialize_valid_cells` returns the
  existing stable row-major pattern or whole-field cell list. Neither operation filters static obstacles,
  occupancy, line of sight, physics or path cost.
- Added `BattleCore::tile_reach_query_for_unit` as the sole runtime caster projected-tile/facing adapter. Global
  queries preserve the prior ability to operate without facing.
- Migrated basic/skill unit membership, explicit player-tile acceptance and capability materialization, preview
  projection, instant/persistent TileArea runtime, and TileArea usefulness to the authority.
- The usefulness adapter now obtains the same valid-cell footprint as runtime delivery before collecting targets;
  no target result changes under the valid-battlefield placement/movement invariant.
- Cast reach and delivery footprints remain distinct call-site inputs. No Phase 17 usefulness or Phase 18 preview
  meaning was introduced.

### First Green And Focused Regression

- Typed query exact filter: 1 passed, 979 filtered after the intended red.
- Pre-extraction characterization exact filter: 1 passed, 979 filtered after migration.
- Preview family: 9 passed, 971 filtered.
- Targeting family: 21 passed, 959 filtered.
- TileArea family: 9 passed, 971 filtered.
- Manual player unit/tile selection exact filter: 1 passed, 979 filtered.
- Whole-field skill targeting without facing exact filter: 1 passed, 979 filtered.
- Automatic TileArea no-useful-target exact filter: 1 passed, 979 filtered.
- `cargo fmt --all -- --check`: passed after two local formatting corrections.

## 2026-08-10 — E05 Integration And Current Dirty Checkpoint

### Integration Results

- `cargo test -p game_core --test skill_runtime_contract`: 14 passed.
- `cargo test -p game_core --test ron_loading`: 25 passed.
- `cargo test -p game_core --test live_skill_catalog_audit`: 4 passed.
- `cargo test -p game_core --test skill_test_suite`: 4 passed.
- No live RON, integration-test, battlefield-field or canonical file was changed by Phase 16.

### Static And Dirty Evidence

- Duplicate search leaves production `affected_tiles`/`contains_target_tile` calls only inside the typed authority;
  other hits are geometry self-checks or tests. Whole-field availability search now materializes through the query.
- `git diff --check` over all touched production/test targets passes.
- Expanded status is 450 entries with SHA-256
  `0efbf502ec0817950f7ae8f4368cfc0796c3b6e8b7183fae8e4dfeb0c59263c3`; the single increase from E01 is the
  previously clean `tile_range.rs`, exactly the new Phase 16 production authority.
- Current hashes: tile range
  `73930102fc59b03f028189681934de2859b2dad19e76d3622f4967b5f594e8aa`, targeting
  `44e9af5263125cf3a88282dba0c354065a4e58c6ac5eaf29f582308a08307abf`, simulation
  `432fabbd57659c8f4417dc3fec43082345f3981d442017ccc6659903da615602`, preview
  `e9e963f6e9b256d5713ad0cebc8d04060fe81043bebda70bc1f9de3e6c1c1a62`, usefulness
  `1b67efc7429c0c802674007b8b224ba9831cade04160ab6e7647ecba36c4da02`, and area
  `23e25cdfb7831dedc970912d531c20c669083bd36218f00af37cd0d9de9b8f42`.
- Battlefield field, skill runtime integration, RON integration and target canonical hashes remain byte-identical to
  E01. Public/serde/checkpoint/DTO/wire definitions were not edited.

### Next Experiment

Run complete debug/release Core and server validation, workspace static checks and final compatibility/diff audit.
Then update the nearest canonical, prepare completion evidence and perform the independent phase-exit review.

## 2026-08-10 — E06 Broad Validation, Compatibility And Audit Handoff

### Broad Validation

- Debug complete Core lib: 980 passed, zero failed/ignored/filtered.
- Release complete Core lib: 980 passed, zero failed/ignored/filtered.
- Release skill runtime integration: 14 passed.
- Release RON integration: 25 passed.
- Debug live item/skill activation integration: 4 passed.
- Debug game server all targets: 59 passed; zero-test binary/example harnesses completed successfully and are not
  counted as behavioral proof.
- `cargo check --workspace --all-targets`: passed.
- Final `cargo fmt --all -- --check` and repository-wide `git diff --check`: passed.

### Compatibility And Canonical Audit

- The new query/error are `pub(crate)`, have no serde derives and add no public Rust, RON, V3 checkpoint, DTO,
  message, server or Unity shape. Existing preview DTO construction is unchanged; only its cell producer changed.
- Battlefield layout and both integration files retain their E01 hashes. Live skill RON was not edited; current
  `game_resources/data/skills/base.ron` hash is
  `4087df054a663e20ddab14f184a206d532f8df7b472c656bbd708835298f1770`.
- `skill_target_contract.md` now records the one typed geometry mechanism, stable valid-cell semantics, intentional
  lack of obstacle/occupancy/path filters, and continued distinct ownership of cast reach versus delivery footprint.
  Its final hash is `b9d583f5042794ce72e2f56200aa684eaf18147f0d4399b3e3030e8e96f1a03d`.
- Expanded status remains 450 entries with unchanged name-only SHA-256
  `0efbf502ec0817950f7ae8f4368cfc0796c3b6e8b7183fae8e4dfeb0c59263c3`; no unexpected path entered the dirty set.
- Production duplicate search leaves direct geometry calls only inside `TileReachQuery`; remaining direct calls are
  `TileRangePattern` self-invariants or characterization tests.

### Phase Boundary

- Phase 17 still owns actual delivery-target usefulness policy; Phase 16 only gives it the shared valid footprint.
- Phase 18 still owns effect-preview union/footprint meaning; Phase 16 preserves current DTO cell assignment.
- Completion conditions P16-C01 through P16-C09 are evidenced. P16-C10 remains pending the required independent
  review under `goal_completion_review_guide.md`.

### Next Experiment

Read the completion guide in full, conduct the independent evidence/compatibility/dirty-worktree review, and create
`COMPLETION_AUDIT.md`. Do not activate Phase 17 unless the verdict is `Complete / High`.

## 2026-08-10 — E07 Independent Review Handoff

- All 567 lines of `core_enhanced/docs/goal_completion_review_guide.md` were reread after implementation, canonical
  update and final broad validation.
- Master Phase 16 assignment and the exact Stage B order were independently rechecked: CRA-P09-C06 must provide the
  typed parity seam before Phase 17 usefulness and Phase 18 preview corrections.
- Current production source, new cross-product tests, representative selector/manual-cast/area tests, canonical,
  duplicate search, compatibility boundary and expanded dirty state were re-inspected rather than accepting the
  implementation narrative as proof.
- Phase status is now `review_pending`; `Implementation Complete` remains false. The audit is read-only except for
  its own record and must deny Phase 17 activation unless every matrix row passes with long-term fit `High`.

## 2026-08-10 — E08 Completion Transition

- Independent phase-exit verdict: `Complete / High`; improvement class `none`; material finding none.
- Audit matrix P16-A01 through P16-A10 passes. Child completion matrix P16-C01 through P16-C10 is complete.
- Audit-inclusive expanded status is 451 entries with normalized SHA-256
  `4e497fb3edf5fc01967182509cf6a1cc014bbe2f7f983e00ea83d774e26cbb34`.
- Phase 16 transitions from `review_pending` to `complete`; `Implementation Complete` changes to true.
- Master may mark Phase 16 `complete — Complete / High` and move Phase 17 CRA-P09-C03 to `ready`. No Phase 17
  production work is authorized until its historical/policy evidence is reviewed and required Child documents are
  initialized, registered active and fully read with this final audit.
