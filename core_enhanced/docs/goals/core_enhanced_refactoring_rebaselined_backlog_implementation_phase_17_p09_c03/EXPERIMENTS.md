# Phase 17 CRA-P09-C03 Experiments

Status: complete
Implementation Complete: true

## 2026-08-10 — E00 Activation And Required-Document Initialization

### Authorization And Dependency

- The user's Master execution request remains active and authorizes the single active Child.
- Phase 16 CRA-P09-C06 has a final `Complete / High` audit with improvement class `none`.
- CRA-POL-004 approves any-step cast admission and current-step execution over actual delivery targets.
- CRA-POL-006 preserves all public/RON/save/wire shapes unless this Goal explicitly stops for policy.
- Phase 17 is next in the exact registry. Phases 18–44 remain pending and unauthorized.

### Historical Evidence Carried Into Initialization

- CRA-P09-C03 remains a correctness correction with `3/2/2/3/2/2/unknown = 14/18`.
- EnemySingle TileArea acquisition historically evaluates the anchor rather than the materialized/clipped/filtered
  delivery set; a schema-valid anchor-excluding footprint can therefore invert admission versus execution.
- The former first-hostile-step approximation cannot implement the approved any-step/current-step split.
- Current live EnemySingle TileArea footprints include anchors, so the correction must retain current live choices.
- Required first proof: anchor-excluding actual-set selection/cast/execution table and useful-step-order lifecycle table.
- Phase 18 preview union/content work must not be prebuilt.

### Initialization Result

- Required Child documents initialized: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`.
- Master Phase 17 coordination changes from `ready` to `active`; no other Child is active.
- Phase 17 production status/source/test/data/canonical investigation: not started.
- Test executions: none.
- Production/test/data/canonical mutation before initialization: none.

### Next Experiment

Read all three Child documents and the complete final Phase 16 audit. Then capture expanded dirty status and
prospective target hashes before tracing current usefulness or delivery-target code.

## 2026-08-10 — E01 Required Reads And Dirty Baseline

### Read And Dependency Gate

- All 264 lines across the Phase 17 Plan, Experiments and Notes were fully read after Master registered Phase 17 as
  the sole active Child.
- The complete final 242-line Phase 16 audit was then reread. Its `Complete / High`, improvement `none` verdict and
  typed membership/materialization seam remain current.
- CRA-POL-004 and the original/re-audited candidate, test, boundary and exact-order records were reread. Phase 18–44
  remain pending and no later production behavior is authorized.

### Dirty Boundary

- HEAD remains `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Expanded post-initialization status is 454 entries with normalized SHA-256
  `db4cb68b213cef6e96191066a68e9164b5e166903f0e090c71e7a2fbeda36aa5`.
- The increase from Phase 16's audit-inclusive 451 is exactly the three required Phase 17 Child documents; Master
  coordination files were already dirty.
- Prospective pre-source fingerprints:
  - target usefulness `1b67efc7429c0c802674007b8b224ba9831cade04160ab6e7647ecba36c4da02`
  - skill simulation `432fabbd57659c8f4417dc3fec43082345f3981d442017ccc6659903da615602`
  - skill cast runtime `e8356dddee42726f862c5efae8f779572b32cb5b13078c532bd80271769d8b0d`
  - skill area runtime `23e25cdfb7831dedc970912d531c20c669083bd36218f00af37cd0d9de9b8f42`
  - Core test module `c6255583aa6af6bc1c4fec782f31b0199ec336f30831a5b87e91adeff00e2002`
  - typed tile range `73930102fc59b03f028189681934de2859b2dad19e76d3622f4967b5f594e8aa`
  - skill runtime integration `afd09b3f6392e8c0175a10d5be3781638e6221c956a55e8c4d129f80c1b49c58`
  - RON integration `75558ff28be83cafcfe535252f552903b2077249c2cd81ef00b41d63195854b3`
  - target canonical `b9d583f5042794ce72e2f56200aa684eaf18147f0d4399b3e3030e8e96f1a03d`
  - live Skill RON `4087df054a663e20ddab14f184a206d532f8df7b472c656bbd708835298f1770`.
- Skill cast runtime is clean. Usefulness, simulation, area, Core tests, tile range, both integrations, canonical and
  live Skill RON are already dirty from user/campaign/earlier Phase work. Existing hunks remain user-owned until
  reconciled.
- Scoped diff inspection confirms the Phase 16 query integration is separable in usefulness/area. Simulation and
  Core test modules contain large campaign changes; relevant hunks must be isolated before Phase 17 edits.

### Next Experiment

Trace the live usefulness/admission/execution and runtime delivery-target graph, then run nonzero focused baselines
before adding the CRA-POL-004 decision tables.

## 2026-08-10 — E02 Live Graph And Failing Characterization

### Runtime Graph

- Automatic acquisition passes one `usefulness_step` into `choose_skill_target_by_rule`; candidate filtering therefore
  evaluates the selected anchor against one step rather than evaluating the skill over delivery sets.
- Manual EnemySingle validation reaches `skill_has_useful_automatic_cast_target`, but that function also evaluates an
  `EnemySingle` TileArea step against the anchor before inspecting its delivery.
- TileArea execution already materializes valid cells, applies `hit_targets`/`include_caster`, sorts unit UUIDs and
  gates the current step. Its gate incorrectly becomes disabled for an empty hostile target set.
- Direct/target-locked delivery uses the selected unit as its actual target. Targetless bounded directional projectile
  admission remains its existing explicit executable exception.
- The pre-cast area resolver can derive a CastTarget anchor directly from a unit target at `time_ms`; no active cast
  state or effect application is required for predictive actual-set evaluation.

### Tests First — Red

- Added three decision-table characterizations in the Core test module:
  - useless selected anchor but a useful enemy in an anchor-excluding materialized footprint;
  - useful selected anchor but no enemy in the filtered footprint;
  - first hostile step ineffective and later hostile step useful, plus the reverse order.
- `cargo test cast_usefulness_ -- --nocapture` compiled and ran exactly 3 tests; all 3 failed at their intended
  behavioral assertions (`980 filtered out`).
- Observed inversions were `None` instead of the actual-area anchor, `Some(anchor)` instead of empty-area rejection,
  and `None` instead of later-step admission. This is the expected CRA-P09-C03 red state, not an infrastructure error.

### Next Experiment

Introduce one typed usefulness mode for candidate selection, one side-effect-free current-step actual-delivery query,
and any-step cast aggregation. Then make the three red characterizations green before expanding the matrix.

## 2026-08-10 — E03 Actual-Delivery Authority And Green Matrix

### Implementation

- Replaced the nullable first-step approximation with typed candidate-usefulness modes:
  - `Cast { time_ms, skill }` evaluates every hostile step for candidate admission;
  - `DeliveryStep { time_ms, step_index, step }` evaluates only the current step during retargeting.
- Added one side-effect-free actual-delivery evaluator. Direct and TargetLocked delivery use the unit target; TileArea
  uses the Phase 16 valid-cell materializer, authored hit filter/caster inclusion and stable UUID-sorted units.
- Automatic acquisition now supplies the authoritative event/readiness time. The test-only zero-time wrapper preserves
  existing unit-test ergonomics without becoming production authority.
- Empty `Enemies`/`Any` TileArea sets now retain the hostile usefulness gate instead of disabling it merely because no
  hostile unit survived filtering.
- TargetLocked projectile release applies the same current-step hostile usefulness decision. A no-op projectile step
  records no launch but does not cancel a later useful step.
- Step-context resolution now receives authoritative time and step index so `RetargetOnStep` can use the actual current
  delivery rather than an anchor-local approximation.
- Updated the combat-domain canonical with the any-step cast/current-step execution split, actual-set pipeline,
  anchor non-authority, manual/automatic parity and unchanged shape/tie boundaries.
- No public type, RON field, live RON row, save/checkpoint field, event payload or wire DTO changed.

### Expanded Decision Matrix

- The original 3 red cases are green.
- Added friendly/enemy filter behavior, corner clipping with exactly three valid footprint cells, manual PlayerUnit
  admission/rejection through the public validator and actual lifecycle proof.
- Added a TargetLocked no-op lifecycle case: the ineffective first step creates no projectile while the later stun
  executes. The direct first-step no-op likewise leaves HP unchanged and does not cancel the later stun.

### Test Results

- `cargo test cast_usefulness_ -- --nocapture`: 4 passed, 0 failed (`981 filtered out`).
- `cargo test useless_projectile_step_is_a_noop_without_cancelling_later_delivery -- --nocapture`: 1 passed, 0 failed
  (`984 filtered out`).
- `cargo test --lib`: 985 passed, 0 failed, 0 ignored, 0 measured, 0 filtered out.
- `cargo fmt --all -- --check` initially reported only the new scoped hunks; `cargo fmt --all` normalized those hunks.

### Next Experiment

Run focused legacy targeting/area/projectile tests, debug integration/live-catalog tests, release parity, server/workspace
checks and post-change shape/data/diff evidence. Then complete the Child checklist and perform the independent audit.

## 2026-08-10 — E04 Final Validation, Compatibility And Audit Handoff

### Final Behavioral Evidence

- Final focused actual-set filter: 5 passed, zero failed; it covers automatic selection, public manual PlayerUnit and
  PlayerTile validation, both step orders, `Allies`/`Enemies`/`Any`, caster inclusion and corner clipping.
- TargetLocked current-step lifecycle: 1 passed; no projectile launch was recorded for the ineffective first step and
  the later stun executed.
- Final complete Core lib: debug 986/986 and release 986/986.
- Final debug integrations: skill runtime 14/14, RON 25/25, live catalog 4/4, skill suite 4/4 and live item 4/4.
- Final release integrations: skill runtime 14/14 and RON 25/25.
- Final game server: 59/59; zero-test binary/doc harnesses completed and are not counted as behavioral proof.
- `cargo check --workspace --all-targets`, `cargo fmt --all -- --check` and repository-wide `git diff --check` pass.

### Static And Compatibility Evidence

- `skill_cast_target_usefulness_step` and `usefulness_step` have zero production matches. Candidate selection now
  receives only typed cast/current-delivery intent; direct hostile per-unit classification remains downstream inside
  the actual-delivery evaluator and direct current-step execution.
- No Phase 18 range-preview/effect-preview implementation was added. Area geometry and Phase 16 tile query retain
  activation hashes `23e25cdf...` and `73930102...` respectively.
- Integration sources retain activation hashes `afd09b3f...` and `75558ff...`.
- Live Skill RON remains byte-identical at
  `4087df054a663e20ddab14f184a206d532f8df7b472c656bbd708835298f1770`; no data rewrite occurred.
- Final canonical hash is `c559737e9f3e12685f0a40be0536a209fe82e45b13c574440cbea2d0806ee325`.
- Final production/test hashes are usefulness `73e90a5c...`, cast `8c586e5a...`, projectile `07dd8844...`, simulation
  `9d4eecc6...` and Core tests `4aab68d4...`.
- Projectile runtime entered the prospective set only after current-step lifecycle inspection exposed the
  TargetLocked release gap. Its Phase 17 hunk is isolated to time/step-aware context and the hostile no-op gate;
  removing only that hunk into `/tmp` reconstructs pre-Phase-17 hash `a100b805...`, while unrelated pre-existing
  kill-stop/provenance hunks remain untouched.
- Expanded dirty status is 456 entries with normalized name/status SHA-256
  `a322af44589c1dc55771bebc2c50c26766ac9fed3b1319d2a22770034a302d16`. No reset, restore, clean, stash or deletion
  was used; existing campaign/user hunks remain present.
- `SkillTargetUsefulness` is crate-private and has no serde. This Phase adds no public Rust, RON, V3 checkpoint,
  event, DTO, server-message or Unity wire field.

### Phase Boundary

- P17-C01 through P17-C09 are complete. P17-C10 remains pending the required independent review.
- Phase 18 remains pending and unauthorized; its preview union/content correction is not present here.
- Status transitions to `review_pending`; `Implementation Complete` remains false until a `Complete / High` audit.

### Next Experiment

Reread all 567 lines of `goal_completion_review_guide.md`, independently inspect the evidence/source/tests/canonical
and dirty boundary, then create `COMPLETION_AUDIT.md`. Do not activate Phase 18 unless the verdict is `Complete / High`.

## 2026-08-10 — E05 Independent Audit Finding And Correction Cycle

- All 567 lines of `core_enhanced/docs/goal_completion_review_guide.md` were reread after final implementation and
  validation.
- The independent pass reconfirmed Master Phase 17/18 dependency, CRA-POL-004, current source/callers, focused
  lifecycle tests, compatibility/static evidence and the nearest canonical rather than accepting E04 as proof.
- One canonical drift was found: runtime and tests apply actual-delivery usefulness to PlayerTile manual admission,
  while the canonical's older activation sequence could still be read as PlayerTile requiring only a valid in-range
  tile. The newer usefulness subsection named PlayerUnit parity but also omitted PlayerTile.
- This is a documentation correction inside the authorized canonical scope, not a policy question or production
  defect. The first audit is stopped without a verdict; status temporarily becomes `correction_required`.
- Correction plan: state that PlayerTile's selected anchor need not be occupied but hostile cast admission evaluates
  its materialized/filtered delivery set, then rerun canonical/static checks and restart the entire 567-line audit.

## 2026-08-10 — E06 Canonical Correction Verified And Reaudit Handoff

- `skill_target_contract.md` now explicitly applies the shared any-step actual-delivery gate to PlayerUnit and
  PlayerTile. It distinguishes an unoccupied selectable tile anchor from the hostile delivery set, which must contain
  a useful executable target.
- No production source/test/data changed during the correction. Production hashes remain usefulness `73e90a5c...`,
  cast `8c586e5a...`, projectile `07dd8844...`, simulation `9d4eecc6...` and Core tests `4aab68d4...`.
- Corrected canonical hash is `1b24581ec7d2fed7d057ffb0e4140a547934b146ef3082493e01b51331fe66b0`;
  live Skill RON remains `4087df05...`.
- `cargo fmt --all -- --check` and repository-wide `git diff --check` pass.
- Expanded status remains 456 entries with unchanged normalized SHA-256 `a322af44...`; the correction changed only
  content of an already-dirty authorized canonical.
- Status returns to `review_pending`. The completion guide must now be reread in full and all evidence re-audited;
  the stopped first pass is not reused as a verdict.

## 2026-08-10 — E07 Completion Transition

- The 567-line completion guide was reread from the beginning after the canonical correction, and the Phase 17
  evidence, implementation, tests, data/shape boundary, canonical and dirty-worktree scope were independently
  re-audited.
- `COMPLETION_AUDIT.md` records all ten completion rows passing and the combined phase-exit verdict
  `Complete / High`; improvement class is `none` and no immediate correction, policy question or follow-up is
  required for the verdict.
- P17-C10 is complete. Phase 17 transitions from `review_pending` to `complete`, and `Implementation Complete`
  becomes true.
- The audit-artifact-inclusive expanded status contains 457 entries with normalized SHA-256
  `ac33d2fbddbb2cd12e3a5a0f985f3bb5ef8589bcb0c8734b9c86008df98dc960`.
- Phase 18 remains unauthorized until the Master records this result and initializes, registers and fully reads its
  three required Child documents under the exact linear protocol.
