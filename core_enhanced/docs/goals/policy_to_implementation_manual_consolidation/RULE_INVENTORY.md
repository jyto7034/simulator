# Policy-To-Implementation Semantic Rule Inventory

Inventory status: complete before manual cutover  
Inventory basis: the seven owner/companion documents frozen in `BASELINE.md` plus the approved consolidation prompt  
Granularity: one row per independent normative meaning or required procedure; rationale, examples, history, and reusable prose are covered by their owning rule unless they introduce a distinct condition

Location abbreviations:

- `RD`: `docs/README.md`
- `EW`: `docs/engineering_workflow.md`
- `RR`: `docs/policy_implementation_readiness_review_guide.md`
- `RT`: `docs/policy_implementation_readiness_goal_prompt_template.md`
- `GG`: `docs/goal_authoring_and_execution_guide.md`
- `CG`: `docs/goal_completion_review_guide.md`
- `ET`: `docs/goal_execution_resume_prompt_template.md`

The final owner column names the intended `policy_to_implementation_manual.md` partition. Compatibility files become non-normative entrypoints only after the atomic cutover.

## Source, authority, and ownership

| Rule ID | Current locations | Normalized meaning | Type | Conflict | Final owner | Disposition | Reason | Scenario |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `PIP-SOT-001` | RD 16–68; EW 25–66; RR 207–223; GG 86–99; CG 139–171 | Workspace actual, released actual, deployed actual, current canonical, approved target, and proposed implementation are distinct claims; unknown axes remain unknown. | normative | overlap | `PIP-00#source-model` | merged | Same precedence rule was repeated with fewer actual axes. | W12 |
| `PIP-SOT-002` | RR 63–96, 310–334; RT 267–315; GG 364–435; CG 78–138 | Every run identifies exact policy input, canonical, repositories/data/consumers, Goal evidence, revisions, and unresolved inputs. | procedure | overlap | `PIP-10#campaign-input` | merged | One normalized input contract replaces three checklists. | W1, W4 |
| `PIP-SOT-003` | RR 209–223, 445–523; GG 835–908; CG 139–171 | Current facts come from current reachable code/data/store/consumer evidence; target meaning comes from approved policy; Goal artifacts do not become product authority. | normative | identical | `PIP-00#authority-precedence` | merged | Core authority invariant. | W6, W12 |
| `PIP-SOT-004` | RR 117–158, 335–399; RT 379–390 | Alignment evidence and code-informed policy closure are separate: implementation implications may expose material questions before final freeze. | procedure | overlap | `PIP-20#closure` | merged | Preserves ATI-01/02 boundary without separate workflow. | W6 |
| `PIP-SOT-005` | RR 452–523; CG 172–225 | Current, target, acceptance, and implementation delta must be written separately and traced to observable evidence. | normative | overlap | `PIP-20#policy-record` | merged | Prevents target/current conflation. | W1, W3 |
| `PIP-SOT-006` | RR 1294–1331; GG 1246–1269; CG 360–383 | After a stable cutover, update the durable current canonical and record effective implementation revisions; never rewrite frozen target evidence to match implementation. | normative | orphan | `PIP-50#canonical-realization` | reworded | Existing text lacked one structured realization model. | W11 |
| `PIP-SOT-007` | GG 100–104; CG 519–562 | Out-of-scope findings are recorded and routed; they do not silently expand the active Goal or its completion claim. | normative | identical | `PIP-00#scope-integrity` | merged | Global scope safety rule. | W9 |
| `PIP-OWN-001` | RR 97–116, 601–658; GG 1052–1110; EW 349–368 | The user owns material product/policy choices; the agent owns evidence-backed technical realization, boundaries, modules, sequencing, and validation details. | normative | overlap | `PIP-00#decision-ownership` | merged | Central owner boundary. | W6, W9 |
| `PIP-OWN-002` | RR 601–658; GG 1052–1110 | DTO/schema/message ordering is user-owned only when it changes observable meaning, compatibility, persistence, or rollout; otherwise it is technical realization. | normative | mismatch | `PIP-20#materiality-test` | reworded | Resolves an implicit boundary. | W6 |
| `PIP-OWN-003` | GG 298–363, 715–776; CG 55–77 | Goal authoring, activation, mutation authorization, and completion review are separate grants or decisions; none implies another. | normative | identical | `PIP-00#separate-authorities` | merged | Safety-critical global rule. | W2, W3 |
| `PIP-OWN-004` | RR 260–309, 1113–1139; GG 364–435 | Execution mode and handoff mode bound allowed mutations; report-only and author-only never authorize production mutation. | normative | overlap | `PIP-10#handoff-mode` | merged | Normalizes mode vocabulary. | W1, W2 |

## Stages, gates, and traceability

| Rule ID | Current locations | Normalized meaning | Type | Conflict | Final owner | Disposition | Reason | Scenario |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `PIP-STG-001` | RR 35–182, 1292–1331; GG 25–58 | The lifecycle is direction/alignment → code-informed closure/freeze → readiness/dependency → Goal gate/preflight → implementation → Phase audit → Campaign audit. | normative | overlap | `PIP-00#pipeline` | reworded | ATI meaning is retained in a Campaign/Phase narrative. | W4 |
| `PIP-STG-002` | RR 37–62, 167–182, 1021–1058 | Each stage has an evidence-backed exit gate; document existence, prose confidence, or an earlier gate cannot substitute for the current gate. | normative | identical | `PIP-00#gate-rule` | merged | Global gate invariant. | W1, W7 |
| `PIP-STG-003` | RR 63–96, 1140–1220; RT 407–447 | Stable policy IDs trace bidirectionally through findings, acceptance, packages, Phase, runtime/data/consumer evidence, validation, realization, and audit. | normative | overlap | `PIP-00#traceability` | merged | Final policy, not finding/package, becomes primary unit. | W3, W5 |
| `PIP-STG-004` | RR 159–182, 815–898; GG 777–834 | Readiness handoff may report only, author inactive Goals, or author and activate exactly one eligible Goal; each mode has explicit evidence. | procedure | overlap | `PIP-40#handoff` | merged | Avoids a separate H1 workflow. | W1, W2 |
| `PIP-STG-005` | RR 899–955; GG 662–776 | Preflight revalidates the current tree, authority, consumers, dependencies, baseline, authorization, and exact activation target before mutation. | procedure | overlap | `PIP-40#preflight` | merged | One Phase entry contract. | W7, W8 |
| `PIP-STG-006` | RR 1294–1331; GG 1340–1388; CG 563–597 | Phase completion, next-Phase activation, and Campaign completion are separate transitions with separate audit evidence. | normative | identical | `PIP-60`, `PIP-70` | merged | Prevents implicit cascading approval. | W10 |
| `PIP-STG-007` | RR 1059–1080; GG 961–1008; CG 480–562 | A failed or invalid gate rewinds to the owner stage and invalidates only evidence dependent on the changed premise. | normative | mismatch | `PIP-80#transition-matrix` | reworded | Existing guides did not update every state axis consistently. | W7, W9 |
| `PIP-STG-008` | RR 1080–1112 | Fast Track may inline readiness/handoff artifacts but cannot skip source separation, closure, authorization, baseline, Goal gate, atomicity, canonical sync, or audit. | normative | orphan | `PIP-10#path-selection`, `PIP-40#fast-track` | reworded | Defines safe compression. | W13 |

## Goal structure, discovery, and lifecycle

| Rule ID | Current locations | Normalized meaning | Type | Conflict | Final owner | Disposition | Reason | Scenario |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `PIP-GOL-001` | GG 59–85, 436–519; RR 400–424 | Use Single when one atomic completion boundary suffices; use Master/Phase only for independently auditable dependency stages or one campaign envelope. | normative | overlap | `PIP-30#goal-architecture` | merged | One structure decision. | W3, W4 |
| `PIP-GOL-002` | GG 70–85, 909–923, 1340–1388 | Master coordinates policy coverage, dependencies, and audits but does not mutate production; Phase owns its allowed atomic mutation. | normative | identical | `PIP-30#master-phase` | merged | Core Master invariant. | W4 |
| `PIP-GOL-003` | consolidation prompt 185–200; RR 659–814 | Final approved policy is the traceability unit; work packages are dependency/atomicity analysis nodes and do not replace product scope. | normative | orphan | `PIP-30#policy-to-phase` | retained | New explicit normalization of existing practice. | W4, W5 |
| `PIP-GOL-004` | GG 520–661; ET 22–120 | New and resumed Goals must identify the exact current execution target, read direct dependencies, classify incomplete metadata, and pass the Goal Gate before work. | procedure | overlap | `PIP-40#goal-gate` | merged | Consolidates resume/new paths. | W2, W8 |
| `PIP-GOL-005` | GG 607–630 | Only explicit user/Master selection grants execution authority; unselected nonterminal Goals remain collision candidates, while terminal Goals are historical and never regain authority implicitly. | normative | contradiction | `PIP-00#goal-discovery` | superseded | Replaces “unnamed means archive” with safe limited discovery. | W8 |
| `PIP-GOL-006` | GG 961–1008, 1311–1339; CG 430–562 | Lifecycle, activation, coordination, authorization, and gate validity are orthogonal; duplicated metadata is derived, not another state. | normative | mismatch | `PIP-00#state-axes`, `PIP-90#metadata` | reworded | Removes semantic duplication without breaking Schema 2 history. | W7, W10 |
| `PIP-GOL-007` | GG 482–519, 961–1008 | Material policy/scope decisions and external blockers are explicit nonterminal states; they are not “partial completion.” | normative | identical | `PIP-80#nonterminal` | merged | Preserves honest status. | W6, W12 |
| `PIP-GOL-008` | GG 1340–1388 | A Phase may start only when direct prerequisites are Complete/High, Master selects it, its Gate is valid, and authorization is current. | normative | overlap | `PIP-40#activation` | merged | Single activation rule. | W4, W10 |
| `PIP-GOL-009` | GG 985–1008; CG 457–562 | Terminal complete/cancelled/superseded artifacts are immutable history; corrections use a new linked Goal and realization revision. | normative | orphan | `PIP-80#terminal-correction` | reworded | Adds explicit post-complete route. | W11 |

## Authorization

| Rule ID | Current locations | Normalized meaning | Type | Conflict | Final owner | Disposition | Reason | Scenario |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `PIP-AUT-001` | GG 298–363, 715–776; RR 260–309, 1113–1139 | Production mutation requires explicit provenance, exact scope/operations/effects, current validity, and no inherited authority. | normative | overlap | `PIP-00#authorization` | merged | Global safety invariant. | W2, W3 |
| `PIP-AUT-002` | GG 298–363 | Authorization lifecycle is pending/current/consumed/stale/revoked/N/A; only current permits mutation, and completion consumes it. | normative | overlap | `PIP-00#authorization-state` | merged | State owner fixed. | W7, W10 |
| `PIP-AUT-003` | GG 715–776; ET 51–104 | The execution gate must bind the current request, Goal, exact paths/operations, repository identity, baseline, and authorization artifact before implementation. | procedure | overlap | `PIP-40#goal-gate` | merged | One gate artifact. | W2 |
| `PIP-AUT-004` | GG 298–363; CG 55–77 | Read-only inspection and approved Goal-document authoring do not imply production authorization; external effects need their own explicit scope. | normative | identical | `PIP-00#authorization` | merged | Prevents authority creep. | W1, W2 |
| `PIP-AUT-005` | RR 1059–1079; GG 298–363 | Scope/policy/operation changes stale authorization only when the grant no longer covers the intended mutation; ordinary source drift stales gates/evidence, not the grant by itself. | normative | mismatch | `PIP-80#transition-matrix` | reworded | Precise invalidation avoids needless or unsafe reuse. | W7 |

## Dependency, atomicity, migration, and implementation

| Rule ID | Current locations | Normalized meaning | Type | Conflict | Final owner | Disposition | Reason | Scenario |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `PIP-SEQ-001` | RR 659–695; RT 391–399 | Group implementation by shared authority, persistence lifetime, schema cutover, transaction, and independently valid completion—not by document order or file count. | normative | identical | `PIP-30#package-model` | merged | Package definition. | W4, W5 |
| `PIP-SEQ-002` | RR 696–754 | Build typed dependency edges from concrete failure/evidence; contract SCCs into coimplementation or stop for a policy/scope decision. | procedure | identical | `PIP-30#dependency-audit` | merged | Core safe sequencing rule. | W4, W5 |
| `PIP-SEQ-003` | RR 733–814 | Each stage ends in a runnable/current-valid intermediate state with activation/completion gates and a stable seam; temporary duplicate authority is not a sequencing device. | normative | overlap | `PIP-30#stage-safety` | merged | Preserves intermediate-state safety. | W4, W5 |
| `PIP-SEQ-004` | RR 786–814 | Parallel work is allowed only with positive evidence of disjoint authority/files/contracts and safe merge/validation; same topological layer is insufficient. | normative | identical | `PIP-30#parallel-safety` | merged | Prevents unsafe concurrency. | W4 |
| `PIP-SEQ-005` | RR 224–259, 523–600 | Difficulty, risk, priority, confidence, and uncertainty are separate; finding scores are not summed into schedule or sequence. | normative | overlap | `PIP-20#assessment` | merged | Retains readiness assessment semantics. | W1 |
| `PIP-SEQ-006` | RR 956–1020; GG 1162–1245; CG 244–300 | Every Phase defines target-first acceptance, migration, rollout, compatibility, persistence, producer/consumer, and legacy removal together. | normative | overlap | `PIP-50#implementation-contract` | merged | One vertical cutover contract. | W3, W5 |
| `PIP-SEQ-007` | GG 1179–1200; RR 956–982; CG 284–300 | Defaults, fallbacks, dual schemas, ignored fields, and compatibility shims are forbidden unless explicitly approved, bounded, tested, and assigned removal criteria. | normative | identical | `PIP-50#compatibility` | merged | Safety rule. | W3 |
| `PIP-SEQ-008` | GG 1162–1178; RR 720–814 | If parts cannot independently be current, implement them atomically; do not publish placeholder schema/data/consumer states. | normative | overlap | `PIP-30#atomic-group`, `PIP-50#atomic-cutover` | merged | Crosses planning and execution via references. | W5 |
| `PIP-SEQ-009` | RR 983–1020; GG 1201–1245 | Establish a non-zero focused safety net before mutation, then run proportional broad validation and record omissions without turning skipped evidence into pass. | procedure | overlap | `PIP-50#validation` | merged | One validation contract. | W3, W12 |
| `PIP-SEQ-010` | GG 1246–1269; EW 67–101 | Canonical sync happens after stable implementation cutover and before completion audit; update only the owning durable contract. | normative | overlap | `PIP-50#canonical-realization` | merged | Normalizes timing. | W3, W11 |

## Evidence, dirty state, and validation

| Rule ID | Current locations | Normalized meaning | Type | Conflict | Final owner | Disposition | Reason | Scenario |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `PIP-EVD-001` | GG 269–297; CG 78–138 | Durable evidence references identify repository, path/artifact, revision or content hash, observation method, timestamp when material, and result. | normative | overlap | `PIP-00#evidence-reference` | merged | One evidence format. | W1, W7 |
| `PIP-EVD-002` | RR 425–442; GG 1111–1161; CG 301–314 | Freeze HEAD/status/hashes/owners and relevant runner baseline before mutation; later evidence must distinguish initial, user, and Goal changes. | procedure | overlap | `PIP-00#baseline-dirty` | merged | Global dirty rule. | W7, W8 |
| `PIP-EVD-003` | GG 1111–1161; EW 403–411 | Preserve unrelated dirty/untracked work; never reset, restore, stash, delete, bulk-format, or attribute pre-existing changes to the Goal. | normative | overlap | `PIP-00#baseline-dirty` | merged | Safety-critical rule. | W8 |
| `PIP-EVD-004` | RR 485–523; CG 226–283 | Trace reachable production authority from input through state/data/persistence/transport/consumer; tests and types alone do not prove production reachability. | procedure | overlap | `PIP-20#boundary-trace`, `PIP-50#validation` | merged | Code-informed evidence rule. | W6, W12 |
| `PIP-EVD-005` | GG 1201–1245; CG 315–359 | Record exact command, non-zero executed count, pass/fail/skip, baseline failures, environment limits, and relevance; do not relabel unexplained failures. | normative | identical | `PIP-00#validation-evidence` | merged | Global evidence semantics. | W3, W12 |
| `PIP-EVD-006` | CG 270–283; RR 485–523 | Consumers are confirmed, inaccessible, excluded, or unknown with effect on claims; inaccessible evidence may block only the affected contract. | normative | overlap | `PIP-00#consumer-evidence` | merged | Honest claim boundary. | W12 |
| `PIP-EVD-007` | CG 301–383; GG 1270–1310 | Completion evidence is read-only against final implementation/canonical state and includes scoped diff, legacy, compatibility, consumers, validation, and dirty preservation. | procedure | overlap | `PIP-60#audit-input` | merged | Phase audit contract. | W3, W9 |

## Audit, verdict, and finalization

| Rule ID | Current locations | Normalized meaning | Type | Conflict | Final owner | Disposition | Reason | Scenario |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `PIP-AUD-001` | CG 28–54, 172–225; GG 1270–1310 | Audit every completion condition and invariant against current evidence in an explicit matrix; checklist prose is not proof. | procedure | overlap | `PIP-60#completion-audit` | merged | Core audit procedure. | W3 |
| `PIP-AUD-002` | CG 480–562; GG 1270–1339 | Only `Complete / High` permits completion finalization; partial/document-only/policy/scope/external outcomes remain accurately nonterminal or not applicable. | normative | overlap | `PIP-60#verdict` | merged | Single completion gate. | W9 |
| `PIP-AUD-003` | CG 384–429 | Audit long-term direction separately from immediate correctness; refactor candidates do not block unless required by approved completion. | procedure | identical | `PIP-60#fit` | merged | Preserves Fit dimension. | W3 |
| `PIP-AUD-004` | CG 430–456; GG 1270–1310 | Reviewer independence is proportional to risk; medium risk may use the same implementer only in a fresh separate read-only pass. | normative | identical | `PIP-60#reviewer` | merged | Reviewer contract. | W3 |
| `PIP-AUD-005` | CG 457–479; GG 1311–1339 | After an audit candidate freezes, only enumerated deterministic metadata/link finalization may occur; any material edit invalidates and reruns the audit. | normative | identical | `PIP-60#finalization` | merged | Prevents self-fulfilling audits. | W10 |
| `PIP-AUD-006` | CG 563–597; GG 1340–1388 | Campaign audit independently verifies cross-Phase policy coverage, seams, canonical realization, regression evidence, and no hidden deferred work. | procedure | overlap | `PIP-70#campaign-audit` | merged | Master exit contract. | W10 |
| `PIP-AUD-007` | RR 1324–1331; CG 578–597 | A Campaign audit failure rewinds the owning Phase or Campaign integration stage and never rewrites a completed Phase’s historical audit. | normative | overlap | `PIP-80#transition-matrix` | merged | History-safe correction. | W10 |
| `PIP-AUD-008` | GG 1389–1449; CG 701–744; RR 1373–1410 | Final reports state verdict, realized scope/revisions, tests and omissions, canonical status, remaining risks, authorization disposition, and next allowed action. | template | overlap | `PIP-90#report-template` | merged | One reporting schema. | W1, W10 |

## Templates and compatibility

| Rule ID | Current locations | Normalized meaning | Type | Conflict | Final owner | Disposition | Reason | Scenario |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `PIP-TPL-001` | RT 25–94, 95–475 | A reusable Campaign/readiness prompt supplies policy inputs, scope, repositories/consumers, handoff mode, authorization, and requested path; it invokes manual rules rather than copying them. | template | overlap | `PIP-90#campaign-prompt` | reworded | Template becomes a thin invocation contract. | W1, W4 |
| `PIP-TPL-002` | ET 22–120 | A Goal execution/resume prompt identifies exact Goal, intended action, authorization source, and requested result; the Goal Gate derives the rest. | template | overlap | `PIP-90#execution-prompt` | reworded | Removes shadow checklist. | W2 |
| `PIP-TPL-003` | GG 1502–1625; CG 598–700; RR 1167–1220 | PLAN, Master, stage, and completion-audit skeletons are formats only and must reference the normative state/evidence rules. | template | overlap | `PIP-90#artifact-templates` | merged | One schema owner. | W3, W4 |
| `PIP-TPL-004` | RD 69–177; each guide “문서 책임” | README owns discovery/index; engineering workflow owns generic engineering practice; the manual is the only current PIP norm owner; old guides preserve links only. | normative | mismatch | `PIP-00#document-ownership` | reworded | Enables atomic owner cutover. | all |
| `PIP-TPL-005` | RR 1347–1372; RT 476–503; GG scattered examples/failure patterns; CG examples/templates | Examples, failure patterns, rationale, and history may illustrate owner rules but cannot add an unnumbered exception or checklist. | normative | overlap | `PIP-00#normative-owner` | merged | Compression rule protects semantics. | all |

## Section-level non-rule disposition

| Material | Current locations | Type | Disposition | Owner linkage |
| --- | --- | --- | --- | --- |
| Long introductions and repeated “why this matters” prose | RR 183–206; GG introductions; CG 28–54 | rationale | compressed | `PIP-STG-001`, `PIP-AUD-001` |
| Repeated failure-pattern lists | RR 1347–1372; GG stop-condition examples; CG verdict examples | example | compressed to boundary cases | Relevant `PIP-20/30/40/50/60/80` local contracts |
| Historical ATI naming and old guide role descriptions | RR 35–182; guide responsibility sections | history | retained as crosswalk, not owner | `PIP-00#ati-crosswalk`, compatibility entrypoints |
| Full reusable prompt prose | RT 95–475; ET 51–104 | template | superseded by thin inputs | `PIP-TPL-001`, `PIP-TPL-002` |
| Duplicate PLAN/audit skeletons | GG 1502–1625; CG 598–700 | template | merged | `PIP-TPL-003` |
| README pipeline explanation | RD 93–162 | rationale/index | reduced to link and short synopsis | `PIP-TPL-004` |
| Engineering PIP lifecycle and Goal template | EW 120–145, 369–402 | procedure/template | superseded for PIP use; generic workflow retained | `PIP-STG-001`, `PIP-TPL-003` |

## Inventory totals and preservation verdict

- Independent Rule IDs: 63.
- Types: normative 45, procedure 14, template 4.
- Dispositions: retained 1, merged 50, reworded 11, superseded 1, removed 0.
- Conflict-bearing rows: contradiction 1, mismatches 5, orphans 4; all have an explicit owner and are resolved in `CONFLICT_REGISTER.md` or the target model.
- No independent normative meaning is assigned to README, engineering workflow, a compatibility guide, or a prompt entrypoint after cutover.
- “Removed” is zero: prose compression removes repetition and examples, not an unowned safety meaning.
