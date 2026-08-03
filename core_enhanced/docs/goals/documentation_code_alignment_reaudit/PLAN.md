# Documentation–Code Alignment Reaudit

Status: complete

Implementation complete: true

Started: 2026-07-18

## Objective

Verify, with traceable runtime/data/server/Unity/test evidence, that the current durable
`core_enhanced` documentation describes the behavior implemented by the current live codebase, and
classify every material mismatch without changing policy or implementation during the audit.

## Audit Character

This is an audit-only goal. Reading, search, test execution, probe execution when justified, and
audit-document updates are in scope. Production code, live RON, tests, durable canonical documents,
external Unity contracts, and Unity implementation are not changed as part of this goal.

An identified correction requires explicit user approval and a separate correction goal. This
separation prevents the audit from silently making code match a document, or making a document match
code, before deciding which side expresses the intended policy.

## Source Of Truth And Interpretation

Use the project evidence order defined by `docs/README.md`:

1. actual runtime code;
2. live RON/data;
3. Unity-facing snapshot/command/WebSocket contracts and their producers/consumers;
4. current durable policy/contract documents;
5. goal documents and historical audit records.

This order identifies current behavior; it does not automatically decide intended policy. A
gameplay, UX, balance, schema, persistence, reward/failure timing, or external DTO mismatch is
classified as `Policy decision needed` unless current approved policy makes the correction
unambiguous.

## Primary Documentation Scope

Current durable Core documents:

- `docs/README.md`
- `docs/game_rulebook.md`
- `docs/skill_target_contract.md`
- `docs/core_runtime_contract.md`
- `docs/data_loading_contract.md`
- `docs/audit/live_skill_catalog_evidence_contract.md`
- `docs/engineering_workflow.md`
- `docs/probe_registry.md`

Specialized documents are audited where their claims touch live behavior:

- `docs/skills/lobotomy_content_catalog.md`
- `docs/skills/abnormality_skill_design_notes.ko.md`
- `docs/skills/skill_fragment_system.md`
- `docs/skills/live_content_index.md`
- `docs/decisions/*.md`

External Unity canonical documents and implementation surfaces:

- `/mnt/f/unity projects/ark/docs/unity_core_contract.md`
- `/mnt/f/unity projects/ark/docs/core_unity_battle_transport_contract.md`
- relevant Unity DTO consumers, presenters, command producers, tests, and probes

Historical inputs are regression checklists, not current authority:

- `docs/archive/audits/documentation_code_alignment_audit_2026-07-10.md`
- `docs/archive/audits/comprehensive_project_review_2026-07-10.md`
- `docs/goals/cross_layer_contract_hardening_master/`
- other goal records linked by current code or canonical documents

## Implementation And Data Scope

Trace claims through the current live path as applicable:

- `src/game/**`
- `tests/**`
- `../game_resources/data/**`
- `../game_server/src/**`
- relevant server tests and examples
- external Unity implementation, tests, and probe contracts

Generated logs, debug exports, `target/`, `tmp/`, completed goal records, and archived reports are not
treated as behavior authority unless an executable test explicitly establishes them as fixtures.

## Claim-Level Audit Method

Do not issue a verdict for an entire document based on representative sampling. Split each
substantive canonical section into independently checkable claims and maintain a coverage ledger.
For each claim:

1. Restate the expected behavior in one precise paragraph.
2. Identify the authoritative producer or state transition.
3. Follow input/authoring, parsing, validation, mutation, projection/serialization, and consumption
   paths that apply to the claim.
4. Inspect relevant live RON/data and cross-catalog validation.
5. Inspect Core-to-server-to-Unity boundaries when the behavior is externally visible.
6. Search for old names, competing producers, fallback paths, compatibility shims, ignored tests,
   debug-only bypasses, and consumer-side recomputation.
7. Identify tests/probes and state exactly which part of the claim each one proves.
8. Run the smallest focused validation capable of exercising the behavior.
9. Record evidence, verdict, severity, remaining risk, and the next check.

Each claim must have evidence or an explicit `Insufficient evidence` verdict. Absence of a discovered
mismatch is not evidence by itself.

## Verdicts

Each claim receives exactly one primary verdict:

- `Aligned`: document, runtime, live data, external boundary, and applicable test evidence agree.
- `Document drift`: current implementation layers agree but durable documentation is stale or
  overstates behavior.
- `Implementation drift`: current canonical policy is clear but runtime or live data does not
  implement it.
- `Cross-layer drift`: Core, server, Unity contract, or Unity implementation use competing meanings
  or shapes.
- `Policy decision needed`: evidence exposes an unresolved product, gameplay, UX, schema, balance,
  persistence, or external-contract choice.
- `Insufficient evidence`: the claim cannot yet be traced or executed far enough to support another
  verdict.
- `Not applicable`: a current approved policy or removed live path makes the claim inapplicable.

Record `Test gap` as an additional evidence finding when behavior appears aligned but no meaningful
test protects it. A passing smoke or membership test is not behavior evidence beyond its assertions.

Severity:

- `P0`: data loss, unrecoverable state, authoritative divergence, or release-blocking protocol fault.
- `P1`: material player-visible behavior or live integration mismatch.
- `P2`: real but bounded contract/document/test gap.
- `P3`: discoverability, terminology, or low-risk evidence weakness.

## Audit Passes

### Pass 0 — Baseline And Inventory

- Record the dirty worktree without modifying or attributing pre-existing changes.
- Inventory current durable documents, live data entrypoints, runtime modules, server boundaries,
  Unity boundaries, tests, and probes.
- Build the canonical-section claim ledger.
- Record a focused baseline before interpreting new test failures as findings.

### Pass 1 — Gameplay And Run Flow

- Audit modes, run start, recruitment, Floor/map progression, node selection and sessions.
- Audit combat start/result, retreat, failure, recovery, reward, research, inventory, growth, employee
  condition, Psychology, and BossOmen behavior.
- Trace allowed-action/capability ownership instead of inferring availability from handlers alone.

### Pass 2 — Battle Runtime And Targeting

- Audit determinism/RNG identity, setup, movement, blocking, attack, damage, skill cast/step,
  projectile, TileArea, automatic targeting, range preview, event log, checkpoint, result stats, and
  atomic result commit.
- Confirm canonical state, derived projections, presentation events, and debug outputs are not used
  interchangeably.

### Pass 3 — Data And Content

- Audit embedded bundle ownership, strict schema parsing, loader publication boundary, local and
  cross-catalog validation, generated preview validation, and fixture separation.
- Compare content/index claims with live RON without treating design-only content as implemented.
- Audit live skill manifest membership evidence separately from behavior coverage.

### Pass 4 — Server And Unity Boundary

- Compare typed Core projections, server mapping, serialized envelopes, external canonical fields,
  Unity command producers, and Unity consumers.
- Audit casing, optionality, identity, revision/sequence, message ordering, reconnect/resync, setup
  recovery, and final result flows.
- Check that Unity does not recompute Core-owned gameplay rules from authored metadata.

### Pass 5 — Tests, Legacy, And Cross-Component Review

- Map each material claim to focused tests or probes and identify smoke-only or implementation-shaped
  evidence.
- Search for stale fields/types, dual schemas, fallbacks, ignored tests, stale local contract copies,
  and unreachable but misleading live-looking code/data.
- Recheck historical findings for regression without inheriting their old verdicts.
- Review whether individually aligned components define competing sources of truth when combined.

### Pass 6 — Validation And Report

- Run focused validation throughout the audit and the smallest justified broad suites last.
- Produce `AUDIT_REPORT.md` with coverage summary, claim/finding table, detailed material findings,
  cross-component findings, validation evidence, policy questions, and prioritized correction goals.
- Leave durable canonical documents and production artifacts unchanged until follow-up work is
  explicitly authorized.

## In Scope

- Read-only inspection of current code, live data, server, external Unity documents, Unity consumer
  paths, tests, and goal/history evidence.
- Execution of non-mutating searches, focused tests, broad tests, and justified registered probes.
- Updates to this goal's `PLAN.md`, `EXPERIMENT_NOTES.md`, `EXPERIMENTS.md`, and final
  `AUDIT_REPORT.md`.
- Precise correction proposals with affected files, policy owner, risk, and validation requirements.

## Out Of Scope

- Fixing production code, tests, live data, canonical documents, server/Unity implementation, or
  external contracts during the audit.
- Inventing gameplay, UX, balance, content, migration, or wire policy.
- Restoring deleted legacy files or compatibility behavior.
- Treating goal completion claims, archived findings, logs, or generated output as current policy.
- Reformatting or reorganizing unrelated files.
- Cleaning pre-existing dirty-worktree changes or generated artifacts not created by this audit.

## Completion Conditions

- Every substantive section of the primary durable documents is represented in the coverage ledger
  and has a verdict or an explicit reason it is non-behavioral.
- Every material claim has runtime/data/server/Unity/test evidence appropriate to its path, or is
  explicitly `Insufficient evidence` with a concrete blocker and next check.
- Every mismatch has a stable finding ID, severity, owner boundary, evidence links, and one primary
  verdict.
- Cross-component source-of-truth conflicts and historical-finding regressions are reviewed
  separately from per-document findings.
- Focused and broad validation commands, timestamps, results, and environmental limitations are
  recorded in `EXPERIMENTS.md`.
- `AUDIT_REPORT.md` distinguishes documentation drift, implementation drift, cross-layer drift,
  policy questions, test gaps, and non-issues.
- Proposed corrections are grouped into separately approvable follow-up goals; none are silently
  implemented by this audit.
- The final report states remaining coverage limitations and does not claim complete alignment while
  any material `Insufficient evidence` item remains unexplained.
- 사용자와 의논하여 정해야 할 정책이 있을 경우 goal을 종료한다.

## Stop Conditions

Stop the affected audit branch and record a question instead of choosing a side when evidence would
require any of the following decisions:

- Unity-facing DTO/message shape or consumption-order change;
- live RON schema, persistence, or save migration;
- gameplay, UX, balance, reward, failure, consumption, retreat, death, or node-flow policy;
- deletion or replacement of existing content;
- deciding between conflicting current documents when neither is demonstrably historical;
- running a probe that requires credentials, production access, or external state not already placed
  in scope.

A stopped branch does not prevent progress on independent audit claims.

## Initial Validation Menu

Select commands by claim; do not run all of them merely to create activity:

```text
cargo check -p game_core
cargo test -p game_core --test ron_loading -- --nocapture
cargo test -p game_core --test live_skill_catalog_audit -- --nocapture
cargo test -p game_core --test skill_runtime_contract -- --nocapture
cargo test -p game_core
cargo check -p game_server
cargo test -p game_server
```

Use `docs/probe_registry.md` for live WebSocket/admin probe selection and record the exact server
command, probe command, environment, and observed result.

## Completion Report Requirements

Report:

- audited documents and code/data/consumer surfaces;
- coverage ledger totals by verdict and severity;
- material findings with exact evidence;
- cross-component and historical-regression results;
- tests/probes run and what each result proves;
- policy questions and blocked branches;
- recommended correction goals in priority order;
- remaining risks and coverage limitations.
