# Boss Runtime v1 Implementation Master — Notes

## Approved structural decision

- Main track: production-ready Boss Runtime v1 implementing BP-01~BP-34 and one live reference Boss.
- Sub-track: existing refactor candidates classified by actual Boss dependency rather than treated as one blanket
  prerequisite.
- P00 preflight belongs inside this Master as the first child Phase. A standalone preflight-before-Master Goal is not
  used because readiness to form the campaign is already established by the completed audit.
- The Master does not implement production changes. Child Phase activation is a separate explicit action.
- Goal and child names are document identifiers assigned by the agent from the approved scope and repository naming
  convention. They do not require a separate user naming approval and do not authorize Phase activation.

## Initial repository baseline

- Root: `/mnt/f/work/simulator`
- Commit: `f318414fb0156a67e04c5479000c2a2683fc0c09`
- Status entry count: 218
- Status SHA-256: `9f64c0247e80064c4a78136d0c3cc5a89c8730206da937d96b2abd5fb7774a96`
- The dirty/untracked worktree contains user-owned changes. This Master will not clean, restore, move or absorb them.

## Authority and lifecycle

- `PLAN.md` is the sole Master-level Phase status and dependency registry.
- Target policy remains the BP decision record; current runtime/data/canonical remain authoritative for current
  behavior until a Phase implements and documents the target.
- The completed readiness audit is evidence input, not production authority.
- Child Goal documents own their implementation evidence and must be created only when activated.
- Conversation context is transient; Phase transitions and cross-Phase findings are recorded immediately.

## Plan change policy

- The current P00~P07 order is an evidence-based execution baseline, not an immutable schedule.
- P00, implementation, validation or completion audit may trigger controlled split/merge/reorder, a correction Phase or
  a conditional disposition when current evidence invalidates a planning assumption.
- Technical replanning inside the approved final result does not require a new gameplay-policy approval, but it must be
  recorded in the affected child and Master before dependent production continues. Child activation remains explicit.
- Any change to user-visible meaning, approved scope, compatibility policy or final completion meaning still requires
  user approval. BP coverage and the first-live-reference-Boss outcome cannot be reduced by replanning.
- Audit-only work records the defect and routes correction back to an implementation Phase or a separately registered
  correction Goal; it does not mutate production directly.
- Replanning evidence is written when discovered, not reconstructed in a batch after the Phase or conversation context
  has been compacted.

## Preliminary risks for P00

- Current status has changed since the readiness audit baseline, so P00 must distinguish material Boss-boundary drift
  from unrelated concurrent work.
- Unity was stale and concurrently changing during the readiness audit; current relevant consumer paths must be
  rechecked only if the actual modified surface touches Boss replay/presentation conclusions.
- P02/P03 may be one or two atomic implementation boundaries depending on current source ownership; P00 must decide
  without weakening policy coverage.
- P06 should not be forced early if P04/P05 use existing events only, but final v1 completion must explicitly decide
  whether a public Boss lifecycle is required by the approved live reference Boss.
- Independent refactor work is not automatically safe to parallelize merely because its candidate disposition says
  independent; file/authority/test overlap must be checked at activation time.

## Alternatives not adopted

### Complete all refactors first

Rejected because most candidates are not Boss gates and early cleanup can pre-shape or remove the unproven Boss seam.

### Implement Boss first and refactor reactively

Rejected because P06-C01/C02/C03 are known correctness prerequisites and coimplementation boundaries are already
identified. Refactor timing is registry-driven, not ad hoc.

### Standalone preflight before creating a Master

Rejected because the completed readiness audit plus the user's approved final objective already justify a Master.
P00 is instead the first non-production Phase and gates every production child.

## Current open questions

- The first live reference Boss identity, required behavior vocabulary, balance/assets and presentation boundary are
  deferred for explicit user approval before P05. This does not block P00~P04.
- P06 activation is derived from the approved reference Boss's actual public presentation contract. P00 defines the
  evidence gate; P05 produces the content evidence; the Master then records P06 activation or an explicit
  `not_applicable_to_v1` disposition.
- P00 must confirm whether deployment can retain an active battle across binary/data generations. If it can, the
  default no-in-place-migration assumption needs a user-visible session cutover decision before P05.

## Goal-authoring compliance correction — 2026-08-09

- The authoring guide now states that `goal_name` is an agent-assigned document identifier when the user does not
  provide one. Separate naming delegation/approval is not an execution input.
- The Master registry now contains stable proposed names for P00~P07 instead of `name pending` placeholders. Boundary
  changes found by P00 may rename a child only with history and policy coverage preserved.
- `PLAN.md` now separately declares implementation scope, DTO/data/save migration, deployment/compatibility,
  focused/broad/omitted validation and the required workflow/canonical inputs.
- The original initialization inspected representative Master PLAN files before creating the three required artifacts.
  Under the guide's strict pre-document allowlist this was a process-order deviation: it was limited to documentation
  patterns, performed no runtime/code investigation and caused no production mutation. It is retained here rather than
  retroactively described as compliant.

## External access and omitted validation

- The external Unity tree is a read-only consumer for this campaign. It was stale at readiness-audit time and is not
  considered verified for a future public lifecycle until the relevant Phase checks its actual current source and
  available build/test surface.
- No runtime or test runner is used for Master authoring/compliance edits because they mutate no production contract.
  Structural coverage, child non-creation and dirty preservation are the applicable validations.
- Final-boss-specific content, unapproved art/VFX/animation and independent CRA candidates remain unvalidated here
  because they are explicit non-scope, not because they are assumed correct.

## Master document structural checkpoint

- The actual Master objective is the complete Boss Runtime v1 platform plus one live reference Boss, not merely the
  fixture slice. Final-boss-specific content remains separate.
- P00 is an internal investigation Phase and the only ready child candidate. No child directory or production action
  exists yet.
- P01 owns the three shared correctness candidates; P02 owns strict authoring/setup; P03 owns Core runtime and
  target/movement seams; P04 proves the existing-Wire fixture; P05 publishes one live reference; P06 is conditional on
  a real public lifecycle; P07 audits the combined program.
- BP-01~BP-34 expand to exactly 34 Phase-coverage IDs without duplicate or omission. The 19 relevant CRA IDs are exact
  and remain a sub-track registry rather than policy authority.
- P02/P03 and P05/P06 are preliminary atomic boundaries subject to P00 source/collision evidence. P00 may refine the
  split but cannot weaken coverage or activate production.
- Master-excluded dirty status shape is identical to initialization. Master work is still confined to its three
  documents; this compliance correction additionally edits only `goal_authoring_and_execution_guide.md`.

## P00 activation checkpoint — 2026-08-09

- The user instructed execution of the full Master, so P00 is now the sole active Phase.
- P00 is investigation-only; activation does not authorize P01 or any production mutation.
- P01 remains pending until P00 independently reaches `Complete / High` and `ready_for_g0`.
- P00 owns current drift/collision evidence, the 19-candidate execution registry and the exact G0 handoff.

## P00 completion checkpoint — 2026-08-09

- P00 independently reached `Complete / High` and `ready_for_g0`; all eight completion conditions pass.
- Current-tree evidence did not require a Phase boundary replan. One P01 child with C01→C02→C03 milestones is the
  approved next execution unit.
- The current dirty tree remains the user-owned implementation baseline. P01 must use symbol-level test-first edits.
- P05 reference identity approval and the resulting P06 activation decision remain intentionally deferred and do
  not block P01~P04.
- Master coordination is now `Active Phase: none`, `P01: ready`; production remains unauthorized until P01 child
  activation is durably recorded.

## P01 activation checkpoint — 2026-08-09

- P01 is now the sole active child; P02 is blocked.
- The Phase owns only terminal authority, total event order and atomic world advance corrections.
- All potentially touched source/test files were hashed before activation because six of seven are already dirty.
- Implementation starts with failing C01 characterization tests; Boss-specific code remains prohibited.

## P01 completion checkpoint — 2026-08-09

- P01 reached `Complete / High` and `ready_for_p02`; all three correctness candidates are implemented and broadly
  green.
- The child records every failing characterization, compile/fixture/format correction and truncated-output recovery
  at occurrence time. Completion evidence was not reconstructed from conversation context.
- P01 added no Boss-specific data/runtime/Wire surface. P02 now owns the first Boss catalog and typed setup work.
- User dirty state is preserved against the exact activation baseline. Master is between Phases with P02 `ready`, not
  yet active.

## P02 activation checkpoint — 2026-08-09

- P02 is the sole active child; P03 is blocked.
- It owns strict definitions/compilation, the one embedded source inventory and pre-Core typed setup resolution.
- The source-count canonical drift (21 documented versus 22 actual before adding Boss) is an explicit first milestone.
- P02 may publish an empty Boss catalog foundation but cannot opt an official encounter into Boss behavior.

## P02 completion checkpoint — 2026-08-09

- P02 reached `Complete / High` and `ready_for_p03`; strict authoring, atomic publication and typed setup are now a
  verified foundation rather than runtime behavior.
- Production content still has no Boss opt-in, so no-binding gameplay remains the active live path.
- P03 must consume the captured compiled definition/frame without raw catalog lookup and owns all hierarchical
  activation, checked deadlines, transition arbitration, target/path execution and rollback semantics.
- CRA-P01-C01 and CRA-P05-C04 are complete at their Boss-required seams. The remaining refactor catalog is neither
  implicitly complete nor authorized by this checkpoint.
- Master is between Phases with P03 `ready`; production mutation is not authorized until the P03 child is created and
  explicitly activated from a fresh dirty/owner baseline.

## P03 activation checkpoint — 2026-08-09

- P03 is now the sole active child; P04 is blocked.
- It consumes P02's captured definitions/frames and owns runtime activation, arbitration, checked deadlines,
  target/static-grid query and Boss movement directives inside `BattleCore`.
- No additional refactor candidate is authorized. Existing movement/attack/skill/damage/death modules remain leaf
  authorities and production Boss data remains empty.
- Implementation begins with exact non-opt-in characterization and runtime construction tests before source mutation.

## P03 completion checkpoint — 2026-08-09

- P03 reached `Complete / High` and `ready_for_p04`; runtime lifecycle, arbitration, target/path and movement/action
  ownership now form an audited internal Core contract.
- Production Boss collections remain empty and public Wire/checkpoint/save remains unchanged. P04 must prove the
  authored-to-server existing-event slice without converting test fixture data into official live publication.
- Unity remains a stale external consumer baseline. P04 may rely only on source evidence that unchanged
  movement/damage projection is sufficient; it must not claim current Unity parity or mutate that external tree.
- Whole-Core Boss-boundary cloning is a performance watch for representative P04/P05 content, not an open P03
  correctness or refactor blocker.

## P04 activation checkpoint — 2026-08-09

- P04 is now the sole active child; P05 is blocked.
- The Phase must cross the actual strict bundle/setup, world/Core lifecycle and existing server projection rather than
  proving only private Boss-runtime helpers.
- Fixture content stays test-only and public transport stays unchanged. A discovered need for Boss-specific Wire or
  durable graph restore stops P04 and becomes P06 decision evidence.
- Unity remains stale read-only consumer evidence and is not current producer authority.

## P04 completion checkpoint — 2026-08-09

- P04 reached `Complete / High` and `ready_for_p05_decision`. Strict raw source publication, exact typed world actor
  identity, full TrackAndRejoin success, paired ordinary behavior, explicit failure outcomes, Pause, interruption and
  reconnect all have current-tree evidence.
- Existing battle events/checkpoint and game_server transport are sufficient for this reference slice. P04 neither
  activates P06 nor proves that P06 is unnecessary for the eventual official Boss; P05 content supplies that decision
  evidence.
- Production Boss RON remains empty. The raw fixture is not an implicit official identity/content choice.
- Master coordination is `Active Phase: none`. P05 activation is blocked only by the deferred explicit user approval
  of the first live reference Boss identity/content boundary.

## P05 policy-decision checkpoint — 2026-08-09

- Master is `policy_decision_required`, with no active child. P05 has not been created or activated.
- The missing input is not an implementation detail: identity, content/balance/assets and presentation semantics
  determine live data, canonical meaning and whether the synchronized public P06 cutover is required.
- Until approval, only read-only candidate/impact investigation is authorized. No production RON, runtime, DTO,
  server or Unity mutation may start.
- Current read-only recommendation is Little Red with a finite pursuit Pattern, unchanged balance/rewards and
  existing-event transport. The exact approval choices and rejected alternatives are durable in
  `P05_CONTENT_DECISION_PACKET.md`.
- External Unity is now accessible and must not be described as unavailable. Its current battlefield uses generic
  opponent capsules and has no Little Red-specific battle asset mapping; this makes the asset/presentation choice a
  real user-visible gate rather than an implementation default.

## P05 approval and activation checkpoint — 2026-08-09

- The user approved decision-packet option 1 and explicitly allowed the agent-authored Little Red behavior to be
  disposable test/reference content because final content will be redesigned later.
- The approved behavior is therefore a live validation fixture, not a claim that Little Red's final lore, bounty
  system, balance or presentation has been designed. P05 must keep current stats/rewards/skill, use the common
  opponent capsule and existing public transport, and add no dedicated bounty marker, art or VFX.
- P05 is the sole active Phase. Its child owns all production mutation. The Master remains orchestration-only.
- P06 is not silently skipped or pre-activated. P05 must first prove that the approved live path is completely
  represented by existing spawn/movement/skill/damage/death/checkpoint transport.

## P05 completion and final-audit checkpoint — 2026-08-09

- P05 is `Complete / High`. The one live Little Red reference is disposable content, not final character canon.
- Existing Core/server/current Unity event and checkpoint semantics are sufficient for the actual reference, so P06
  is explicitly `not_applicable_to_v1` and has no child directory.
- P07 is active in `audit_only` mode. Any material production defect found there must reopen an implementation Phase
  or create an approved correction Goal; P07 cannot repair it while preserving an exit verdict.

## Master completion checkpoint — 2026-08-09

- P07 found no material production, policy, canonical, consumer, migration, legacy or validation gap and completed
  `Complete / High` without product mutation.
- BP-01~BP-34 are exact and fully passing. P00~P05 are all independently `Complete / High`; P06 is an evidence-backed
  `not_applicable_to_v1`, not an omitted implementation.
- Final current workspace validation passes Core 930, integrations 3 + 4 + 25 + 13 + 4 and server 56, with zero
  failures/ignored. Formatting and diff checks pass.
- The Master is complete. Little Red remains disposable reference content; public lifecycle, final boss content,
  optional cleanup and independent refactors require separate future authorization.
