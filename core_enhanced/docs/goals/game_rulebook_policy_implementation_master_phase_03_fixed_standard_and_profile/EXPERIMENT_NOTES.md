# Game Rulebook Policy Implementation Master Phase 03 — Notes

## Structural Judgment

- Fixed Standard and successor profile are one atomic package because F3 completion and tutorial unlock must have a
  crash-recoverable both-or-neither host publication boundary.
- Internal data/runtime/store work may use target-first units, but no partial fixed map or profile schema is a Phase
  completion state.
- P02's exact Active Run V4 and encounter progression graph are stable dependencies, not P03 migration targets.

## Source-Of-Truth Boundary

- Strict live RON owns the three fixed tutorial maps, their content/tuning and exact reward pools.
- Core owns mode admission, fixed run progression, result semantics and one permanent profile reducer/projection.
- game_server owns successor profile bytes, bounded V1 migration, crash recovery and publication ordering.
- Checkpoint owns run-local Standard state; it never rolls back permanent profile maxima/flags/sets.

## Dirty Worktree Warning

- Activation inherits a broad pre-existing dirty tree across facility/map/world/data/docs and game_server.
- P03 will capture exact target diffs before editing and classify overlap with current user work. It will not reset,
  stash, restore, delete or bulk-format unrelated paths.
- If fixed-map/profile/store target hunks cannot be merged safely, the affected production mutation stops.

## Consumer Scope

- Confirmed development boundary: Core, production live RON and game_server.
- Unity inspection/mutation/runner remains explicitly excluded by user direction. No ignored field, false tutorial
  completion or compatibility fallback will be added for it.
- Public deployment is not currently planned; crash/restart and persistence gates remain mandatory internally.

## Pending Investigation Units

- Fixed authoring representation and strict cross-catalog validation for exact slot payloads
- Run-local deathless Floor state and the precise employee-death observation seam
- Current live FinalBoss membership derivation from strict BossOmen/PVE catalogs
- Successor transaction-journal failpoints and startup recovery wiring

## Alternatives Not Applied

- No adaptive Standard fallback or procedural-to-fixed hybrid.
- No FinalBoss on Standard F3 and no replacement boss/content selection.
- No scripted tutorial fragment in the minimum bootstrap.
- No Nothing There pattern redesign or global 60% runtime multiplier.
- No permanent V1 dual reader/writer, semantic set transform or partial profile reset.
- No profile fields inside Active Run/checkpoint.
- No P04 sweep command/schema and no WP005 fragment-skill content.

## Initial Risks

- Existing Standard/facility code may assume terminal role and adaptive generation are shared with Endless.
- An actor-only stat override may require a new strict data seam without leaking into other waves or P02 difficulty.
- Standard completion and profile durability currently span two aggregates; exact crash-safe transaction ownership is
  discovery pending.
- V1 current/backup migration must preserve dormant IDs and survive interrupted successor publication without a
  permanent compatibility reader.
- Record events may currently lack a single host publication point or exact deathless definition state.
- Broad dirty overlap may make safe attribution difficult.

## Activation Findings

- The three approved topology IDs and every reused encounter/event/shop/HQ reference are live. Only the three new
  single-entry reward pools are missing.
- Weighted profile assignment is the sole reason Standard is adaptive. A fixed slot-authoring branch can leave
  Endless generation unchanged and keep one strict map representation.
- F3 currently uses `RunFinalBoss`, and map encounter selection separately promotes a final Standard Boss to
  `FinalBoss`; both authorities must change together to `StandardCompletion` plus `NormalBoss`.
- Nothing There base stats are exact `1200/88/60/55`. The current `BattleUnitStatScale` lacks magic resistance and
  applies encounter-wide, so it is explicitly unsuitable for BOOT-10.
- Compendium V1 owns exactly the four approved sets. Its server transaction is independent from Active Run V4,
  confirming that a shared durable terminal intent/recovery boundary is necessary.
- The immediately preceding P02 full suite is the P03 baseline (Core 1055, server 63) because this Phase performed
  only document creation and read-only inventory before activation.

## Cross-Phase Impact

- P03 must preserve P01's checked 150/200 Monolith tuning and P02's V4/source/BossOmen semantics.
- P04 depends on the final permanent/profile serialization seam being stable, but P03 must not pre-author sweep
  fields or commands.
- Master final audit will combine Standard/profile with V4/sweep; P03 evidence must therefore distinguish run-local
  checkpoint state from permanent monotonic state exactly.

## Resolved Implementation Shape

- `fixed_slot_assignments` is a full slot/content authoring mode, not an extra hint layered on adaptive assignment.
  Fixed/adaptive fields are mutually exclusive and publication rejects incomplete or inconsistent maps.
- `tutorial_fixed_v1` preserves the complete difficulty schema while declaring `authored_fixed: true`; all tuning is
  100% and fixed Standard keeps Floor anchor only for checkpoint-generation integrity, never enemy scaling.
- Exact actor overrides are node-local authored combat policy. Lookup requires the same slot, kind and payload, and
  only a matching abnormality draft receives the four-axis exact override.
- `PlayerProfileState` is the sole permanent gameplay aggregate. Its nested compendium unions sets; tutorial and
  all-live-FinalBoss flags OR; numeric records max. Run checkpoint/rollback never owns or replaces it.
- Highest Floor and deathless streak are Endless-only. The all-live-FinalBoss record uses completed BossOmen chain
  IDs from one current Endless run, not cumulative compendium defeat state.
- Active Run V4 now requires `record_progress`. This is an intentional development-only exact-shape addition, not a
  compatibility default; prior writer V4 payloads reject.
- PlayerProfile V2 current files replace compendium V1. V1 parsing is bounded to the old filenames for one-shot
  four-set preservation, followed by durable V2 publication and legacy cleanup.
- Standard completion uses a durable intent journal. Until profile publication, Active Run invalidation and journal
  cleanup all finish, the host exposes `completion_recovery_pending`, blocks continue/ordinary commands and allows
  only typed retry. Full profile reset is not allowed to bypass the pending intent.

## Exit Findings

- Direct ST-03 composition and same-process journal fail-closed gaps were found during the independent review and
  corrected before the completion verdict; both have dedicated regression tests.
- P02 exact V4 graph, encounter progression and BossOmen filtered suites remain green after P03.
- Current Core total is 1065 tests and game_server total is 71, all passing with zero ignored.
- Canonical ownership now describes fixed Standard, PlayerProfile V2, run-local record progress and terminal journal
  recovery. Goal documents remain execution evidence rather than durable gameplay authority.
- Unity is explicitly unverified/unmodified. This is a bounded consumer omission, not a compatibility claim or
  hidden fallback.

## Remaining Non-Material Risk

- The bootstrap values and content are intentionally provisional and may be tuned during development.
- Active Run's additive exact V4 field rejects earlier development V4 files. This is documented and approved by the
  no-public-rollout constraint; there is no silent migration.
- The repository remains broadly dirty, so P04 must repeat hunk-level ownership checks and avoid bulk cleanup.
