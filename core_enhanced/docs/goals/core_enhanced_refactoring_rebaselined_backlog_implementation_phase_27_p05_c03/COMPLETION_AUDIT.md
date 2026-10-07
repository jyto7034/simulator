# Phase 27 CRA-P05-C03 Completion Audit

Audit Mode: `phase_exit`
Audit Target: `core_enhanced_refactoring_rebaselined_backlog_implementation_phase_27_p05_c03`
Audit Date: 2026-08-10 (`Asia/Seoul`)
Audit Pass: `2` — fresh review after exact one-path dirty-boundary exception approval
Current Status: `active`
Implementation Complete Claim: `false`
Primary Verdict: `Complete`
Long-Term Fit: `High`
Improvement Class: `none`
Next Phase Activation: `allowed after record-only Phase 27/Master transition`

## Audit Inputs

- Policy source: Phase 27 `PLAN.md`, CRA-POL-006, final exact order/test/gate catalogs, P05-C01 dependency audit.
- Goal documents: current 226-line PLAN, 313-line EXPERIMENTS and 172-line EXPERIMENT_NOTES were read fully.
- Mandatory guides: the 198-line resume attachment, 874-line authoring/execution guide and 567-line completion
  review guide were read fully for this independent pass.
- Master registry: Phase 27 is the sole `active` Child; Phase 28 CRA-P08-C05 remains `pending` and requires Phase 27
  `Complete / High`.
- Direct predecessor: Phase 26 final audit is `Complete / High`.
- Semantic prerequisite: Phase 06 P05-C01 final audit is `Complete / High`; battle setup regenerates instead of
  trusting the serialized presentation cache.
- Repository HEAD: `f318414fb0156a67e04c5479000c2a2683fc0c09`.
- Activation dirty state: 511 expanded rows at `31491d6979b1e18fbec73510c9b4877f46d1971af0b7ef161699869d624e6738`.
- Current expanded state: 513 rows at normalized (C-locale sorted) SHA-256
  `0df73ab3f4add04e7a281d6a50eb9d4d26c9f38a2949b710cc4214c82f1a6471`.
- Removing only the intended new tracked `combat_preview/types.rs` row and this Phase-owned audit row yields the
  exact activation state: 511 rows at `31491d6979b1e18fbec73510c9b4877f46d1971af0b7ef161699869d624e6738`.
- Approved exception evidence: current `game_server/logs/app.log.2026-08-09` is a 57-byte user-created marker with
  exact text `로그 손실을 한 경로의 예외로 명시적 승인`, SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730` and mtime/ctime
  `2026-08-10 22:55:07.178423600 +0900`. It approves the original log loss; it is not a restoration claim.

## Approved Policy Restatement

CRA-P05-C03 is a behavior-preserving private-authority consolidation. Ordinary manifested, Boss, Event-authored and
Monolith preview resolution must retain exact live node/category/encounter/difficulty/seed inputs, complete ordered
output, error/cache ordering, Event retry, Monolith invalidation and P05-C01 battle-start regeneration. One private
typed input, resolver and instance-to-preview conversion replace the duplicated mechanics. Floor remains a retained
public compatibility input but cannot become gameplay/cache authority. Under CRA-POL-006, workspace caller absence
does not authorize removal or narrowing of any public function, field or signature. No DTO, serde, V3, RON, server or
Unity migration is authorized.

## Completion Evidence Matrix

| ID      | Completion condition                                      | Policy basis                                                   | Runtime/data/contract evidence                                                                                                                                                             | Validation evidence                                                                        | Verdict  |
| ------- | --------------------------------------------------------- | -------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------ | -------- |
| P27-C01 | complete resolver/converter/cache/caller graph identified | Phase objective and inventory gate                             | `combat_preview/mod.rs`, `types.rs`, `world/combat.rs`, `node_flow.rs`, validation and snapshot callers; workspace/test/external classifications recorded in E03                           | caller/owner/Floor searches rerun in audit                                                 | pass     |
| P27-C02 | dirty targets and concurrent work preserved               | authoring guide §8, completion guide §10, exact user exception | one-path original-log loss is explicitly accepted; marker is preserved; every other activation row reconstructs exactly after excluding the two intended Phase-added rows                    | current/filtered counts and normalized hashes; marker byte/metadata check                   | pass     |
| P27-C03 | old behavior characterized first                          | test-first constraint                                          | two complete-value tests were added before production consolidation; baseline commands selected nonzero old-production cases                                                               | old production 27 + 11 + 42 + 3 + 19 + 1; new exact tests 1 + 1 green before refactor      | pass     |
| P27-C04 | one private resolved-input and conversion authority       | target private authority                                       | one `CombatPreviewResolutionInput`, one `resolve_combat_preview_for_node`, one `try_generate_from_request`, one `from_battlefield_instance`; single production conversion/generation sites | negative duplicate searches and 28 preview tests                                           | pass     |
| P27-C05 | cache/setup/error ordering exact                          | P05-C01 and locked order                                       | presentation cache hit precedes applicability/difficulty gate; battle-start mode receives no cache; commit publication remains separate; exact missing-authority text retained             | cache-role test 1, durable Continue 1, map-flow 42                                         | pass     |
| P27-C06 | ordinary/Boss/Event/Monolith parity exact                 | four-family matrix                                             | typed mode retains actual versus generation category/encounter; Monolith invalidation remains in node flow; Event retry sees cache first                                                   | ordinary/Boss complete equality 1, Event 3, Monolith 19, preview 28                        | pass     |
| P27-C07 | Floor contained without public narrowing                  | CRA-POL-006                                                    | no `request.floor_index` read; field and all public wrapper signatures remain; compatibility comment added                                                                                 | four seeds, Floor 0 vs `u32::MAX`, complete instance/preview equality 1                    | pass     |
| P27-C08 | DTO/V3/RON/server/Unity compatibility exact               | no-migration boundary                                          | public/serde types preserved; live data and all four canonicals retain activation hashes; deployed Rust consumers remain unknown and APIs are retained                                     | checkpoint 6, live RON 25, Core/server all-target suites                                   | pass     |
| P27-C09 | focused and broad validation green                        | Phase validation strategy                                      | current production hashes match E05/E06 implementation; no later source/data/canonical change                                                                                              | Core debug/release 1,067 each; server debug/release 59 each; four all-target checks exit 0 | pass     |
| P27-C10 | downstream boundary and independent audit current         | Phase exit gate                                                | no Phase 28 path exists or is modified; Master row 28 remains pending                                                                                                                      | this fresh review yields `Complete / High`                                                  | pass     |

## Runtime And State Evidence

### Generation and conversion

- `BattlefieldGenerator::try_generate_with_difficulty` remains the validated instance producer. Generation still
  derives authored/fallback mission and template state, manifested or standard tuning, ordered geometry/waves,
  warnings and internal scale before conversion.
- `CombatPreview::try_generate_from_request` captures the compatible encounter identity, invokes that validated
  producer and routes the result to `from_battlefield_instance`.
- `from_battlefield_instance` is the sole production move of all 19 `BattlefieldInstance` fields plus `node_id` and
  owned `encounter_id`. Audit search finds one `battlefield_template_id: instance...` and one
  `enemy_stat_scale: instance...` production mapping.
- Public `try_generate_for_node`, `try_generate_for_node_at_floor`, `generate_for_node` and
  `generate_for_node_at_floor` retain their names, visibility, argument order, return behavior and panic text.

### World resolution and cache lifecycle

- `CombatPreviewResolutionMode::Presentation` derives generation category and optional encounter from the live node,
  clones the cache, and returns `CacheHit` before category/difficulty/Monolith gates. This preserves Event-authored
  retry behavior.
- Presentation misses for non-Combat/Boss return `NotApplicable`. Manifested nodes still require the exact live
  difficulty authority, run Monolith authority validation, derive the `PREV` node seed and generate before the
  mutable wrapper publishes the cache.
- `BattleStart` takes the explicit generation category/encounter but retains `actual_category` and live difficulty
  for manifested-authority errors. It receives no cache and never returns a cache hit.
- `explicit_combat_preview_for_node` therefore remains P05-C01 fresh setup authority. The later successful commit,
  rather than the resolver, publishes its preview.
- Monolith conversion still changes encounter/stage difficulty, removes `combat_previews` and attempts, then invokes
  presentation resolution. Event, ordinary and Boss callers retain their existing paths.

## Data Evidence

- No live RON/data file was changed. Current hashes equal activation:
  - PVE encounters: `0b3f03b7e68c3aff5db6270e9eaa83eacb4eb2da188e55058899db22f0a3ee1c`;
  - battlefield templates: `1f51d0e4cc1a784c684e60cf4bef9ee4b4e943aa776cd7db781ad7078144223f`;
  - battlefield archetypes: `c8251aab51532ee5f045fe182999e0f627e89c7e8ca936f2714bf165fc043dd9`;
  - run policy: `83278e068302506f683eb593260ff3fe1024f71c30236c3a4b46c3e47ef7fe55`.
- Production validation still generates baseline and manifested-difficulty previews across five seeds, validates
  required warnings and passes the preview into live setup-feasibility checks.
- The 25-case live-RON integration suite exercises live encounter references, authored scenario preview data and
  published warning contracts. A `game::data::validation::` name filter selected zero and is not counted as proof.

## DTO, Transport And Persistence Evidence

- `CombatPreview` remains the same public `Serialize`/`Deserialize` DTO. `enemy_stat_scale` remains `#[serde(skip)]`;
  `threat_warnings` remains defaulted; no field, tag, casing or nullability change was introduced.
- `BattlefieldGenerationRequest` retains public `category`, `encounter_id`, `seed` and `floor_index` fields.
- V3 `combat_previews` remains a presentation cache. Snapshot projection still clones that cache only for
  `NodeConfirm`; P05-C01 battle setup regenerates from current authority.
- No checkpoint version, reader/writer, server message, Unity DTO, migration, alias, dual read/write or fallback was
  changed.
- Current canonical hashes equal activation:
  - Core runtime: `6e20d6a6c0609e6217b3a00d98371b7e13af12ef7d649f86855238c74c1ec00d`;
  - game rulebook: `6c53ecd5d0588aace1a2b043b31434d004327d60938ecf61066ac19acfd31951`;
  - server bridge: `4d734af0115a8c57e17bb8d43cd18af76452345df936633dde9c5fa188dd0219`;
  - external Unity: `a3224e244f9577fbb6a4d6d01e44b7ea85f3ba1969f656a22712b0ba972063f5`.
- `core_runtime_contract.md` already states that node-confirm/V3 previews are presentation-only and that actual
  ordinary/Boss/Event/Monolith setup regenerates from current node identity, manifested difficulty, encounter, live
  data and deterministic seed. `game_rulebook.md` already rejects legacy Floor-number scaling. No canonical edit is
  required for this behavior-preserving private refactor.

## Consumer Evidence

- Confirmed workspace production consumers:
  - data validation calls the public Floor wrapper and crate-private difficulty-aware wrapper;
  - world resolution is the only production caller of the crate-private difficulty-aware wrapper.
- Confirmed workspace test consumers: live-RON integration and Event combat tests use public `generate_for_node`;
  map-flow tests use public `try_generate_for_node`; preview tests cover the request/generator/wrapper families.
- The `events/combat.rs` matches are inside its `#[cfg(test)]` module, not production runtime authority.
- `game_server` has no direct Rust symbol reference and consumes serialized Core behavior through the existing
  bridge. The accessible external Unity tree has no Rust API symbol reference and continues to consume the unchanged
  serialized preview contract.
- Unknown deployed Rust consumers cannot be disproved. CRA-POL-006 is satisfied by retaining every public symbol,
  field and signature.

## Legacy And Compatibility Audit

- Removed private duplication: the second instance-to-preview mapping and second world generation authority.
- Retained by approved compatibility policy: all public generator/preview functions and the public Floor-bearing
  request/wrappers.
- Audit search finds no `request.floor_index` read, no parallel public API, schema alias, dual behavior, new fallback,
  normalization, silent recovery, ignored characterization, debug bypass or Phase 28 placeholder.
- Existing encounter/template mission fallback behavior is unchanged production policy, not a compatibility shim
  introduced by this Phase.

## Scoped Diff And Dirty Preservation

### Intended Phase paths

- Production: `combat_preview/mod.rs`, `combat_preview/types.rs`, `world/combat.rs`.
- Characterization: `world/tests/combat.rs` plus strengthened tests in `combat_preview/mod.rs`.
- Goal records: PLAN, EXPERIMENTS, EXPERIMENT_NOTES and this audit.
- No live data, save/schema, server, Unity or canonical file changed. `git diff --check` is clean on all intended
  Phase paths.

### Finding F-01 — pre-existing untracked log deleted; exact exception approved

- Activation status included `?? game_server/logs/app.log.2026-08-09`; the file was 688,108 bytes with modification
  timestamp `2026-08-10 02:18:41 +0900`.
- The file was incorrectly treated as a new test artifact and explicitly deleted. It was not produced by the current
  test run: rerunning all 59 server tests did not recreate it.
- Recovery checks found no open deleted handle, rotation copy, F:-drive/work copy or Recycle Bin copy. Git cannot
  restore an untracked file; the original bytes remain lost.
- External user state later created a 57-byte marker at that path with the exact unambiguous approval text recorded
  in Audit Inputs. This accepts only the original 688,108-byte log loss as a dirty-boundary exception. It does not
  waive another path, and the marker itself is protected current user-owned state.
- Current normalized status-set comparison is decisive: excluding only the intended `types.rs` and audit rows
  reproduces all 511 activation rows and activation hash exactly. The marker remains unchanged across fresh tests.
- Disposition: F-01 remains visible as historical data loss but is closed for Phase completion by explicit approval;
  every non-excepted dirty-preservation obligation passes.

## Validation Results

### Pre-refactor baseline on old production

- `cargo test --lib game::combat_preview::tests::`: 27 passed, 0 failed, 0 ignored.
- `cargo test --lib game::combat_setup::`: 11 passed, 0 failed, 0 ignored.
- `cargo test --lib game::world::tests::map_flow::`: 42 passed, 0 failed, 0 ignored.
- `cargo test --lib event_started_combat`: 3 passed, 0 failed, 0 ignored.
- `cargo test --lib monolith`: 19 passed, 0 failed, 0 ignored.
- `cargo test --lib durable_continue_recomputes_battle_setup_instead_of_trusting_serialized_preview_cache`: 1
  passed, 0 failed, 0 ignored.

### Test-first characterization on old production

- `cargo test --lib public_generation_entrypoints_preserve_complete_output_across_floor_values`: 1 passed, 0 failed,
  0 ignored.
- `cargo test --lib presentation_and_battle_start_resolvers_preserve_complete_manifested_previews_and_cache_roles`:
  corrected exact filter 1 passed, 0 failed, 0 ignored.
- Preview suite after test addition but before production refactor: 28 passed, 0 failed, 0 ignored.
- Initial sibling-module draft produced two E0624 compile errors because it called a private helper. The test was
  corrected to exercise real Select/Confirm behavior without widening visibility; this was not a production failure.

### Focused post-implementation

- Preview suite: 28 passed; resolver/cache-role exact filter: 1 passed.
- Setup: 11 passed; map-flow: 42 passed; Event-authored: 3 passed; Monolith: 19 passed.
- Durable Continue: 1 passed; V3 checkpoint validation: 6 passed; live-RON integration: 25 passed.
- Every counted command reports 0 failed and 0 ignored. The zero-case validation-name filter is excluded.

### Fresh exception-reaudit execution

- `cargo test --lib game::combat_preview::tests::`: 28 passed, 0 failed, 0 ignored.
- `cargo test --lib presentation_and_battle_start_resolvers_preserve_complete_manifested_previews_and_cache_roles`:
  1 passed, 0 failed, 0 ignored.
- `cargo test --lib durable_continue_recomputes_battle_setup_instead_of_trusting_serialized_preview_cache`: 1
  passed, 0 failed, 0 ignored.
- `cargo test --lib event_started_combat`: 3 passed, 0 failed, 0 ignored.
- `cargo test --lib monolith`: 19 passed, 0 failed, 0 ignored.
- Marker size, timestamps and SHA-256 are identical before and after these executions.

### Broad current implementation

- Core `cargo test --all-targets`: 1,016 lib + 4 + 4 + 25 + 14 + 4 integration = 1,067 passed, 0 failed,
  0 ignored; zero-case binary targets excluded.
- Core `cargo test --release --all-targets`: the same 1,067 passed, 0 failed, 0 ignored.
- Core `cargo check --all-targets` and `cargo check --release --all-targets`: both exit 0; checks are compile gates,
  not behavior counts.
- Server `cargo test --all-targets` and `cargo test --release --all-targets`: 59 passed in each profile, 0 failed,
  0 ignored. A post-incident debug rerun also passed 59/59.
- Server `cargo check --all-targets` and `cargo check --release --all-targets`: both exit 0.
- Non-strict Clippy exits 0. Core reports established lib 82/lib-test total 135 warning debt plus integration warnings;
  server reports dependency warnings and server lib 3/lib-test total 7. No warning points to a new Phase 27 symbol.
  Strict warning denial is not claimed.
- These broad results remain current for the fresh review because all four Phase source/test hashes, all four live
  data hashes, all four canonical hashes, direct dependency audits and the execution environment are unchanged.
  Only Goal records and the non-consumed approval marker changed; the fresh focused commands above verify the current
  runtime seam without manufacturing an unnecessary production change.

## Baseline Failures

- No old-production baseline test failure was observed.
- No focused or broad post-implementation test/check failure remains.
- The only failed development attempt was the test-only E0624 visibility error described above; it was corrected
  without production visibility change.

## Cross-Component Findings

- No Core/data/serde/V3/server/Unity contract drift was found. All behavior-bearing components use the same retained
  preview shape and P05-C01 setup boundary.
- No Phase 28 production or Goal artifact was prebuilt; Master Phase 28 remains pending.
- F-01 is cross-boundary only as an explicitly accepted repository-preservation exception. It does not affect
  runtime, data, DTO, persistence, server behavior or Unity contract, and no broader waiver is inferred.

## Long-Term Direction Review

| Candidate ID | Work type                    | Current→target authority result                                                      | Preserved invariant result                                                        | Consolidation result                                                    | Validation evidence                             | Verdict |
| ------------ | ---------------------------- | ------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------- | ----------------------------------------------------------------------- | ----------------------------------------------- | ------- |
| CRA-P05-C03  | behavior-preserving refactor | duplicated conversion/world generation → one typed private input/resolver/conversion | ordinary/Boss/Event/Monolith, cache/setup/error/Floor/public/serde behavior exact | duplicate private mechanics removed; compatible public surface retained | focused families plus Core/server debug/release | pass    |

- Fit: `High`.
- Improvement class: `none` for the implementation.
- Reason: authority is smaller and domain-typed, public compatibility is deliberate and tested, the presentation
  versus setup boundary is explicit, tests protect durable behavior rather than helper layout, and no next-Phase
  placeholder or remigration seam was introduced.
- The approved one-path exception does not alter the implementation fit or create a runtime compatibility seam.

## Primary Verdict

`Complete`.

The approved functional refactor, compatibility boundary and all required validations are complete with High
long-term fit. F-01 is not hidden: the original diagnostic-log loss remains documented and unrecovered, but the
user explicitly approved that exact one-path loss. Exact status-set evidence proves every non-excepted activation
path remains accounted for, and the new approval marker is preserved. No unresolved material finding remains.

## Status Transition

- from: `active`, `Implementation Complete: false`.
- to: `complete`, `Implementation Complete: true`.
- Master Phase 27: may transition to `complete — Complete / High`.
- Phase 28: may transition from `pending` to `ready`, then be initialized and solely activated under its own
  activation protocol.

## Remaining Risk And Required Action

1. Apply the record-only Child and Master Phase 27 completion transitions and preserve the approval marker unchanged.
2. Record Phase 28 as `ready`; initialize/read its three required documents before any Phase 28 production search,
   test or mutation, then make it the sole active Child.
3. Remaining material Phase 27 risk: none. The accepted original-log loss remains an explicit historical exception,
   not a recovered artifact or a general dirty-work precedent.
4. Unknown deployed Rust callers remain a non-blocking compatibility risk because every public API was retained.
