# Boss Runtime v1 Phase 00 — Notes

## Activation baseline

- Root: `/mnt/f/work/simulator`
- Start commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- Pre-creation full status: 221 entries
- Pre-creation status SHA-256: `41f02b5b73145453f032c3dfcf2931f1225db3225cb70a7ccaf82de5e882bd8d`
- The worktree is substantially dirty/untracked. Existing changes are user-owned and will not be cleaned, restored,
  stashed, reformatted or absorbed into P00 findings without source/authority evidence.

## Structural decision

- P00 is an investigation-only Phase and owns no production mutation.
- Direct predecessor is the completed readiness audit (`Complete / High`, `conditionally_ready`). There is no direct
  predecessor implementation Phase.
- Current audit findings must be distinguished from the 2026-08-08 readiness snapshot.
- P00 must finish and independently audit current drift, collisions, the 19-candidate registry and P01 handoff before
  any implementation child can become ready.

## External and consumer boundary

- Core and game_server are workspace producer/transport surfaces.
- `/mnt/f/unity projects/ark` is an accessible but external read-only consumer surface for P00.
- Unity was stale at readiness-audit time; P00 will classify only material changes relevant to replay/presentation or
  file/authority collision and will not treat consumer drift as Core authority.
- No Unity runner or mutation is authorized in P00.

## Policy gates that do not block P00

- First live reference Boss identity/content/presentation approval is required only before P05.
- P06 activation/not-applicable disposition follows the actual approved reference Boss public contract.
- Active-battle deployment lifetime becomes a policy question only if current topology contradicts the Master's
  no-in-place-migration assumption.

## Durable evidence protocol

- Each bounded investigation unit records commands, source paths, current observation, historical comparison,
  disposition and remaining gap before the next unit begins.
- Unresolved observations stay explicit; they are not silently converted into final findings.
- Conversation context and later reconstruction are not completion evidence.

## Current unanswered investigation questions

- Which relevant production files changed after the readiness audit, and do their actual semantics invalidate a prior
  owner/seam conclusion?
- Which active/incomplete Goals overlap P01 and later Boss boundaries in files, authority, migration or tests?
- Are C01→C02→C03 still the correct P01 milestone order and one atomic child boundary?
- Are P02/P03 and P05/P06 still the correct deployment/review lifetimes?
- What exact current test commands and failure-injection seams make P01 immediately executable?

## Collision finding — 2026-08-09

- Exact status search found no concurrent active production Goal. The only active pair is this P00 and its Master.
- Defense Route Tactical Command is complete, but its Core/live-data/server changes are still part of the user-owned
  dirty baseline and directly overlap every future Boss Phase.
- P01 may proceed after current-symbol characterization because Goal ownership is unambiguous. It must not use
  whole-file replacement or assume the start commit represents the current runtime.
- Unity is a separate, heavily dirty repository. Any P06 work must be re-baselined and synchronized as a separate
  producer/consumer unit; P00 remains read-only there.

## P01 G0 drift finding — 2026-08-09

- Current code still exhibits all three audited prerequisite defects; none may be dropped as stale audit debt.
- Provisional order remains C01 terminal authority, C02 total order, C03 rollback-safe world pipeline. C01 must shape
  the state-owned finish contract consumed by C03 rather than leaving two terminal authorities.
- C03 cannot be planned as an existing full-session clone transaction. Its handoff must identify testable preflight,
  staging and commit/rollback owners explicitly.

## Later-phase drift finding — 2026-08-09

- P02/P03 do not need reordering or merging: current data/setup can resolve immutable Boss bindings before runtime
  construction, while the runtime kernel remains a distinct high-risk lifecycle boundary.
- P05 remains the correct official publication point for all-difficulty and full-setup feasibility. P02 may build the
  pure typed setup seam and fixture validation without publishing a live opt-in.
- P04 can prove a Wire-neutral vertical slice with existing movement/damage/death presentation. P06 remains an
  explicit conditional cutover, not an assumed part of every Boss implementation.
- Current process topology has neither active-battle persistence nor hot data generation swap. The default
  no-in-place-migration assumption stands.

## Refactor registry decision — 2026-08-09

- Unconditional campaign refactor work is limited to 7 candidates coupled to P01/P02/P05.
- CRA-P10-C03 is conditional on an actual P06 public contract.
- P07-C03/C04 and P09-C05 wait until the P04 vertical slice proves the relevant seam; disposition is sufficient for
  Boss completion unless a later evidence-backed replan activates a separate cleanup Goal.
- The eight independent candidates remain separate Goals only. Shared low-level primitives may be coordinated, but
  their full implementation is not silently absorbed into Boss runtime work.

## P01 executable handoff decision — 2026-08-09

- `game_core` is the exact Cargo package and fully qualified `--lib -- --exact` filters are required; a short exact
  name matched zero tests and is durably recorded as a failed validation attempt.
- Three representative fully-qualified baseline tests ran once each and passed. They prove runner feasibility only,
  not G0 correctness coverage.
- P01 will be one child with C01→C02→C03 review milestones. Exact mutation owners, failing test names, failure
  injections and focused/broad commands are fixed in `P01_G0_HANDOFF.md`.
- The narrow C02 cross-variant decision preserves every priority band and makes AutoCastEnd precede ManualCastEnd at
  their shared band. A deterministic explicit variant key replaces incidental heap behavior.
- Current C03 failures can be injected without inventing a public seam: future cursor, revision overflow, invalid VN
  lookup and missing attempt-roster reconciliation. Projection is covered at its real cursor/revision fallible
  boundaries; current state transition itself is infallible.
- P00's investigation questions are now closed. Remaining work is artifact integrity, scoped dirty preservation and
  independent P00 completion review before P01 activation.
