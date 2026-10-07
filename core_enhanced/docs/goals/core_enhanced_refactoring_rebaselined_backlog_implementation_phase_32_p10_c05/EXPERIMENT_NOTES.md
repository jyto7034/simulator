# Phase 32 CRA-P10-C05 Experiment Notes

Status: complete
Implementation Complete: true

## N01 — Structural Boundary

- `GameError` owns semantic error identity. The new authority may return only the stable code already associated
  with that identity; it must not choose which error occurs or alter parameter detail.
- Core snapshot and server adapters retain distinct contextual messages and structured fields. Message duplication
  is not part of this candidate merely because it appears near code duplication.
- Domain/preflight validation, rollback and publication remain separate authorities. Centralizing a code must not
  become a catch-all error conversion or recovery path.
- Phase 33 result projection and Phase 34 battle DTO transport remain pending and must not be prebuilt.

## N02 — Exact Compatibility Boundary

- Every current snake-case code is external protocol vocabulary and is frozen by CRA-POL-006.
- `ActionUnavailable` reason-as-code behavior is part of the same equality proof, but disabled-reason selection
  remains owned by the current capability evaluator.
- Parameterized errors need representative values to prove code invariance while retaining exact structured detail
  and messages.
- No `unknown`, wildcard/default, compatibility alias or dual old/new mapping is permitted; the authority must be
  exhaustive over the semantic enum.

## N03 — Dirty Worktree Precautions

- Initialization baseline is 531 expanded rows with normalized status SHA-256
  `5b566e36b4107b89ea83a1ed84b7ff10a68ccad0017e930af073fd5ee0cf836d`.
- These three Child files are the only expected new paths before activation. Existing Core/server source, tests,
  canonicals and Goal history belong to the user/completed campaign and must not be reset, restored or broadly
  reformatted.
- The 57-byte Phase-27 marker is protected at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`; its one-path exception does not expand.
- Capture prospective target hashes and current status before the first new Phase-32 production-content read.

## N04 — Validation And Remaining Risk

- The decisive safety net is an exhaustive old-production cross-layer table, not a unit test of a newly added
  accessor. It must prove both Core projections and server mapping agree before consolidation.
- Context-specific messages and structured fields need independent assertions so code reuse cannot silently move or
  normalize their ownership.
- Unknown deployed Rust/Unity consumers do not block a code-identical private consolidation, but any code or shape
  difference is a stop condition.
- Exact variants, parameter categories, disabled reasons, visibility, paths, tests and minimal code representation
  remain discovery pending after sole activation.

## N05 — Initialization Boundary

- Phase 32 is initialized only. Master still marks it `ready`; no implementation Child is active.
- Historical Phase-10 observations and Phase-31 closure are dependency evidence, not permission to skip Phase-32
  activation, prospective fingerprints, current diff attribution or baseline.
- No Phase-32 production inspection or test has occurred. Sole activation is the next permitted transition.

## N06 — Activation Boundary

- Phase 32 is now the sole active Child; Phase 33 and all later work remain pending.
- The 534-row post-initialization status and hash in E02 are the activation document baseline.
- Active-document reread and prospective target fingerprints must close before new production detail is read.

## N07 — Prospective Dirty Collision Map

- The Core error carrier, both snapshot/capability projections, Core focused tests, server state/handler/message
  adapter and principal canonicals are already modified. Their exact activation hashes are recorded in E03; no
  Phase-32 hunk is yet attributed to them.
- `player_game_actor/session.rs` and `game_server_core_bridge_goal.md` are clean prospective evidence surfaces. A new
  status row on either requires an exact Phase-32 reason; prefer not to touch them unless current evidence demands it.
- The earlier snapshot-error cleanup Goal may explain some present architecture but cannot replace current source,
  caller, output or test evidence.
- The smallest expected production change is an exhaustive stable-code authority at the semantic Core error plus
  call-site replacement in existing Core/server adapters. Do not move messages or failure behavior to avoid a dirty
  file.

## N08 — Current Error Boundary

- Current source has 26 `GameError` identities and 41 typed disabled reasons. `ActionUnavailable` deliberately uses
  the disabled reason as its externally visible error code, so an enum-level code method must preserve that dynamic
  branch rather than assign `action_unavailable`.
- Only code identity consolidates. Core's lowercase snapshot messages and server's presentation messages are both
  complete exhaustive matches and must remain separately authored.
- Parameterized resource/stat/static-data/feature/fragment/authority values affect messages/details but not codes.
  Characterization must use nontrivial values so discarding or moving those details is observable.
- The previous snapshot-error Goal is provenance for `StaticObstacleBlocked`, not evidence that the current full
  code map has one owner.

## N09 — Frozen Characterization

- Both old-production layers now pass 66-row tables: 25 fixed variants plus every one of the 41 current disabled
  reasons through `ActionUnavailable`.
- The server error wire discriminator is lowercase `"error"`; the first new test deliberately records and then
  corrects its mistaken uppercase expectation. Production was not changed to fit the test.
- Exact Core and server messages are intentionally different for most variants and are frozen separately. The
  implementation must remove only code tuple literals, never either message match.

## N10 — Implemented Ownership Boundary

- `GameError::code()` is the sole stable semantic-code owner. Its exhaustive match has 26 arms: 25 fixed literals
  and the pre-existing dynamic disabled-reason branch.
- Core snapshot and server conversion now consume that method. They continue to own complete, context-specific
  message matches; message normalization remains deliberately out of scope.
- The public accessor is additive only because the server crate consumes Core. It changes no DTO or externally
  serialized value and introduces no wire code type or migration.

## N11 — Validation Closure

- Both 66-row characterization tables pass in debug and release after consolidation. Focused snapshot/server suites
  and representative stale/missing authority, revision reuse and future-resync failures also pass.
- Core all-target behavior is 1,074/1,074 in debug and release; server is 61/61 in both profiles. Workspace checks,
  final format and scoped diff hygiene pass.
- Clippy succeeds with the same pre-existing Core 82/134 and server 3/7 warning baselines. No new warning touches
  the Phase-32 code.
- Accessible Unity readers do not own a duplicate `GameError` code map. Local and external canonical hashes are
  unchanged because observable behavior did not move.

## N12 — Status-Hash Correction

- The actual 534-row canonical C-sort status hash is `a937cc3f...`, not the derived `7f12bd4a...` recorded during
  activation. Removing the three Phase-32 Child paths and Phase-31 audit exactly reconstructs the independently
  recorded 530-row Phase-31 activation set at `848bd871...`.
- The correct intermediate hashes are `da983cae...` for 531 rows after the Phase-31 audit and `a937cc3f...` for 534
  rows after the three Phase-32 documents. Thus the discrepancy is an inherited/derived hash-recording error, not
  an unexplained path mutation.
- Retain both the historical entries and this explicit correction for auditability. Completion evidence must cite
  the corrected reconstruction and must not repeat the superseded hashes as proof.

## N13 — Completion Boundary

- The independent 286-line audit returns `Complete / High`; every P32-C01~P32-C12 condition passes and improvement
  class is `none`.
- The 535th status row is only `COMPLETION_AUDIT.md`. Production, canonical, external-consumer and protected-marker
  evidence remains exact after the audit-time reruns.
- Phase 33 is `ready`, not active. Its result-projection work requires a new Child, full dependency/history/policy
  reread, sole activation and prospective fingerprints before any production inspection.
