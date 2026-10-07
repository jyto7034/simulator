# Phase 21 CRA-P02-C04 Experiment Notes

Status: complete
Implementation Complete: true
Candidate: CRA-P02-C04
Master Phase: 21 of 44
Date: 2026-08-10 (`Asia/Seoul`)

## Locked Decisions

- CRA-POL-006: public Rust/V3/map DTO fields and all current values remain exact.
- F-C01 remains restored graph admission authority; Phase 20 remains exact capture/restore authority.
- Graph-derived selectability remains command/snapshot admission; stored availability is not promoted.
- Consolidate only paired progression-ID/node-state mutation across current legitimate lifecycles.
- Preserve normal/BossOmen/admin behavior, sorting/dedup, directed and concealed/obscured rules.
- Phase 22 equipment/loadout and every later refactor/cleanup are prohibited from this Child.

## Open Evidence Slots

- N02: activation expanded status, prospective target fingerprints and scoped dirty attribution.
- N03: current normal/Boss/admin/restore mutation call graph and exact transition table.
- N04: public/V3/DTO/server/Unity/admin consumers and compatibility inventory.
- N05: pre-refactor valid/invalid/post-restore characterization.
- N06: implemented typed transition authority, transferred callers and obsolete direct writes.
- N07: final focused/broad/static/canonical evidence and independent-review handoff.

## 2026-08-10 — N01 Policy, Dependency And Activation Boundary

- Source/evidence: CRA-POL-006; original/re-audited P02-C04 evidence; Final order/test/gates; F-C01 and Phase 20
  completion audits; Master readiness transition.
- Observation: valid production paths are green, but normal methods, BossOmen and admin historically mutate related
  public progression IDs and map node states through multiple seams.
- Observation: exact behavior proof must precede encapsulation. Public/serde fields and graph-derived navigation are
  preservation seams, not removal targets.
- Impact: typed private transitions may consolidate paired writes only; navigation, schema, Boss/admin semantics,
  compatibility and Phase 22 work are forbidden.
- Disposition: activate Phase 21 only; do not inspect production until all Child documents and the final Phase 20
  audit are fully read and the dirty baseline is captured.

## 2026-08-10 — N02 Activation Dirty Baseline

- Source/evidence: expanded status, scoped status and SHA-256 fingerprints after mandatory reads and before
  production/test/canonical content inspection.
- Observation: HEAD is `f318414fb0156a67e04c5479000c2a2683fc0c09`; expanded status is 472 entries at
  normalized SHA-256 `c6a90d5151d585c0e3ff4d1122679829eb439b6dfd7fcd926a0d5a25af04e0a0`.
- Core prospective hashes: progression `135335b9...`, map module `c29d05b9...`, BossOmen `e586065d...`, node flow
  `a311f02b...`, map content `233bb50a...`, admin root `8c0c4bca...`, admin fixtures `7bad7784...`, admin tests
  `456ff896...` and state `1e5f9177...`.
- Test prospective hashes: map flow `1fb7e507...`, combat `2e0dbdc4...`, checkpoint validation `9d95703b...`.
- Canonical hashes: runtime `6e20d6a6...`, rulebook `6c53ecd5...`, server bridge `4d734af0...`, external Unity
  `a3224e24...`.
- Observation: progression, node flow, combat test and the three workspace canonicals are tracked-modified;
  checkpoint validation is untracked. BossOmen/admin/map-flow and other prospective paths currently have no status
  row. Existing overlapping content predates Phase 21 until scoped attribution proves otherwise.
- Impact: edits must be minimal and hash-tracked. No reset, restore, clean, stash, broad format or adjacent
  progression correction is permitted.
- Disposition: baseline gate complete; current production/test/canonical inspection may begin.

## 2026-08-10 — N03 Current Mutation Call Graph And Transition Table

- Normal entry authority: `handle_confirm_enter_node` stages map/progression and calls `enter_node`. Admission is
  `selectable_node_ids`; success changes the entered node to `Unavailable + Revealed`, sets current, clears stored
  availability and retains completed IDs. Unavailable/missing admission failure changes neither map nor progression.
- Normal completion authority: staged node completion calls `complete_current_node`. It makes current
  `Completed + Revealed`, inserts current into deterministically sorted completed IDs, clears availability and
  either returns a terminal resolution or derives reachable successors through directed completed transit. Normal
  successors become `Available`; concealed successors become revealed; availability is sorted and deduplicated.
- BossOmen seam: after forced boss normal completion returns no terminal resolution, `boss_omen.rs` selects
  precommitted `BossOmenExitGate` terminals, directly makes them `Available + Revealed`, directly assigns sorted
  availability, then clears forced BossOmen state. Current remains the completed boss and completed IDs retain it.
- Detached admin seam: the generic admin fixture appends one `admin_fixture` node as `Unavailable + Revealed`,
  directly makes it current and directly stores it as the sole available ID; completed IDs remain unchanged. This
  special debug fixture state is intentionally preserved even though it is not a normal checkpoint lifecycle.
- Authored admin seam: SavePoint resolves/generates an authored facility, rewrites the authored node content,
  makes it `Unavailable + Revealed`, directly sets current, clears availability and removes that node from
  completed IDs. The resulting state satisfies F-C01 and produces a valid SavePoint checkpoint after completion.
- Restore seam: F-C01 resolves the exact facility, validates map shape and calls the existing progression invariant
  before Phase 20 conversion mutates live state. Continue success therefore restores sorted IDs/node states that
  compare exactly with graph-derived selectable IDs.
- Direct production write count outside `progression.rs`: BossOmen one availability assignment plus sort; generic
  admin two assignments; authored admin current assignment, availability clear and completed retain. Whole-value
  replacement after generated-map creation remains legitimate construction, not an in-place seam.
- Disposition: transfer these three paired seams into crate-private methods on `MapProgression`; retain the existing
  checker and all normal algorithms unchanged.

## 2026-08-10 — N04 Compatibility And Consumer Inventory

- `MapProgression` remains `pub`, derives exact `Serialize/Deserialize`, and keeps public
  `current_node_id`, `available_node_ids`, `completed_node_ids` fields. Phase 20 V3 capture/restore serializes the
  value unchanged; F-C01 reads it unchanged.
- Core player commands and root snapshot use `selectable_node_ids`, not stored availability, for admission and
  `map_navigation.selectable_node_ids`. `MapProgression::view` masks stale non-selectable `Available` projections
  and emits the existing sorted `MapViewDto` without internal ID lists.
- Server handlers consume projected node state/navigation and expose no mutation seam. The server bridge canonical
  says Core owns map progression. Unity canonical explicitly consumes only `map_navigation.current_node_id` and
  `selectable_node_ids`, honors directed completed transit and concealed/obscured rules, and forbids public legacy
  progression lists.
- BossOmen canonical requires the precommitted concealed Exit Gate to be the sole revealed successor before Gate
  confirmation. Admin result/session snapshots depend on exact fixture states but do not require direct fields.
- No source schema, DTO, RON, server, Unity or canonical edit is required. The target is a private Core owner move
  under CRA-POL-006.
- Disposition: preserve every public/serde/DTO value and verify negative direct-write search plus unchanged canonical
  hashes at exit.

## 2026-08-10 — N05 Pre-Refactor Characterization

- Baselines before test edits: progression 10/10, map flow 40/40, admin 16/16, BossOmen filter 11/11 and checkpoint
  validation 6/6.
- Added normal assertions freeze exact sorted successor IDs, equality with graph-derived selectability, revealed
  available node states and the existing checkpoint invariant. Invalid unavailable and missing entry now compares
  full cloned map and progression for no mutation.
- Added BossOmen assertions freeze current/completed/available vectors, completed boss state, sole Gate category,
  kind, `Available + Revealed` state and post-transition F-C01 invariant.
- Added admin assertions freeze detached Maintenance current/availability/completed and node state/visibility, plus
  authored SavePoint empty availability, non-completed current and `Unavailable + Revealed` node with valid F-C01.
- Added post-Continue assertions freeze restored V3 invariant success and stored availability equality with
  graph-derived selectable IDs before exact re-export comparison.
- These behavior characterizations passed with counts 10, 1, 6, 1 and 1 respectively. The next invalid
  forced-terminal atomicity test produced the expected pre-implementation `E0599` for the absent typed helper.
- Disposition: characterization gate complete; production implementation may begin.

## 2026-08-10 — N06 Implemented Authority And Caller Transfer

- Source/evidence: `progression.rs`, `boss_omen.rs`, `world/admin/fixtures.rs`; focused transition tests and final
  direct-write search.
- `reveal_terminal_nodes` owns typed forced-terminal node state/visibility plus sorted stored availability. It first
  proves every selected terminal node exists, so corrupt multiple-terminal input cannot leave an earlier node
  revealed before returning `MissingNode`.
- `activate_detached_fixture_node` owns append/current/availability and node state/visibility while preserving the
  special detached admin result exactly. It does not pretend the debug fixture is a normal checkpoint lifecycle.
- `activate_authored_fixture_node` owns state/current/availability/completed updates after the caller authors
  SavePoint content. Its result still satisfies the existing F-C01 invariant and yields a valid checkpoint.
- Boss state clearing, admin UUID consumption, node content/session construction and generated-map whole-value
  replacement stay with their current lifecycle owners. No helper absorbed unrelated policy or transaction work.
- Public fields stay unchanged. Normal `enter_node`, `complete_current_node`, `selectable_node_ids`,
  `validate_checkpoint_state`, `refresh_available_nodes` and `view` algorithms are unchanged.
- Production direct-write search outside `progression.rs` now has zero writes; all external matches are reads. The
  obsolete Boss/admin field assignments and paired node-state writes were removed.
- Disposition: P21-C04–C08 complete; no compatibility shim, canonical edit or Phase 22 prebuild is needed.

## 2026-08-10 — N07 Final Validation And Review Handoff

- Focused current-tree totals: progression 11, map flow 40, admin 16, BossOmen 11 and checkpoint validation 6; all
  passed with zero failed/ignored.
- Broad current-tree totals: Core debug/release each 998 unit + 51 official integrations; server debug/release each
  59. Core/server debug/release checks, formatter and repository diff check all pass.
- Clippy remains a diagnostic-only global gate because unchanged broad debt produces Core lib 83, Core lib-test
  136 and server lib 3 warnings. Scoped output has no Phase 21 production diagnostic; no unrelated cleanup was made.
- Final hashes: progression `798c04a1...`, BossOmen `eb34f230...`, admin fixtures `c319893c...`, admin tests
  `8a1078a2...`, combat test `933df538...`, checkpoint validation `6c269025...`.
- Canonicals are byte-identical to activation: runtime `6e20d6a6...`, rulebook `6c53ecd5...`, server bridge
  `4d734af0...`, external Unity `a3224e24...`. Private ownership moved without changing durable language.
- Activation status was 472 entries at `c6a90d51...`; pre-audit is 475 at `30ebdf00...`. BossOmen, admin fixtures
  and admin tests were clean at activation and account for the three new status rows. All pre-existing dirty files
  and the untracked checkpoint test were preserved in place.
- No data, schema, DTO, server, Unity, migration, fallback, public visibility or Phase 22 source changed.
- Disposition: P21-C01–C09 complete; set Child to `review_pending` and perform the independent phase-exit audit.

## 2026-08-10 — N08 Independent Audit Disposition

- Audit source: final `COMPLETION_AUDIT.md`, after complete guide/Child/runtime/consumer reread and 84 focused reruns.
- Primary verdict: `Complete`; long-term fit: `High`; improvement class: `none`.
- All P21-C01–C10 pass. No material policy, runtime, data, persistence, public, consumer, canonical, dirty-scope or
  validation finding remains.
- Audit-inclusive status: 476 entries, normalized SHA-256 `29e4646f...`.
- Disposition: Phase 21 complete. Master may record `complete — Complete / High` and prepare Phase 22 only through
  the strict linear activation protocol.
