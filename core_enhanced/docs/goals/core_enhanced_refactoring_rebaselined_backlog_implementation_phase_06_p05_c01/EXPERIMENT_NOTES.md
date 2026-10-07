# Phase 06 CRA-P05-C01 Experiment Notes

Status: complete
Implementation Complete: true

## Activation Boundary

- Sole active candidate: CRA-P05-C01, Master Phase 06 of 44.
- Direct predecessor Phase 05 is `Complete / High`; Phase 07 remains pending.
- Mutation authority is limited to serialized preview cache non-authority, its durable proof and nearest canonical.
- All three Child documents must be read fully before Phase 06 production/status inspection.

## Authority Notes

- Manifested node difficulty/live policy owns internal scale; serialized preview is a presentation carrier.
- `ResolvedCombatSetupPlan` is already the typed aggregate setup authority. This Phase should correct its input path,
  not create a parallel plan or redo C02/C04.
- Current runtime/serialization source overrides historical function names and field assumptions after initialization.

## Compatibility And Policy Notes

- CRA-POL-006 requires retaining current V3 and outward preview fields even when they cease to be setup inputs.
- Do not turn cache drift into silent repair of invalid graphs. The test mutation must remain structurally valid and
  differ only in a derived/cache value that is not approved checkpoint authority.
- Difficulty, content, rewards, Boss/scenario semantics and public helper cleanup are out of scope.

## Dirty Worktree Risk

The tree contains extensive user-owned combat preview/setup, Boss runtime, checkpoint/server, tests and canonical
changes. Fingerprint every prospective source/test/canonical target before editing. Do not reset, restore, clean,
stash or broadly format; stop if the required hunk cannot be separated.

## Testing Notes

- The durable sequence must use real serialization/import and the production start path, not call a private resolver
  with synthetic arguments only.
- Compare an unmodified control and mutated-cache reload at setup/enemy/reward/scenario fields that the production
  result exposes or commits.
- Retain explicit nonzero preview/setup/V3/live/server counts and distinguish zero-test binaries.
- A cache serde round trip alone is not correctness proof; the post-Continue battle must ignore the cache.

## Out-Of-Scope Follow-Up Handling

Record duplicate preview conversion/resolver, public helper and mission-shape cleanup under CRA-P05-C03/C05. Do not
absorb them into this correction.

## Implemented Authority Boundary

- Presentation selection still reads/writes `RunState::combat_previews`; cancellation intentionally does not delete
  that cache and V3 continues to persist it.
- Battle start never reads that cache. `explicit_combat_preview_for_node` deterministically regenerates the setup
  input, then the existing resolved plan owns enemy/scenario/reward construction and commit.
- The active session and cache are overwritten with the freshly planned preview only after setup succeeds, so a
  failed plan retains the existing atomicity boundary.
- Event-started combat is safe under the same rule: its authored encounter and explicit combat category regenerate
  deterministically even though the map node retains Event category and no manifested difficulty.

## Durable Proof Notes

- The exact test deliberately uses a valid Hard manifested node so skipped scale drift is observable (`110` to
  serde default `100`).
- It mutates the serialized cached wave to another existing abnormality, so Continue accepts a structurally valid V3
  document and the test proves gameplay non-authority rather than relying on invalid-checkpoint rejection.
- The control and mutation each Continue into a fresh Core and start the same node. Final active preview, enemy
  source identity and runtime stats match the pre-save authoritative preview/control.
- A separate Event retry mutation proves the retained cache is also non-authoritative for non-manifested authored
  combat.

## Compatibility And Follow-Up

- CRA-POL-006 is satisfied: V3 version/fields, `CombatPreview` serde/public shape, RON and transport are unchanged.
- Duplicate instance→preview mapping, two resolver families and ghost Floor threading remain owned by later
  CRA-P05-C03. Mission/wrapper cleanup remains CRA-P05-C05. Neither is required for this correctness correction.
- No new policy question or external consumer decision was exposed.
