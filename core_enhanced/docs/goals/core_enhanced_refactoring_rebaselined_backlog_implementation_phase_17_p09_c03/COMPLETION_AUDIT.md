# Phase 17 CRA-P09-C03 Completion Audit

Audit Mode: phase_exit
Audit Target: Phase 17 of 44 — CRA-P09-C03 Evaluate Usefulness Over Actual Delivery Targets
Current Status: review_pending
Implementation Complete Claim: false pending this audit
Policy Source: CRA-POL-004 and cross-cutting CRA-POL-006
Goal Documents: `PLAN.md`, `EXPERIMENTS.md`, `EXPERIMENT_NOTES.md`
Repository/Commit/Dirty State: `/mnt/f/work/simulator`; HEAD
`f318414fb0156a67e04c5479000c2a2683fc0c09`; expanded dirty status 456 entries with normalized
SHA-256 `a322af44589c1dc55771bebc2c50c26766ac9fed3b1319d2a22770034a302d16`
Audit Date: 2026-08-10 (`Asia/Seoul`)

## Completion Evidence Matrix

| ID | Completion condition or invariant | Policy basis | Runtime/data/contract evidence | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- |
| P17-A01 | Cast admission is any-step over actual delivery targets | CRA-POL-004 | `SkillTargetUsefulness::Cast` filters candidates through `skill_has_useful_automatic_cast_target`; the latter iterates every step and delegates to the actual-delivery evaluator | useful-later/useless-first and reverse-order focused test; debug/release Core 986/986 | pass |
| P17-A02 | Anchor identity does not substitute for an area target set | CRA-POL-004 | TileArea usefulness resolves geometry, materializes valid cells and collects filtered targets before classifying effects | useless-anchor/useful-neighbor and useful-anchor/empty-footprint red→green cases | pass |
| P17-A03 | Materialization, clipping, filters, caster inclusion and stable order match runtime | CRA-POL-004; Phase 16 seam | predictive evaluation calls the same area geometry, `valid_tile_area_tiles` and UUID-sorted `collect_tile_area_targets_at` used by runtime | `Allies`/`Enemies`/`Any`, include-caster and 3-cell corner clipping matrix; Phase 16 query hash unchanged | pass |
| P17-A04 | Automatic and manual admission share the decision | CRA-POL-004 | automatic/readiness target selection uses typed cast mode; PlayerUnit and PlayerTile explicit validation plus final manual resolution use the shared cast evaluator | automatic assertions, public manual PlayerUnit positive/negative and PlayerTile empty/useful tests | pass |
| P17-A05 | Execution/no-op is current-step only | CRA-POL-004 | direct target resolution retains current-step classification; instant/persistent TileArea gates current targets; TargetLocked release invokes the current delivery evaluator | direct first-step HP no-op plus later stun; no projectile launch plus later stun | pass |
| P17-A06 | A no-op step does not cancel later work or alter target ordering/ties | CRA-POL-004 | each delivery finalizes its own step; typed usefulness remains only a candidate filter inside existing `CurrentTarget`/`LowestHealthEnemy`/`Nearest` ordering | both step orders and live multi-step skill suite 4/4; skill runtime integration 14/14 | pass |
| P17-A07 | Obsolete first-step/anchor shortcuts are removed without parallel authority | CRA-POL-004 | zero matches for `skill_cast_target_usefulness_step`/`usefulness_step`; one actual-delivery evaluator owns direct/projectile/TileArea prediction | static symbol/caller search and full Core suites | pass |
| P17-A08 | Public/RON/save/event/DTO/server/wire shapes and live data remain exact | CRA-POL-006 | new enum is crate-private/no-serde; no Phase 17 shape or data edit; live Skill RON is byte-identical | RON debug/release 25/25; live catalog 4/4; live item 4/4; server 59/59 | pass |
| P17-A09 | Canonical and Phase 18 boundary match current implementation | completion guide §12; Master order | corrected `skill_target_contract.md` records any-step/current-step, PlayerUnit/PlayerTile parity and actual set; preview union remains Phase 18 | canonical reinspection/hash; no obsolete-symbol matches; Master Phase 18 still pending | pass |
| P17-A10 | Dirty user/campaign work is preserved and validation is broad/current | Goal invariant; completion guide §§10–11 | activation/post hashes, isolated projectile reconstruction and unchanged dependent hashes distinguish Phase work; no destructive normalization used | focused 5/5 + 1/1; Core debug/release 986/986; integrations/server/workspace/fmt/diff all green | pass |

## Approved Policy Restatement

CRA-POL-004 requires cast admission to accept a skill when at least one step has a useful executable effect over the
targets that delivery would actually reach after footprint materialization, valid-board clipping and authored target
filtering. The selected anchor is geometry input, not proof that an affected target exists. Once admitted, each step
evaluates only its own current actual delivery set; an ineffective step is a no-op and cannot cancel later work.
Stable ordering/ties, release timing and the established no-target lifecycle remain unchanged.

The current runtime, corrected combat canonical and approved policy now express the same rule. Before this Phase,
candidate selection supplied one representative hostile step and EnemySingle TileArea evaluated the selected unit
instead of its footprint. Those shortcuts no longer exist.

## Runtime And State Evidence

### Automatic acquisition and admission

- Production callers pass authoritative `time_ms` into `resolve_skill_cast_target_at`.
- EnemySingle acquisition retains cast reach, tile range, lifecycle, hostility, air eligibility and authored
  `UnitTargetRule` ordering.
- Candidate usefulness receives `SkillTargetUsefulness::Cast { time_ms, skill }` and calls the cast-wide evaluator.
- The cast-wide evaluator iterates every hostile step. ReuseCastTarget derives that step's direct/area context from
  the candidate; RetargetOnStep selects with `DeliveryStep { time_ms, step_index, step }`, avoiding recursion into
  cast-wide aggregation.
- AutoCastStart performs the same cast-wide check after target resolution. Rejection retains pending auto-cast state
  and creates no release/delivery, matching the established no-target lifecycle.

### Manual validation

- `resolve_manual_skill_cast_request` performs eligibility/lock/resonance checks before resolving a target.
- PlayerUnit validation checks lifecycle, hostility, air and reach, then the shared cast-wide usefulness decision.
- PlayerTile validation checks valid reach for the selected tile and then evaluates the materialized delivery set.
  The selected tile need not be occupied; an empty hostile footprint is nevertheless not useful.
- A final shared usefulness gate covers Self, AutomaticUnit, PlayerUnit and PlayerTile after target resolution.
- Failure returns `NoValidTarget` before manual-cast state, resonance or event mutation. Existing manual validation
  atomicity tests remain green in the complete suites.

### Actual delivery target authority

- Direct and TargetLocked delivery use the resolved unit as the actual candidate target.
- Targetless DirectionalCollision retains the approved existing executable exception only when authored range or
  lifetime bounds and caster facing exist.
- TileArea prediction resolves the same anchor/tile origin as runtime, applies the Phase 16 valid-cell materializer,
  then calls `collect_tile_area_targets_at` with authored `hit_targets` and `include_caster`.
- Collection filters inactive units and sorts UUID bytes. Prediction allocates no area/projectile identity, records
  no event, applies no effect and mutates no cast state.
- Hostile Enemies/Any gates remain active for empty target sets, fixing the former empty-set bypass.

### Current-step execution

- Instant direct EnemySingle resolution continues to require current-step hostile usefulness.
- Instant and persistent TileArea release evaluate the current materialized/filtered target set only.
- TargetLocked projectile release now skips launch when the current hostile step has no useful actual target.
- A skipped/no-op delivery finalizes that step normally. Later scheduled steps remain in the cast and execute.
- No cooldown, resonance, release time, target rule, tie-breaker, event shape or terminal-delivery ordering was
  changed by this Phase.

## Data Evidence

- `game_resources/data/skills/base.ron` was not edited and retains SHA-256
  `4087df054a663e20ddab14f184a206d532f8df7b472c656bbd708835298f1770`.
- Current live EnemySingle TileArea definitions already include their anchors; live catalog and skill suites confirm
  existing published choices and scenarios remain valid.
- No RON schema, loader or publication rule was added. Debug/release RON tests both pass 25/25.
- Fixture-only behavior is not used as publication evidence: live catalog, live item activation and production RON
  integrations all ran against the current bundle.

## DTO/Transport/Persistence Evidence

- `SkillTargetUsefulness` and its fields are crate-private and have no serialization derives.
- No public skill type, event-log payload, range/effect preview DTO, behavior DTO, server message, save/checkpoint
  field or version changed in the Phase 17 scoped edit inventory.
- Usefulness is transient derived state at selection/release time; it creates no authoritative state requiring
  checkpoint persistence or migration.
- Server 59/59 and the strict RON suites pass against the same current Core. No dual read/write, compatibility alias
  or rollout shim was introduced.

## Consumer Evidence

- Confirmed workspace consumers: automatic acquisition/AutoCastStart, live skill readiness, public manual
  PlayerUnit/PlayerTile activation, RetargetOnStep, direct execution, instant/persistent TileArea and TargetLocked
  projectile release.
- Confirmed workspace data/transport boundary: live Skill RON, game server Core dependency and current combat
  canonical.
- Canonical external consumer: Unity consumes existing Core event/preview shapes. It does not receive the
  crate-private usefulness plan.
- External Unity repository was not accessed in this Phase. It is not a completion dependency because no serialized
  shape, field, tag, nullability or message order changed under CRA-POL-006.
- Explicitly excluded: Phase 18 public effect-preview union/content correction and any per-step preview DTO.

## Legacy/Compatibility Audit

- `skill_cast_target_usefulness_step` and `usefulness_step` have zero matches in current production Core.
- Candidate selection has no nullable first-step compatibility mode; it requires typed cast or current-delivery
  intent.
- Anchor-local hostile classification remains only as the correct direct per-unit primitive beneath the
  actual-delivery evaluator and direct execution; it is not a TileArea admission shortcut.
- No ignored/disabled test, debug bypass, old/new schema pair, fallback alias or future preview placeholder was added.
- Existing targetless directional projectile behavior is an approved preserved contract, not a new compatibility
  layer.

## Scoped Diff And Dirty Preservation

- Activation expanded status: 454 entries, normalized SHA-256
  `db4cb68b213cef6e96191066a68e9164b5e166903f0e090c71e7a2fbeda36aa5`.
- Pre-source hashes were recorded for usefulness, simulation, cast, area, Core tests, Phase 16 query, integrations,
  canonical and live data before their Phase 17 edits/inspection.
- Final pre-audit expanded status: 456 entries, normalized SHA-256
  `a322af44589c1dc55771bebc2c50c26766ac9fed3b1319d2a22770034a302d16`.
- Final hashes: canonical `1b24581...`, usefulness `73e90a5c...`, cast `8c586e5a...`, projectile `07dd8844...`,
  simulation `9d4eecc6...`, Core tests `4aab68d4...`.
- Area runtime `23e25cdf...`, Phase 16 query `73930102...`, skill integration `afd09b3f...`, RON integration
  `75558ff2...` and live Skill RON `4087df05...` retain their activation hashes.
- Projectile became a required prospective target only after audit of current-step execution found that TargetLocked
  release did not use the gate. Removing only the Phase 17 context/gate hunk into `/tmp` reconstructed pre-Phase-17
  hash `a100b805...`; unrelated pre-existing kill-stop/provenance hunks remain present.
- No reset, restore, clean, stash, destructive deletion or broad unrelated formatting was used. The temporary
  reconstruction lives outside the repository. Existing user/campaign changes and untracked files remain intact.

## Canonical Alignment

- `core_enhanced/docs/skill_target_contract.md` is the nearest targeting/delivery canonical.
- It records cast-wide any-step aggregation, current-step execution, anchor non-authority, valid-board clipping,
  filters/caster inclusion, UUID ordering, automatic/manual parity, TargetLocked release and directional exception.
- The first independent audit correctly stopped on stale PlayerTile wording. The canonical was corrected in a
  separate authorized correction cycle, static checks reran, and this audit restarted after rereading all 567 guide
  lines. The corrected canonical now distinguishes an unoccupied PlayerTile anchor from its nonempty useful hostile
  delivery set.
- Phase 18 remains the sole owner of public `effect_preview_cells` union/content semantics; this Phase adds no preview
  implementation or placeholder.

## Validation Results

All commands ran in the current dirty worktree.

- Initial intended red: `cargo test cast_usefulness_ -- --nocapture` — 3 selected tests ran and all 3 failed at the
  intended behavioral assertions; 980 lib tests were filtered out. A preceding no-run compilation error caused by
  private test access was corrected by testing the shared public-in-domain seam and is not counted as behavioral red.
- Intermediate first green: the same filter — 3/3 passed.
- Expanded final focused: `cargo test cast_usefulness_ -- --nocapture` — 5 passed, 0 failed, 0 ignored; 981 lib tests
  filtered out. Re-executed after the canonical correction and fresh guide read.
- Projectile lifecycle: `cargo test useless_projectile_step_is_a_noop_without_cancelling_later_delivery -- --nocapture`
  — 1 passed, 0 failed, 0 ignored; 985 lib tests filtered out. Re-executed after correction.
- `cargo test --lib` — 986 passed, 0 failed, 0 ignored, 0 measured, 0 filtered out.
- `cargo test --release --lib` — 986 passed, 0 failed, 0 ignored, 0 measured, 0 filtered out.
- `cargo test --test skill_runtime_contract` — 14 passed, 0 failed.
- `cargo test --test ron_loading` — 25 passed, 0 failed.
- `cargo test --test live_skill_catalog_audit` — 4 passed, 0 failed.
- `cargo test --test skill_test_suite` — 4 passed, 0 failed.
- `cargo test --test live_item_skill_activation` — 4 passed, 0 failed.
- `cargo test --release --test skill_runtime_contract` — 14 passed, 0 failed.
- `cargo test --release --test ron_loading` — 25 passed, 0 failed.
- `cargo test` in `game_server` — 59 passed, 0 failed; binary/doc harnesses selected zero tests and are not counted
  as behavioral evidence.
- `cargo check --workspace --all-targets` — passed; compile-only evidence is used only as a static workspace gate.
- `cargo fmt --all -- --check` — passed.
- repository-wide `git diff --check` — passed.

The production tree did not change after the final debug/release/integration/server runs. The subsequent correction
changed only canonical/Goal documents, so broad behavior results remain current; focused behavior was nevertheless
rerun during the fresh audit.

## Baseline Failures

- No unrelated baseline failure remains.
- The three initial failures were the intended CRA-P09-C03 characterizations: rejected useful affected footprint,
  accepted empty footprint through a useful anchor and rejected a later-useful multi-step cast.
- No ignored test or known environment failure is being separated from the verdict.

## Cross-Component Findings

- Phase 16's typed valid-cell materializer is consumed without changing its contract.
- Core selection, manual validation and runtime delivery now share actual-set semantics; server and live data consume
  unchanged public contracts.
- Phase 18's preview union is still absent and pending. No temporary Phase 17 DTO or duplicate preview authority
  bridges the phases.
- The first audit finding P17-F01 (stale PlayerTile canonical wording) was corrected and the full audit restarted.
- Unresolved material cross-component findings: none.

## Long-Term Direction Review

| Candidate ID | Work type | Current→target authority result | Preserved-invariant result | Removal/consolidation result | Validation evidence | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| CRA-P09-C03 | correctness correction | first-step/anchor approximation replaced by typed cast/current-delivery intent and one actual-set evaluator | ordering, ties, timing, no-target lifecycle, data and shapes preserved | obsolete first-step selector removed; geometry/filter producer reused rather than duplicated | focused red→green, Core 986 debug/release, integrations/server/static | pass |

- Fit: High.
- Improvement class: none.
- Reason: authority is narrower and clearer, reuses the Phase 16/runtime seams, introduces no synchronized state or
  public migration and leaves Phase 18 to its exact owner. Tests protect admission/execution behavior through public
  manual paths and event/effect outcomes rather than only enum layout or private helper shape.

## Primary Verdict

- Complete.
- Long-term fit: High.
- Combined phase-exit verdict: `Complete / High`.

All evidence-matrix rows pass. There is no unresolved material finding, policy question, scope decision, external
blocker or required compatibility layer.

## Status Transition

- from: `review_pending`, `Implementation Complete: false`
- to: `complete`, `Implementation Complete: true`
- P17-C10: complete through this audit
- next Phase activation: allowed only after Master records Phase 17 `complete — Complete / High` and initializes,
  registers and fully reads the Phase 18 Child documents under the exact linear protocol

## Remaining Risk And Required Action

- Effect usefulness classification intentionally retains its established hostile/support rules; changing refresh,
  resistance or support-value prediction is outside CRA-P09-C03.
- Targetless directional collision uses the approved bounded/facing executable exception rather than predictive path
  collision simulation.
- Unity was not inspected because no wire contract changed. Any future UI preview requirement remains Phase 18 or a
  separately approved versioned consumer migration.
- Immediate correction required: none.
- Out-of-scope follow-up required for this verdict: none.
- Policy question: none.
