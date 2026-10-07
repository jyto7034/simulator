# Phase 30 CRA-P10-C01 Experiment Notes

Status: complete
Implementation Complete: true

## N01 — Structural Boundary

- This Phase centralizes facts, not execution. The descriptor may answer route owner, scope and battle-authority
  need; it must not answer state-dependent availability or perform a mutation.
- Core-common and Core-battle domain evaluators remain the source of typed capability/disabled-reason truth.
- `ContinueRun`, `ResetActiveRun`, `RetryEncounterCompendium` and `ResetEncounterCompendium` remain server-host
  operations carried by the Core vocabulary. A typed route must expose that boundary without sending them through
  Core domain dispatch.
- Phase 31 CRA-P10-C04 depends conservatively on the stable descriptor but is not prebuilt here.

## N02 — Compatibility And Policy

- CRA-POL-006 freezes command serde names, routes, result/error/snapshot shapes, top-level setup/update, nested
  resync and public API compatibility.
- No DTO, RON, V3/save, wire, Unity or Boss-runtime migration is approved or expected.
- Availability, payload validation, revision preflight, persistence and publication ordering are retained distinct
  authorities; their duplication is not removal scope merely because they inspect the same command.
- There is no open policy question. Any observable difference or proposed owner change is a stop condition.

## N03 — Dirty Worktree Precautions

- Initialization baseline is 522 expanded rows with normalized status SHA-256
  `3ec521d59411b6a660dcd28f24510332f9725677ca4ddbebb2780657d6f4ef7c`.
- The Phase 30 Child files are the only expected new paths at initialization. Existing dirty source/test/canonical
  work belongs to the user or completed campaign Phases and must not be reset, restored, reformatted broadly or
  rewritten outside candidate hunks.
- The 57-byte Phase 27 approval marker is protected at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`; its exception scope does not expand.
- Prospective target hashes and scoped diffs must be recorded after activation and before detailed production reads.

## N04 — Validation And Remaining Risk

- The decisive new safety net is exhaustive descriptor plus projected-versus-executable parity, passing against old
  duplicated production before extraction.
- Existing representative tests remain distinct: serde vocabulary, common/battle capability, VN/revision
  no-mutation, host recovery persistence and actor ordering do not replace one another.
- Unknown deployed Rust/Unity consumers remain a compatibility risk, but not an implementation blocker while all
  public/serialized behavior remains exact.
- Exact current file ownership, test commands/counts and whether the descriptor can remain private are discovery
  pending after sole activation.

## N05 — Activation Boundary

- Phase 30 is the sole active Child. Phase 31 remains pending and absent.
- Activation authorizes current-tree investigation, characterization, candidate-scoped implementation and
  validation for CRA-P10-C01 only.
- The 525-row post-initialization status and hash in E02 are the activation document baseline; prospective target
  hashes must be added before production details are read.

## N06 — Prospective Dirty Collision Map

- `behavior.rs`, `world.rs`, `battle_capability.rs`, `snapshot.rs`, `state.rs`, three player-game actor files and
  several canonicals are already dirty. Their full activation hashes are recorded in E03; no Phase 30 hunk is yet
  attributed to them.
- `action_capability.rs`, both load-balance files, player-game session and the server bridge canonical are clean
  prospective targets. A new status row in any of them must be explained exactly.
- The command descriptor visibility cannot be chosen from historical evidence alone. Prefer the narrowest type that
  lets the direct server consume route ownership without exposing a new serialized or broadly public contract.

## N07 — Current Route Model

- A three-value Common/Battle/Host enum would erase two real distinctions. Ten common-projected commands retain
  payload/domain preflight, while `AdvanceVisualNovel` and `SelectVisualNovelChoice` switch between common and
  battle shared preflight by current context.
- The smallest exact typed route therefore distinguishes `CoreCommon`, `CoreDomain`, `CoreBattle`,
  `CoreContextual` and `HostRecovery`. This is metadata, not a new execution path.
- The exhaustive characterization table freezes all 56 rows before implementation. The old world dispatch match
  remains because it extracts payloads and selects domain mutation handlers; treating it as removable metadata
  would exceed the candidate and weaken type safety.

## N08 — Implemented Ownership Boundary

- A private exhaustive descriptor is the only owner of route, scope and battle-authority facts. The public
  `ActionKind::route()` and two additive non-serde route types are the smallest compile boundary that lets the
  separate server crate consume host ownership without parsing behavior variants.
- The 36-entry common array is retained solely because vector order is observable. Descriptor membership owns which
  commands belong there, and bidirectional debug checks prevent the ordered view from becoming a second membership
  authority.
- The 18-entry battle evaluator remains the state-dependent availability and order authority. Its bidirectional
  descriptor parity closes omission drift while preserving exact reasons and domain revalidation.
- The first implementation sketch would have projected common actions in enum order. Review caught that this would
  reorder the existing capability vector, so the retained implementation preserves the historical order. This is a
  design correction, not a runtime or test failure.
- Host recovery routing is typed, but compendium/reset/continue lifecycle branches remain in their original server
  order. A no-op exhaustive `HostRecoveryAction` match intentionally forces future server review if a host operation
  is added.

## N09 — Validation And Audit Handoff

- Final implementation validation is green in Core/server debug and release: Core all-target 1,073 tests each,
  server all-target 59 each, all-target checks and Clippy exit 0, and format/scoped diff checks exit 0.
- Existing Clippy warnings are recorded as diagnostic baseline and are not candidate regressions or cleanup scope.
- The implementation adds one status row because activation-clean `action_capability.rs` became a typed consumer;
  the pre-audit tree is 526 rows at normalized hash `882d5f6b...`.
- The protected marker remains exact. No data/save/wire/Unity/Boss or Phase 31 surface changed.
- Completion remains false until a separate review follows the completion-review guide and returns
  `Complete / High`; the implementation evidence alone does not advance the Master.

## N10 — Release Membership Authority Correction

- Debug-only parity is not sufficient to claim release ownership. The first audit correctly rejected the version
  that iterated the ordered array directly even though all current rows matched.
- The corrected design derives membership from the exhaustive descriptor in every build and applies the historical
  array only as a sort key. This preserves current order without leaving a second release membership authority.
- This was a material audit finding, so all earlier completion-review work is non-verdict evidence. The audit must
  restart after correction and rerun broad debug/release validation.

## N11 — Completion Disposition

- The restarted audit verified the corrected release authority and returned `Complete / High`; no immediate
  correction or follow-up refactor remains for CRA-P10-C01.
- Phase 31 may depend on the stable nonserialized route seam, but it must initialize independently and may not treat
  the retained common order or server lifecycle as already migrated host-overlay state.
- Preserve the 527-row post-audit status set and protected marker during the next activation protocol.
