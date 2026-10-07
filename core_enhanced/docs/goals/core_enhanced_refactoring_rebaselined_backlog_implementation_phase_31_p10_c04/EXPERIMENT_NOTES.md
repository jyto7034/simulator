# Phase 31 CRA-P10-C04 Experiment Notes

Status: complete
Implementation Complete: true

## N01 — Structural Boundary

- Core owns gameplay snapshot content and gameplay action availability. The server overlay may preserve, append or
  fatally replace the capability vector only from typed server persistence state; it may not infer gameplay state.
- Active-run and encounter-compendium stores/state machines remain distinct authorities. The overlay reads their
  existing results and does not move persistence into DTO construction or Core.
- Phase 30's descriptor supplies exact host identity/scope metadata. Phase 31 must not reopen route classification.
- Phase 32 error-code consolidation and later result/transport work remain pending and must not be prebuilt.

## N02 — Exact Compatibility Boundary

- The flattened root is the contract. A typed implementation does not authorize a nested `core`/`host` object,
  renamed key, changed omission/null rule or reordered capability vector.
- Fatal compendium recovery is replacement, not an ordinary merge. Exact currently permitted recovery actions and
  order must be characterized before implementation.
- Duplicate absence is an asserted contract. Do not silently dedupe with arbitrary first/last precedence; prove the
  old disjointness and encode explicit composition semantics.
- CRA-POL-006 permits private typed consolidation only with exact old/new serialized parity.

## N03 — Dirty Worktree Precautions

- Initialization baseline is 527 expanded rows with normalized status SHA-256
  `50611a48e9742620b40f47951efb6bb213b77c69b0fe661133603f86ff025d21`.
- These three Child files are the only expected new paths before activation. Existing server/Core source, tests,
  canonicals and Goal history belong to the user/completed campaign and must not be reset, restored or reformatted
  broadly.
- The 57-byte Phase 27 marker is protected at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`; its one-path exception does not expand.
- Capture prospective target hashes and current status before the first new Phase-31 content read.

## N04 — Validation And Remaining Risk

- The decisive new safety net is a complete old-production overlay table, not a helper-only typed-struct test.
  It must compare final flattened values, exact action order and duplicates across real state combinations.
- Existing server actor/persistence tests are retained but do not by themselves prove the full Cartesian overlay
  boundary; table rows and omissions must be discovered and justified.
- Unknown deployed Unity/Rust consumers remain explicit compatibility risk. They do not block a byte/structure-
  identical private server refactor, but any output difference is a stop condition.
- Exact current types, state reachability, field omission rules, test commands/counts and minimal typed shape remain
  discovery pending after sole activation.

## N05 — Initialization Boundary

- Phase 31 is initialized only. Master still marks it `ready`; no implementation Child is active.
- Prior Phase 30 audit observations are dependency evidence, not permission to skip Phase 31 activation,
  prospective fingerprints, current diff attribution or baseline.
- No new Phase-31 production inspection or test has occurred. Sole activation is the next permitted transition.

## N06 — Activation Boundary

- Phase 31 is now the sole active Child; Phase 32 and all later work remain pending.
- The 530-row post-initialization status and hash in E02 are the activation document baseline.
- Active-document reread and prospective target fingerprints must close before new production detail is read.

## N07 — Prospective Dirty Collision Map

- The central server handler, player snapshot state/messages, Core snapshot/capability/state, both nearest
  canonicals and multiple tests are already dirty. Their exact activation hashes are recorded in E03; no Phase 31
  hunk is yet attributed to them.
- `player_game_actor/session.rs`, `encounter_compendium_store.rs` and the server bridge canonical are clean
  prospective targets. Any new status row requires exact explanation.
- Prefer a server-private overlay next to the actual snapshot owner unless the current caller graph proves a
  different narrow boundary. Do not create a parallel public DTO or alter Core ownership merely to avoid a dirty
  file.

## N08 — Frozen Overlay Semantics

- The first safety net is a real 3 × 5 × 5 actor/store matrix (75 rows), not a helper-only unit table, and it passes
  on the old serialized-then-mutated implementation.
- Active-run host status is absent outside title and when unconfigured. Its `system_issue` key is absent for
  missing/loaded and present only for reset-required errors.
- Compendium host status is present in every game state when configured, and its `system_issue` key is always
  present, using JSON null for clean/recovered.
- Title action order is Core first, then the active-run action, then compendium retry when dirty/recovered/fatal,
  then compendium reset. Fatal compendium replaces Core plus active actions and leaves exactly retry then reset.
- Outside title, no host capability is appended and fatal compendium does not clear Core capabilities. No action is
  duplicated in any of the 75 old-production rows.
- The implementation may move these facts into private typed values but may not introduce generic deduplication,
  public DTOs, alternate serializers or Phase 32 error authority.

## N09 — Implemented Authority Boundary

- `PlayerStateSnapshotDto` remains the flattened server wrapper and preserves its public two-argument constructor.
  The host overlay application method and all new status/overlay types are crate-private.
- Actor persistence locks and stores still decide state; DTO constructors only encode a typed projection. Core's
  capability vector is moved into the wrapper unchanged before the host overlay composes its own actions.
- There is one explicit typed append site and one explicit fatal clear site. Duplicate absence is asserted, not
  repaired by a new runtime deduplication policy.
- The final server wire value is created by one serialization after typed composition. Test code retains an
  independent declarative `Value` oracle solely to prove old/new full-output parity.

## N10 — Validation And Audit Handoff

- Debug/release Core behavior totals are 1,073 each and debug/release server totals are 60 each, all green. Live RON
  is covered by the Core integration target; no runtime/data migration exists for this refactor.
- Normal Clippy diagnostic gates exit 0 with established warnings. Strict `-D warnings` remains an unrelated
  repository-wide cleanup gate; the one Phase-31 warning found during diagnosis was corrected and is absent from
  the repeated strict server output.
- The main 530-row status set and normalized hash are unchanged from activation, and the protected marker remains
  byte/hash/timestamp exact.
- Accessible Unity source consumes the exact flattened fields/capabilities and rejects duplicates; it was inspected
  read-only. Unknown deployed consumers are closed by the 75-row full serialized parity proof, not by negative
  search.
- A separate completion audit must reread production, tests, canonical, consumer and dirty evidence rather than
  accepting E04~E07 as proof. Phase 32 remains pending until `Complete / High`.

## N11 — Completion Closed

- The independent 342-line audit returned `Complete / High` with all eleven completion conditions passing and no
  correction finding. The frozen audit hash is `7704d01389707e224118e0eb079b50b8edbf99a68a1d01ee82d73c5286d5e9ba`.
- Typed host snapshot composition is now the single server authority; exact old serialized behavior, Core/store
  ownership, public/data/save/server/Unity compatibility and all dirty-work boundaries are preserved.
- The audit file alone explains the 530→531 status-row increase. The protected marker remains exact.
- Phase 31 is complete. Phase 32 is ready only and must pass its own initialization, policy/history reread, sole
  activation and prospective-fingerprint gates before any production inspection or mutation.
