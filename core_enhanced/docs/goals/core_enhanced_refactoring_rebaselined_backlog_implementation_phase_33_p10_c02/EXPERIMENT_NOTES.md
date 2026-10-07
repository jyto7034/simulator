# Phase 33 CRA-P10-C02 Experiment Notes

Status: complete
Implementation Complete: true

## N01 — Structural Boundary

- `BehaviorResult` owns semantic outcome identity. The target projection may derive only the current stable command
  contract; it must not decide command success, execute mutations or move server persistence/publication ownership.
- Result type, payload and transport classification belong to one Core projection seam. Core battle DTO contents and
  server setup/update/resync field composition remain Phase-34 responsibilities.
- Nested completions are not an excuse for parallel semantic ownership. Their exact current nesting and
  direct-map/wrapped shapes must be characterized before choosing a representation.

## N02 — Compatibility And Public API Boundary

- CRA-POL-006 freezes all current result strings, payloads, nesting and the direct-map `RunComplete` object.
- A public helper/type with no workspace caller is still an unknown-deployment API. It may be retained or privately
  bypassed, but not removed/narrowed without the required consumer/migration evidence.
- No alias, legacy projection, fallback serializer or dual old/new result map is approved.

## N03 — Dirty Worktree Precautions

- Initialization baseline is 535 expanded rows at canonical C-sort SHA-256
  `4f8c208d7f99ca7c1001fc0113910de5b80df2ec85e68e083e88cc2266a380f9`.
- These three Child files are the only expected new paths before activation. Existing Core/server source, tests,
  canonicals and Goal history belong to the user/completed campaign and must not be reset, restored or broadly
  formatted.
- The 57-byte Phase-27 marker is protected at SHA-256
  `614d2fbdbdb0b29def7b90d1fdbc65148aba63339eea72fef5998d4a8ec22730`; its one-path exception does not expand.
- Capture prospective target hashes and current status before the first new Phase-33 production-content read.

## N04 — Validation And Remaining Risk

- The decisive first proof is an exhaustive old-production table over every semantic result, not a unit test of the
  new projection helper. It must cover exact payload values, transport class, nesting and snapshot decisions.
- `RunComplete`, nested completions, preview/read-only and battle accepted/update paths require explicit rows because
  their shapes/lifecycles are not interchangeable.
- Exact server serde and actor order are required even if the duplicate representations are Core-internal.
- Exact current variants, public helpers, construction paths, intermediate count, consumers and minimal target type
  remain discovery pending after sole activation.

## N05 — Initialization Boundary

- Phase 33 is initialized only. Master still marks it `ready`; no implementation Child is active.
- Historical Phase-10 observations and Phase-32 closure are dependency evidence, not permission to skip Phase-33
  activation, prospective fingerprints, current diff attribution or baseline.
- No new Phase-33 production inspection or test has occurred. Sole activation is the next permitted transition.

## N06 — Activation Boundary

- Phase 33 is now the sole active Child; Phase 34 and all later work remain pending.
- The 538-row post-initialization status and `aa6ac2bf...` canonical C-sort hash are the activation document baseline.
- Active-document reread and prospective target fingerprints must close before new production detail is read.

## N07 — Prospective Dirty Collision Map

- The primary Core result carrier, world dispatch/state and focused tests plus server state/message/handler adapters
  are already modified. Their exact activation hashes are recorded in E03; no Phase-33 hunk is yet attributed.
- Core public export/module/manifest surfaces, server session and the server bridge goal are clean. A new status row
  on any requires an exact Phase-33 reason; public API evidence should remain read-only unless the target truly
  requires an additive surface.
- Local canonicals are already dirty and external Unity is heavily dirty. Preserve their exact hashes and avoid
  edits unless current implementation evidence proves a relevant contract statement stale.
- The smallest expected production target is within the current Core result projection plus its server consumer
  adaptation. Do not move actor ordering or prebuild Phase-34 DTO composition to avoid a dirty file.

## N08 — Current Authority And Retention Decision

- The semantic vocabulary is 47 variants: 35 command payload, 10 battle update and two read-only preview.
- The duplicate output authority is the 11-domain-enum layer's separate type/payload matches plus the server's
  independent ten-variant battle-update extractor. The terminal-run persistence match is semantically distinct.
- Preserve the public outer/intermediate/domain types and existing helper signatures for unknown Rust consumers.
  Compatibility must be achieved by delegation to the central source, not by retaining a second output table.
- The exhaustive test oracle may derive ordinary field JSON from `BehaviorResult`'s independent serde shape, but
  must explicitly encode the four exceptional payload rules: direct-map `RunComplete`, direct inner-result
  equipment/preview payloads, unit-result empty/null values and recursive typed completions.

## N09 — Implemented Ownership Shape

- `BehaviorResult::command_result_projection` is the only production table that maps semantic results to stable
  result strings, payload values and transport class.
- Public legacy/domain result types remain because absence of workspace callers is not removal authority. They are
  structural compatibility adapters only: output calls reconstruct a semantic result and delegate to the central
  authority. No retained domain enum owns a result string or JSON payload match.
- Server command mapping and live-tick update extraction consume the same central enum. Actor ordering and terminal
  persistence invalidation remain separate invariants.
- Cloning the typed battle update at the borrowed projection seam changes no wire/state behavior and avoids a second
  battle-variant extractor. Phase 34 still owns any later DTO field de-duplication.

## N10 — Validation Closure

- Final Core broad behavior is 1,077/1,077 in debug and release; server is 62/62 in both. Workspace checks, format
  and candidate-scoped diff hygiene pass.
- Core/server normal Clippy return to their inherited 82/134 and 3/7 warning baselines after boxing only the new
  battle projection branch and using server auto-deref. No Phase-33 warning remains.
- Static proof finds 35 command rows, two read-only rows and ten battle rows in the single central authority; the 11
  retained public domain enums own no stable string or payload table.
- Main status is still the exact 538-row activation set at `aa6ac2bf...`; local and accessible external
  fingerprints outside the two intended production files are unchanged. The protected marker is exact.
- Phase 34 remains pending and absent. A separate current-tree audit is the only remaining Phase-33 gate.

## N11 — Completion Boundary

- The independent 311-line audit returns `Complete / High`; every P33-C01~P33-C12 condition passes and improvement
  class is `none`.
- The 539th status row is only `COMPLETION_AUDIT.md`. Production, canonical, external-consumer and protected-marker
  evidence remains exact after the audit-time reruns.
- Phase 34 is `ready`, not active. Its battle DTO de-duplication work requires a new Child, full dependency/history/
  policy reread, sole activation and prospective fingerprints before any production inspection.
