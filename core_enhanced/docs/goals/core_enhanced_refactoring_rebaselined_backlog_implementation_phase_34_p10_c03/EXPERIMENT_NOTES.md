# Phase 34 CRA-P10-C03 Experiment Notes

Status: complete
Implementation Complete: true

## N01 — Structural Boundary

- Core setup/update DTOs own complete battle content. Server owns message selection, resync wrapping, request
  correlation and publication order; the refactor must not collapse those different responsibilities.
- Reuse means storing/composing the whole Core DTO once. A custom serializer or helper that repeats every field is
  still duplicate schema ownership even if it moves the match.
- Exact top-level JSON, not a preferred Rust enum shape, determines the acceptable representation.

## N02 — Compatibility And Policy Boundary

- CRA-POL-005 freezes the single deterministic union preview collection already delivered by Phase 18.
- CRA-POL-006 freezes current setup/update tags, top-level fields, nested resync and public/API compatibility.
- Private Boss scheduler/session/wake state remains non-serde. A first public Boss lifecycle requires a separate
  producer/consumer/version policy and is not implied by DTO reuse.

## N03 — Test-First Boundary

- The decisive first proof is complete old-production JSON for setup, update and nested resync using real rich Core
  DTOs. Selected-field assertions are insufficient.
- Exact key paths, omitted/null behavior, nested object values and actor/reconnect sequence all matter.
- Test private Rust representation only where needed for construction; the long-lived contract is output and owner
  separation.

## N04 — Dirty Worktree Precautions

- Initialization baseline is 539 expanded rows at canonical C-sort SHA-256
  `f17ab3657507e3cc39e91ac7136b8851d9db346cfeb421cdfbf5b79239b971cc`.
- These three Child files are the only expected new paths before activation. Existing Core/server sources, tests,
  fixtures, canonicals and Goal history are user/completed-campaign work and must not be reset or broadly formatted.
- The 57-byte protected marker remains at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`.
- Capture prospective target hashes/status before the first new Phase-34 production-content read.

## N05 — Initialization Boundary

- Phase 34 is initialized only. Master still marks it `ready`; no implementation Child is active.
- Phase-18/33 and historical P10 observations are dependency evidence, not permission to skip activation,
  prospective fingerprints, current diff attribution or old-production characterization.
- No new Phase-34-directed production/test/data/canonical/external inspection or test occurred. Sole activation is
  the next permitted transition.

## N06 — Activation Boundary

- Phase 34 is now the sole active Child; Phase 35 and all later work remain pending.
- The 542-row post-initialization status and `e1a402f6...` canonical C-sort hash are the activation document baseline.
- Active-document reread and prospective target fingerprints must close before new production detail is read.

## N07 — Prospective Dirty Collision Map

- Core setup/update definitions/builders/tests and the server message/state/handler adapter plus battle fixture are
  already modified. Their exact activation hashes are recorded in E03; no Phase-34 hunk is yet attributed.
- Server session, both manifests and the server bridge goal are clean. A new status row on any requires an exact
  Phase-34 reason.
- Local canonicals are already dirty and external Unity is heavily dirty. Preserve hashes and avoid edits unless
  current runtime evidence proves a relevant statement stale.
- The expected smallest production target is the current server message representation/constructors. Do not move
  Core builders or actor order merely to avoid a dirty file.

## N08 — Current Authority And Baseline Finding

- Core owns 8 update and 16 setup serialized fields. Server repeats 7 and 15 respectively; the missing field in each
  count is the same top-level `type` already owned by the Core DTO.
- Retain the server-owned resync envelope and its routing UUID. Remove only the standalone setup/update field lists
  and destructuring.
- Three complete `Value` comparisons pass on old production before implementation; 20 actor handler tests close the
  old order baseline.
- Unity's checked-in revision fixture is positively consumed but stale relative to current Rust output. It is a
  pre-existing fixture baseline failure, not evidence to regress Rust or add compatibility omission. Close its exact
  consumer-aware disposition before completion.

## N09 — Implemented Ownership Shape

- Server standalone update/setup variants now contain the complete Core DTO and delegate serialization directly.
  There is no server content field list, destructuring constructor, flatten mutation or custom serializer.
- Variant-level untagged delegation is limited to the two Core-tagged DTOs at the end of the internally tagged enum;
  all server-owned variants, including resync, retain their existing representation.
- Production delivery tracking reads UUID/sequence from the contained update. Selection, response/order, resync,
  reconnect and final-snapshot owners did not move.
- Exact text plus `Value` equality proves one `type` key and unchanged field order/shape for standalone DTOs.

## N10 — Runner And Diagnostic Classification

- Do not run Core debug/release suites concurrently while both can write the same diagnostic trace JSON. The one
  observed EOF is a runner collision; sequential debug and release each pass all 1,077 tests.
- Server debug/release each pass 62 tests. Workspace checks and format are green.
- Clippy remains diagnostic-only and unchanged: Core 82/134 and server 3/7 warning baselines in both profiles.

## N11 — External Fixture Disposition

- The external Unity revision fixture is already untracked and stale, and its activation hash remains exact. Phase
  34 has no authority to overwrite it and no Unity source/shape change is required.
- The producer's current JSON remains valid and the exact Unity fixture-consumer test passes 1/1. Unity uses raw
  JSON objects and tolerates the newer fields; the mismatch is fixture-maintenance debt, not a wire incompatibility.
- Record the generator comparison as an expected pre-existing failure, not as a green exact-file comparison. The
  decisive Phase-34 parity evidence is the same complete old/new Rust serialization tests plus unchanged generator,
  consumer and external hashes.

## N12 — Completion-Audit Boundary

- Main status remains the exact 542-row activation set until the audit document is created. Candidate edits stay
  within the three already-dirty server actor files; Core DTOs, generator, canonicals and Unity remain unchanged.
- Phase 35 must remain absent and pending until a separate audit independently checks every P34-C01~C12 condition
  and returns `Complete / High` with no material finding.

## N13 — Completion Boundary

- The independent 322-line audit returns `Complete / High`; every P34-C01~P34-C12 condition passes and improvement
  class is `none`.
- The 543rd main status row is only `COMPLETION_AUDIT.md`. Production, local canonical, external-consumer and
  protected-marker evidence remains exact after the audit-time reruns.
- The external stale fixture/example remains read-only maintenance debt and not a transport incompatibility; its
  exact consumer passed and no Phase-34 correction is required.
- Phase 35 is `ready`, not active. Its cleanup work still requires Child initialization, dependency/history/policy
  reread, sole activation and prospective fingerprints before any production inspection.
