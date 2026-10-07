# Phase 37 CRA-P06-C04 Experiment Notes

Status: complete
Implementation Complete: true

## N01 — Assertion Transfer Boundary

- Six green self-tests do not prove production relevance. Each useful predicate must first pass against a log made by
  the current `BattleCore` and retain a malformed negative derived by changing one edge of that generated control.
- Version, sequence/order, parent/cause, spawn/reference, buff/death and cadence are candidate assertion families;
  current evidence decides which are useful and which are stale or unread.
- Zero-selected filters, compile success and hand-built synthetic-only fixtures are not deletion evidence.

## N02 — Runtime Authority Boundary

- The committed `BattleCore` log owns sequence and relationship facts. The Phase must not wire the disconnected
  validator into production or publication merely to make it appear used.
- Runtime event-log types/version/serde, battle mechanics, result aggregation, live deltas, resync/compression and
  debug trace remain unchanged.
- `source_command_id` or equivalent correlation must not be reinterpreted as causal sequence without current proof.

## N03 — Public Compatibility Boundary

- CRA-POL-006 forbids removing or narrowing public Rust/module/package surfaces from workspace caller absence alone.
- Affirmative consumer inventory plus an already approved deprecation/versioned migration, or explicit breaking
  approval, is required when cleanup changes the public surface.
- If that evidence is unavailable, the correct Phase outcome is `externally_blocked`; retaining a false authority
  and calling the candidate complete, or adding a shim, is not allowed.

## N04 — Cross-Phase Boundary

- Phase 36 is the direct linear predecessor and made no validator change or placeholder.
- Phase 41 separately owns the public ghost `ScenarioEvent.once` field.
- Phase 42 separately owns battle-log read/write/debug-I/O helper cleanup, and Phase 44 owns package targets.

## N05 — Dirty Worktree Precautions

- Initialization baseline is 553 expanded rows at canonical C-sort SHA-256
  `729b1a8b113f67de2fc0662ce54481fedbf39b5316c3b2f0460e53a9d918e086`.
- All existing rows are user/completed-campaign work. Completed Phase-36 changes and its four Child artifacts are
  preservation inputs, not Phase-37 cleanup targets except where current exact overlap is later fingerprinted.
- The protected marker remains 57 bytes at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730` with exact timestamps.

## N06 — Initialization Gate

- Phase 37 is initialized only; Master still marks it `ready` and there is no active implementation Child.
- No prospective current validator/source/test content or caller/config/consumer graph has been inspected.
- Sole activation, complete active-document/predecessor/history reread and prospective path-only fingerprints are
  next.

## N07 — Completion Semantics

- Assertion transfer alone is necessary but not sufficient: the disconnected authority must be removed or
  depublished within CRA-POL-006 for `Complete / High`.
- If public removal remains externally unverifiable, preserve current API and runtime behavior, document the exact
  missing evidence, keep Phase 38 pending and stop automatic progression.

## N08 — Activation Gate

- Phase 37 is now the sole active Child; Phase 38 remains pending and absent.
- The 556-row post-initialization set at `e52f642d...` is the active-document baseline.
- Before reading prospective current validation/runtime/test content, fully reread the active Child, Phase-36 audit
  and governing history, then record filename-only/path-known prospective fingerprints for every likely validator,
  producer, consumer, manifest, data and canonical input.

## N09 — Prospective Dirty Collision Map

- `validation/buffs.rs`, `deaths.rs`, `spawns.rs` and `validator.rs` are already modified; the other ten validation
  files are clean. Any removal or transfer must first distinguish those four pre-Phase hunks exactly.
- Event-log types and most Core producer/test carriers are already modified by prior campaign work. `recording.rs`,
  `combat_trace.rs`, the skill-run helper and manifests/library roots are clean.
- The three nearest canonicals and `ron_loading.rs` are already modified. No content/data/canonical change is
  expected for this cleanup.
- Do not use cleanup as a reason to rewrite or reformat adjacent dirty battle files.

## N10 — Investigation Authority

- Activation reread and path-only fingerprints are complete; current content, symbol/reference, predicate and test
  inspection is now allowed.
- Begin with exact scoped diffs for the four dirty validation files, then enumerate public exports, tests and all
  workspace/package/tool consumers before running a baseline or editing.

## N11 — Current Public Graph And Removal Block

- The current harness is still 14 files/2,926 lines, exported by `pub mod validation` with six public reexports and
  public validator construction/method surfaces. Its only accessible workspace callers are itself and its six
  tests; this remains negative evidence only.
- `game_core` is a potentially publishable library and the accessible tree does not enumerate standalone/deployed
  Rust consumers. CRA-POL-006 therefore blocks removal until explicit breaking approval or an approved migration.
- The unused `event_log_checks` feature is not a reason to extend scope: Phase 44 owns package/target cleanup.

## N12 — Produced-Log Predicate Disposition

- A real 17-entry delayed-wave log passes every default validator family except sequence, where the stale validator
  emits 17 violations because production starts at one and it expects zero.
- Version/boundaries, one-based sequence/time, prior parents, spawned references, HP/death and attack/cadence rules
  are useful and now have generated-log positives plus one-edge malformed negatives.
- Buff database/reapply/expiry and cast interrupt/cancellation relations now run through real producer tests.
  Movement-path configuration and synthetic optional-presence/count policies are not transferred.
- Manual interruption correlation belongs to `interrupted_cast_seq`; a failed first assertion proved that a generic
  cause-parent relation would invent behavior absent from the current producer.

## N13 — Authorized Stopping Point

- Assertion transfer changes only the already-dirty Core test carrier; runtime event types, recording,
  serialization, result/live/debug consumers, data, DTOs, saves, server and Unity are unchanged.
- Focused producer/validator controls, 108 Core-module tests and 1,074 Core all-target debug tests are green; format
  and diff checks are green. Final release/workspace/static validation follows an authorized cleanup, not a false
  completion claim.
- Keep all 14 public harness files and six tests until the user decides the breaking/migration policy. Keep Phase 37
  incomplete and Phase 38 pending.

## N14 — Required User Decision

- Status is `policy_decision_required`, not `active`: the unresolved choice narrows an external/public Rust
  contract and cannot be inferred from workspace absence.
- Recommended path: explicitly approve breaking removal of the disconnected public validation surface now that its
  useful assertions execute against actual producer logs. Alternative path: approve and define a deprecation or
  versioned migration plus the consumer inventory/rollout authority it requires.
- Until one path is approved, do not edit production, delete the harness/tests, run further validation as if cleanup
  were complete, create Phase 38 or call this Phase complete.

## N15 — Policy Resolved And Scope Of Approval

- Master PLAN row 234 explicitly records that no external consumer exists and the proposed cleanup should proceed.
  This is the required breaking-removal authorization under CRA-POL-006.
- Resume Phase 37 as the sole active Child. Remove only `pub mod validation`, the 14-file harness/public surface and
  its six self-tests whose useful assertions have already moved; preserve runtime log types/consumers and the Phase
  44 `event_log_checks` feature.
- Phase 38 remains pending until removal, exact discovery delta, broad validation and an independent `Complete /
  High` audit close Phase 37.

## N16 — Cleanup Result

- The disconnected public authority is absent: one module export, 14 implementation files, six public reexports and
  six self-tests were removed under the explicit approval. No replacement validator, shim, alias or fallback exists.
- The current test set is 1,068, exactly `+1` real producer test and `-6` synthetic validator tests relative to the
  pre-Phase 1,073 set. Exact name-table reconstruction proves no collateral test-name change.
- Core debug/release all-target, workspace debug/release checks, format/diff and Core Clippy pass. Runtime log,
  producer behavior, manifests and protected marker remain exact.
- The only remaining Phase condition is the independent completion audit. Do not activate Phase 38 before its
  `Complete / High` verdict and recorded exit transition.

## N17 — Phase Exit

- The independent audit returns `Complete / High` with all twelve rows passing, fit `High`, improvement class
  `none` and no material finding.
- Phase 37 is complete. Phase 38 is only ready; no Phase-38 Child or production inspection exists yet.
- Preserve the approved public break, producer-owned assertions, 1,068-test discovery baseline, audit artifact and
  protected marker as Phase-38 inputs.
